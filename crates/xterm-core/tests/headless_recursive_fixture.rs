// Source fixture generated from unmodified headless Terminal and WriteBuffer.
use std::{
    cell::{Cell, RefCell},
    error::Error,
    fmt,
    rc::Rc,
};
use xterm_core::{
    event::EmitError,
    headless::{NativeWritePump, QueuedTerminal, Terminal},
    input::write_buffer::{WriteCallback, WriteData, WriteError},
};

type Log = Rc<RefCell<Vec<String>>>;
#[derive(Debug)]
struct ListenerFailure;
impl fmt::Display for ListenerFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("nested listener failure")
    }
}
impl Error for ListenerFailure {}

fn hex(text: &[u16]) -> String {
    if text.is_empty() {
        return "-".into();
    }
    text.iter()
        .map(|u| format!("{u:x}"))
        .collect::<Vec<_>>()
        .join(",")
}
fn callback(
    log: &Log,
    label: &'static str,
) -> Option<WriteCallback<xterm_core::headless::NativeWriteScheduler>> {
    let log = Rc::clone(log);
    Some(Rc::new(move |_| {
        log.borrow_mut().push(label.into());
        Ok(())
    }))
}
fn error_text(result: Result<(), WriteError>) -> String {
    let Err(error) = result else {
        return "-".into();
    };
    let mut cause: &dyn Error = &error;
    loop {
        if cause.is::<ListenerFailure>() {
            return cause.to_string();
        }
        cause = cause
            .source()
            .unwrap_or_else(|| panic!("unexpected error: {error:?}"));
    }
}
fn run_delivery(host: &QueuedTerminal, delivery: &str, log: &Log) -> Result<(), WriteError> {
    match delivery {
        "sync" => host.queue().flush_sync(),
        "resize" => host.terminal().resize(6, 3).map_err(WriteError::external),
        "async" | "same" => {
            if delivery == "same" {
                host.terminal().resize(8, 3).map_err(WriteError::external)?;
                log.borrow_mut()
                    .push(format!("before:{}", host.queue().pending_units()));
            }
            host.pump().poll().map(|_| ()).map_err(|mut errors| {
                assert_eq!(errors.len(), 1);
                errors.pop().unwrap()
            })
        }
        other => panic!("unrecognized delivery {other}"),
    }
}
fn snapshot(host: &QueuedTerminal, log: &Log, error: String) -> Vec<String> {
    let term = host.terminal();
    let buffer = term.buffer().unwrap().active().unwrap();
    let lines = (0..buffer.len().unwrap())
        .map(|i| {
            let row = buffer.get_line(i).unwrap().unwrap();
            let text = row.borrow().translate_to_string(false, None, None, None);
            hex(&text)
        })
        .collect::<Vec<_>>()
        .join(";");
    vec![
        term.cols().unwrap().to_string(),
        term.rows().unwrap().to_string(),
        buffer.cursor_x().unwrap().to_string(),
        buffer.cursor_y().unwrap().to_string(),
        buffer.base_y().unwrap().to_string(),
        buffer.viewport_y().unwrap().to_string(),
        u8::from(term.is_disposed().unwrap()).to_string(),
        host.queue().pending_units().to_string(),
        host.queue().buffer_offset().to_string(),
        host.queue().buffered_chunks().to_string(),
        lines,
        if log.borrow().is_empty() {
            "-".into()
        } else {
            log.borrow().join("|")
        },
        error,
    ]
}

#[test]
fn source_recursive_entry_matches_native_cells_events_queue_and_errors() {
    let fixture = include_str!("fixtures/headless-async-reentry.tsv");
    assert_eq!(fixture.lines().count(), 16);
    for row in fixture.lines() {
        let f: Vec<_> = row.split('\t').collect();
        assert_eq!(f.len(), 18);
        let term = Rc::new(Terminal::new(8, 3, 10).unwrap());
        let host =
            QueuedTerminal::with_pump(Rc::clone(&term), NativeWritePump::with_clock(|| 100.0))
                .unwrap();
        let log = Log::default();
        let output = Rc::clone(&log);
        term.on_resize().subscribe(move |e| {
            output
                .borrow_mut()
                .push(format!("resize:{},{}", e.cols, e.rows))
        });
        let output = Rc::clone(&log);
        term.on_data()
            .subscribe(move |data| output.borrow_mut().push(format!("data:{}", hex(data))));
        let output = Rc::clone(&log);
        let target = Rc::downgrade(&term);
        let queue = host.queue().downgrade();
        let mutation = f[3].to_owned();
        let bells = Cell::new(0);
        term.on_bell().try_subscribe(move |_| {
            let term = target.upgrade().unwrap();
            output.borrow_mut().push("bell".into());
            bells.set(bells.get() + 1);
            match mutation.as_str() {
                "resize" => term.resize(6, 3).map_err(EmitError::callback)?,
                "input" => term
                    .input(&[0x69, 0x6e], true)
                    .map_err(EmitError::callback)?,
                "reset" => term.reset().map_err(EmitError::callback)?,
                "dispose" => term.dispose().map_err(EmitError::callback)?,
                "nested-reset" | "nested-dispose" | "error" => {
                    if bells.get() == 1 {
                        term.resize(6, 3).map_err(EmitError::callback)?;
                    } else if mutation == "nested-reset" {
                        term.reset().map_err(EmitError::callback)?;
                    } else if mutation == "nested-dispose" {
                        term.dispose().map_err(EmitError::callback)?;
                    } else {
                        return Err(EmitError::callback(ListenerFailure));
                    }
                }
                "append" if bells.get() == 1 => {
                    queue
                        .upgrade()
                        .unwrap()
                        .write(WriteData::text("C"), callback(&output, "inner-callback"))
                        .map_err(EmitError::callback)?;
                    term.resize(6, 3).map_err(EmitError::callback)?;
                }
                "append" | "observe" => {}
                other => panic!("unrecognized mutation {other}"),
            }
            Ok(())
        });
        let text = if f[1] == "unicode" {
            "界\x07\u{301}Z"
        } else {
            "A\x07B"
        };
        let data = if f[1] == "bytes" {
            WriteData::Bytes(text.as_bytes().to_vec())
        } else {
            WriteData::text(text)
        };
        host.write(data, callback(&log, "callback")).unwrap();
        let tail: usize = f[4].parse().unwrap();
        if tail != 0 {
            host.write(
                WriteData::text(&"C".repeat(tail)),
                callback(&log, "tail-callback"),
            )
            .unwrap();
        }
        let error = error_text(run_delivery(&host, f[2], &log));
        assert_eq!(snapshot(&host, &log, error), f[5..], "{}", f[0]);
        term.dispose().unwrap();
    }
}

#[test]
fn one_native_queue_per_terminal_and_drop_unhooks_resize() {
    let term = Rc::new(Terminal::new(8, 3, 10).unwrap());
    let first = QueuedTerminal::new(Rc::clone(&term)).unwrap();
    assert!(matches!(
        QueuedTerminal::new(Rc::clone(&term)),
        Err(xterm_core::headless::HeadlessError::QueueAlreadyAttached)
    ));
    drop(first);
    let replacement = QueuedTerminal::new(Rc::clone(&term)).unwrap();
    replacement.write(WriteData::text("queued"), None).unwrap();
    term.resize(6, 3).unwrap();
    assert_eq!(replacement.queue().buffered_chunks(), 0);
}
