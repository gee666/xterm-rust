// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::RefCell,
    fmt,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};
use xterm_core::{
    event::EmitError,
    headless::{NativeWritePump, QueuedTerminal, Terminal},
    input::write_buffer::WriteData,
    lifecycle::DisposableHandle,
    parser::handler::{DispatchError, HandlerResult},
};
type Log = Rc<RefCell<Vec<String>>>;
#[derive(Default)]
struct Gate {
    result: RefCell<Option<Result<bool, DispatchError>>>,
    waker: RefCell<Option<Waker>>,
}
struct Wait(Rc<Gate>);
impl Future for Wait {
    type Output = Result<bool, DispatchError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(value) = self.0.result.borrow_mut().take() {
            Poll::Ready(value)
        } else {
            *self.0.waker.borrow_mut() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
impl Gate {
    fn resolve(&self, value: Result<bool, DispatchError>) {
        *self.result.borrow_mut() = Some(value);
        if let Some(w) = self.waker.borrow_mut().take() {
            w.wake();
        }
    }
}
#[derive(Debug)]
struct Rejection;
impl fmt::Display for Rejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("retained rejection")
    }
}
impl std::error::Error for Rejection {}
fn hex(units: &[u16]) -> String {
    if units.is_empty() {
        "-".into()
    } else {
        units
            .iter()
            .map(|n| format!("{n:x}"))
            .collect::<Vec<_>>()
            .join(",")
    }
}
struct Scenario {
    host: QueuedTerminal,
    gate: Rc<Gate>,
    handler: DisposableHandle,
    log: Log,
    error: DispatchError,
}
impl Scenario {
    fn new() -> Self {
        let host = QueuedTerminal::with_pump(
            Rc::new(Terminal::new(80, 30, 1000).unwrap()),
            NativeWritePump::with_clock(|| 100.0),
        )
        .unwrap();
        let log = Log::default();
        let output = log.clone();
        host.terminal()
            .on_write_parsed()
            .subscribe(move |_| output.borrow_mut().push("parsed".into()));
        let output = log.clone();
        host.terminal()
            .with_parser(|parser| {
                parser.register_osc_handler(1000.0, move |data| {
                    output.borrow_mut().push(format!("older:{}", hex(&data)));
                    HandlerResult::Ready(true)
                })
            })
            .unwrap()
            .unwrap();
        let gate = Rc::new(Gate::default());
        let wait = gate.clone();
        let output = log.clone();
        let handler = host
            .terminal()
            .with_parser(|parser| {
                parser.register_osc_handler(1000.0, move |data| {
                    output.borrow_mut().push(format!("handler:{}", hex(&data)));
                    HandlerResult::PendingFallible(Box::pin(Wait(wait.clone())))
                })
            })
            .unwrap()
            .unwrap();
        Self {
            host,
            gate,
            handler,
            log,
            error: DispatchError::Callback(EmitError::callback(Rejection)),
        }
    }
    fn start(&self) {
        let output = self.log.clone();
        self.host
            .write(
                WriteData::text("A\x1b]1000;some data\x07B"),
                Some(Rc::new(move |_| {
                    output.borrow_mut().push("callback:a".into());
                    Ok(())
                })),
            )
            .unwrap();
        let output = self.log.clone();
        self.host
            .write(
                WriteData::text("C"),
                Some(Rc::new(move |_| {
                    output.borrow_mut().push("callback:b".into());
                    Ok(())
                })),
            )
            .unwrap();
        self.host.pump().poll().unwrap();
    }
    fn control(&self, name: &str) {
        match name {
            "reset" => {
                self.host.terminal().reset().unwrap();
                self.log.borrow_mut().push("control:reset".into());
            }
            "dispose" => {
                self.host.terminal().dispose().unwrap();
                self.log.borrow_mut().push("control:dispose".into());
            }
            "remove" => {
                self.handler.dispose();
                self.log.borrow_mut().push("control:remove".into());
            }
            _ => (),
        }
    }
    fn complete(&self, name: &str) {
        self.gate.resolve(if name == "rejected" {
            Err(self.error.clone())
        } else {
            Ok(name != "bubble")
        });
        for _ in 0..32 {
            if self.host.pump().pending_tasks() == 0 {
                return;
            }
            if let Err(errors) = self.host.pump().poll() {
                assert_eq!(name, "rejected");
                for error in errors {
                    assert_eq!(error.external_ref::<DispatchError>(), Some(&self.error));
                    self.log.borrow_mut().push("error".into());
                }
            }
        }
        panic!("bounded source trace did not complete");
    }
    fn snapshot(&self) -> Vec<String> {
        let queue = self.host.queue();
        let text = if queue.is_disposed() {
            "-".into()
        } else {
            let row = self
                .host
                .terminal()
                .buffer()
                .unwrap()
                .active()
                .unwrap()
                .get_line(0)
                .unwrap()
                .unwrap();
            let text = row.borrow().translate_to_string(true, None, None, None);
            hex(&text)
        };
        vec![
            text,
            queue.pending_units().to_string(),
            queue.buffer_offset().to_string(),
            queue.buffered_chunks().to_string(),
            u8::from(queue.is_disposed()).to_string(),
            if self.log.borrow().is_empty() {
                "-".into()
            } else {
                self.log.borrow().join("|")
            },
        ]
    }
}
#[test]
fn real_source_async_pause_lifecycle_traces_match_native() {
    let data = include_str!("fixtures/input-handler-async.tsv");
    assert_eq!(data.lines().count(), 18);
    let mut scenario = Scenario::new();
    for line in data.lines() {
        let f: Vec<_> = line.split('\t').collect();
        assert_eq!(f.len(), 8);
        match f[1] {
            "paused" => {
                scenario = Scenario::new();
                scenario.start();
            }
            "controlled" => scenario.control(f[0]),
            "complete" => scenario.complete(f[0]),
            phase => panic!("unknown phase {phase}"),
        };
        assert_eq!(scenario.snapshot(), f[2..], "{} {}", f[0], f[1]);
    }
}
