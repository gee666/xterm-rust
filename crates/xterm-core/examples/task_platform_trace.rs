//! Line-oriented trace endpoint for scripts/task-platform-oracle.mjs.
//! Queue unwraps eliminate the recording host's Infallible error type. The
//! fixture has no host failure path; fallible hosts have separate tests.
use std::{
    cell::{Cell, RefCell},
    error::Error,
    io::{self, BufRead},
    rc::Rc,
};
use xterm_core::{
    buffer::{AttributeData, BufferLine},
    circular_list::CircularList,
    platform::{Navigator, Platform},
    task_queue::{CallbackKind, Scheduler, TaskQueue},
    windows_mode::update_windows_mode_wrapped_state,
};

#[derive(Clone, Default)]
struct Host {
    clock: Rc<Cell<f64>>,
    events: Rc<RefCell<Vec<String>>>,
    token: Cell<usize>,
}
impl Scheduler for Host {
    type Handle = usize;
    type Error = std::convert::Infallible;
    fn request(&self, kind: CallbackKind) -> Result<usize, Self::Error> {
        self.token.set(self.token.get() + 1);
        self.events
            .borrow_mut()
            .push(format!("request:{}:{kind:?}", self.token.get()));
        Ok(self.token.get())
    }
    fn cancel(&self, token: usize) {
        self.events.borrow_mut().push(format!("cancel:{token}"));
    }
    fn now(&self) -> f64 {
        self.clock.get()
    }
    fn deadline_exceeded(&self, ms: f64) -> Result<(), Self::Error> {
        self.events.borrow_mut().push(format!(
            "warn:task queue exceeded allotted deadline by {ms}ms"
        ));
        Ok(())
    }
}
fn decode_hex(text: &str) -> Result<String, Box<dyn Error>> {
    if text.len() % 2 != 0 {
        return Err("odd hex length".into());
    }
    let bytes: Vec<u8> = text
        .as_bytes()
        .chunks(2)
        .map(|chunk| Ok(u8::from_str_radix(std::str::from_utf8(chunk)?, 16)?))
        .collect::<Result<_, Box<dyn Error>>>()?;
    Ok(String::from_utf8(bytes)?)
}
fn task(
    queue: &TaskQueue<Host>,
    host: &Host,
    id: usize,
    repeats: usize,
    duration: f64,
    action: usize,
) {
    let host = host.clone();
    let calls = Cell::new(0);
    queue
        .enqueue(move |queue| {
            host.events.borrow_mut().push(format!("task:{id}"));
            host.clock.set(host.clock.get() + duration);
            let first = calls.get() == 0;
            calls.set(calls.get() + 1);
            if first {
                if action == 2 || action == 3 {
                    queue.clear();
                }
                if action == 1 || action == 3 {
                    task(queue, &host, id + 10000, 1, 0.0, 0);
                }
            }
            calls.get() < repeats
        })
        .unwrap();
}
fn platform(fields: &[&str]) -> Result<String, Box<dyn Error>> {
    let ua = decode_hex(fields[3])?;
    let os = decode_hex(fields[4])?;
    let navigator = (fields[2] == "1").then_some(Navigator {
        user_agent: &ua,
        platform: &os,
    });
    let Ok(p) = Platform::detect(fields[1] == "1", navigator) else {
        return Ok("error".into());
    };
    let flags = [
        p.is_node,
        p.is_firefox,
        p.is_chrome,
        p.is_legacy_edge,
        p.is_safari,
        p.is_mac,
        p.is_windows,
        p.is_linux,
        p.is_chrome_os,
    ];
    Ok(format!(
        "{}:{}",
        flags.map(|v| if v { "1" } else { "0" }).join(""),
        ryu_js::Buffer::new().format(p.safari_version)
    ))
}
fn windows(fields: &[&str]) -> Result<String, Box<dyn Error>> {
    let cols = fields[1].parse()?;
    let ybase = fields[2].parse()?;
    let y = fields[3].parse()?;
    let ends: Vec<_> = fields[4].split(',').collect();
    if fields[5].len() != ends.len() {
        return Err("invalid wrapped flags".into());
    }
    let lines = CircularList::new(ends.len())?;
    for (i, end) in ends.iter().enumerate() {
        if *end == "-" {
            lines.push(Rc::new(RefCell::new(BufferLine::new(0, None, false))))?;
            lines.set(i, None)?;
            continue;
        }
        let mut line = BufferLine::new(10, None, fields[5].as_bytes()[i] == b'1');
        let mut codes = end.split('+');
        if let Some(first) = codes.next() {
            line.set_cell_from_codepoint(9, first.parse()?, 1, &AttributeData::default());
        }
        for code in codes {
            line.add_codepoint_to_cell(9, code.parse()?, 0);
        }
        lines.push(Rc::new(RefCell::new(line)))?;
    }
    update_windows_mode_wrapped_state(&lines, ybase, y, cols);
    Ok((0..ends.len())
        .map(|i| {
            lines
                .get(i)
                .map_or("-", |line| if line.borrow().is_wrapped { "1" } else { "0" })
        })
        .collect::<String>())
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut host = Host::default();
    let mut queue = None;
    for line in io::stdin().lock().lines() {
        let line = line?;
        let f: Vec<_> = line.split('\t').collect();
        let required = match f[0] {
            "P" | "E" | "S" => 5,
            "W" => 6,
            "I" | "D" => 2,
            "F" | "C" | "Y" => 1,
            _ => return Err("unknown command".into()),
        };
        if f.len() != required {
            return Err("invalid command fields".into());
        }
        let output = match f[0] {
            "P" => Some(platform(&f)?),
            "W" => Some(windows(&f)?),
            "I" => {
                if let Some(old) = queue.take() {
                    let old: TaskQueue<Host> = old;
                    old.clear();
                }
                host = Host::default();
                queue = Some(TaskQueue::idle(host.clone(), f[1] == "1"));
                None
            }
            command => {
                let q = queue.as_ref().ok_or("queue not initialized")?;
                match command {
                    "E" | "S" => {
                        if command == "S" {
                            q.clear();
                        }
                        task(
                            q,
                            &host,
                            f[1].parse()?,
                            f[2].parse()?,
                            f[3].parse()?,
                            f[4].parse()?,
                        );
                    }
                    "F" => q.flush().unwrap(),
                    "C" => q.clear(),
                    "Y" => q.process_priority().unwrap(),
                    "D" => {
                        let values = f[1]
                            .split(',')
                            .map(str::parse::<f64>)
                            .collect::<Result<Vec<_>, _>>()?;
                        let mut index = 0;
                        q.process(|| {
                            let value = values[index.min(values.len() - 1)];
                            index += 1;
                            value
                        })
                        .unwrap();
                    }
                    _ => return Err("unknown command".into()),
                }
                None
            }
        };
        println!(
            "{}",
            output.unwrap_or_else(|| host
                .events
                .borrow_mut()
                .drain(..)
                .collect::<Vec<_>>()
                .join("|"))
        );
    }
    Ok(())
}
