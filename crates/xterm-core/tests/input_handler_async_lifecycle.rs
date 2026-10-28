// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};
use xterm_core::{
    headless::{QueuedTerminal, Terminal},
    input::write_buffer::WriteData,
    parser::handler::HandlerResult,
};
#[derive(Default)]
struct Gate {
    result: Cell<Option<bool>>,
    waker: RefCell<Option<Waker>>,
}
struct Wait(Rc<Gate>);
impl Future for Wait {
    type Output = bool;
    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<bool> {
        if let Some(result) = self.0.result.take() {
            Poll::Ready(result)
        } else {
            *self.0.waker.borrow_mut() = Some(context.waker().clone());
            Poll::Pending
        }
    }
}
impl Gate {
    fn resolve(&self, value: bool) {
        self.result.set(Some(value));
        if let Some(waker) = self.waker.borrow_mut().take() {
            waker.wake();
        }
    }
}
fn host() -> QueuedTerminal {
    QueuedTerminal::new(Rc::new(Terminal::new(80, 30, 1000).unwrap())).unwrap()
}
fn first_line(host: &QueuedTerminal) -> Vec<u16> {
    let row = host
        .terminal()
        .buffer()
        .unwrap()
        .active()
        .unwrap()
        .get_line(0)
        .unwrap()
        .unwrap();
    let text = row.borrow().translate_to_string(true, None, None, None);
    text
}
fn drain(host: &QueuedTerminal) {
    for _ in 0..32 {
        if host.pump().pending_tasks() == 0 {
            return;
        }
        host.pump().poll().unwrap();
    }
    panic!("bounded test continuation did not finish");
}
fn register(host: &QueuedTerminal, gate: &Rc<Gate>, calls: &Rc<Cell<usize>>) {
    let gate = gate.clone();
    let calls = calls.clone();
    host.terminal()
        .with_parser(|parser| {
            parser.register_osc_handler(1000.0, move |data| {
                assert_eq!(data, "some data".encode_utf16().collect::<Vec<_>>());
                calls.set(calls.get() + 1);
                HandlerResult::Pending(Box::pin(Wait(gate.clone())))
            })
        })
        .unwrap();
}
#[test]
fn public_reset_retains_paused_parser_and_never_decodes_prefix_twice() {
    let host = host();
    let gate = Rc::new(Gate::default());
    let calls = Rc::new(Cell::new(0));
    register(&host, &gate, &calls);
    let done = Rc::new(Cell::new(false));
    let output = done.clone();
    host.write(
        WriteData::text("prefix\x1b]1000;some data\x07tail"),
        Some(Rc::new(move |_| {
            output.set(true);
            Ok(())
        })),
    )
    .unwrap();
    host.pump().poll().unwrap();
    assert_eq!(
        first_line(&host),
        "prefix".encode_utf16().collect::<Vec<_>>()
    );
    assert!(!done.get());
    host.terminal().reset().unwrap();
    assert!(first_line(&host).is_empty());
    gate.resolve(true);
    drain(&host);
    assert_eq!(first_line(&host), "tail".encode_utf16().collect::<Vec<_>>());
    assert_eq!(calls.get(), 1);
    assert!(done.get());
}
#[test]
fn terminal_disposal_cancels_actual_paused_handler_without_outer_callback() {
    let host = host();
    let gate = Rc::new(Gate::default());
    let calls = Rc::new(Cell::new(0));
    register(&host, &gate, &calls);
    let done = Rc::new(Cell::new(false));
    let output = done.clone();
    host.write(
        WriteData::text("prefix\x1b]1000;some data\x07tail"),
        Some(Rc::new(move |_| {
            output.set(true);
            Ok(())
        })),
    )
    .unwrap();
    host.pump().poll().unwrap();
    assert_eq!(calls.get(), 1);
    host.terminal().dispose().unwrap();
    assert!(host.queue().is_disposed());
    assert_eq!(host.queue().pending_units(), 0.0);
    gate.resolve(true);
    drain(&host);
    assert_eq!(calls.get(), 1);
    assert!(!done.get());
}
