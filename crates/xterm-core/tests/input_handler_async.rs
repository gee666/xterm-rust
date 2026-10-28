// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Faithful live InputHandler async cases through the actual terminal and queue.
use std::{
    cell::{Cell, RefCell},
    future::poll_fn,
    rc::Rc,
    task::Poll,
};
use xterm_core::{
    headless::{QueuedTerminal, Terminal},
    input::write_buffer::WriteData,
    parser::{escape::FunctionIdentifier, handler::HandlerResult, params::ParamValue},
};
async fn microtask() {
    let yielded = Cell::new(false);
    poll_fn(|context| {
        if yielded.replace(true) {
            Poll::Ready(())
        } else {
            context.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await;
}
fn host() -> QueuedTerminal {
    QueuedTerminal::new(Rc::new(Terminal::new(80, 30, 1000).unwrap())).unwrap()
}
fn lines(terminal: &Terminal) -> Vec<String> {
    let buffer = terminal.buffer().unwrap().active().unwrap();
    (0..2)
        .map(|i| {
            let row = buffer.get_line(i).unwrap().unwrap();
            let text = row.borrow().translate_to_string(true, None, None, None);
            String::from_utf16(&text).unwrap()
        })
        .collect()
}
fn drain(host: &QueuedTerminal) {
    for _ in 0..32 {
        if host.pump().pending_tasks() == 0 {
            return;
        }
        host.pump().poll().unwrap();
    }
    panic!("immediately resolving test handlers exceeded bounded host turns");
}
fn params(values: Vec<ParamValue>) -> Vec<i32> {
    values
        .into_iter()
        .map(|value| match value {
            ParamValue::Param(n) => n,
            ParamValue::SubParams(_) => panic!("source case has no subparameters"),
        })
        .collect()
}
#[test]
fn async_cup_with_cpr_check() {
    let host = host();
    let cup = Rc::new(RefCell::new(Vec::new()));
    let cpr = Rc::new(RefCell::new(Vec::new()));
    let output = cup.clone();
    host.terminal()
        .with_parser(|parser| {
            parser
                .register_csi_handler(FunctionIdentifier::new("H"), move |values| {
                    output.borrow_mut().push(params(values));
                    // The public Rust callback delegates to the same built-in CUP handler
                    // after awaiting, instead of calling a private InputHandler method.
                    HandlerResult::Pending(Box::pin(async {
                        microtask().await;
                        false
                    }))
                })
                .unwrap()
        })
        .unwrap();
    let output = cpr.clone();
    host.terminal().on_data().subscribe(move |data| {
        let text = String::from_utf16(data).unwrap();
        if let Some(reply) = text.strip_prefix("\x1b[").and_then(|s| s.strip_suffix('R')) {
            output.borrow_mut().push(
                reply
                    .split(';')
                    .map(|n| n.parse::<i32>().unwrap())
                    .collect::<Vec<_>>(),
            );
        }
    });
    host.write(
        WriteData::text("aaa\x1b[3;4H\x1b[6nbbb\x1b[6;8H\x1b[6n"),
        None,
    )
    .unwrap();
    drain(&host);
    assert_eq!(*cup.borrow(), *cpr.borrow());
    assert_eq!(*cup.borrow(), [vec![3, 4], vec![6, 8]]);
}
#[test]
fn async_osc_between() {
    let host = host();
    let weak = Rc::downgrade(host.terminal());
    host.terminal()
        .with_parser(|parser| {
            parser.register_osc_handler(1000.0, move |data| {
                let weak = weak.clone();
                HandlerResult::Pending(Box::pin(async move {
                    microtask().await;
                    let terminal = weak.upgrade().unwrap();
                    assert_eq!(lines(&terminal), ["hello world!", ""]);
                    assert_eq!(data, "some data".encode_utf16().collect::<Vec<_>>());
                    true
                }))
            })
        })
        .unwrap();
    host.write(
        WriteData::text("hello world!\r\n\x1b]1000;some data\x07second line"),
        None,
    )
    .unwrap();
    drain(&host);
    assert_eq!(lines(host.terminal()), ["hello world!", "second line"]);
}
#[test]
fn async_dcs_between() {
    let host = host();
    let weak = Rc::downgrade(host.terminal());
    host.terminal()
        .with_parser(|parser| {
            parser
                .register_dcs_handler(FunctionIdentifier::new("a"), move |data, values| {
                    let weak = weak.clone();
                    HandlerResult::Pending(Box::pin(async move {
                        microtask().await;
                        let terminal = weak.upgrade().unwrap();
                        assert_eq!(lines(&terminal), ["hello world!", ""]);
                        assert_eq!(data, "some data".encode_utf16().collect::<Vec<_>>());
                        assert_eq!(params(values), [1, 2]);
                        true
                    }))
                })
                .unwrap()
        })
        .unwrap();
    host.write(
        WriteData::text("hello world!\r\n\x1bP1;2asome data\x1b\\second line"),
        None,
    )
    .unwrap();
    drain(&host);
    assert_eq!(lines(host.terminal()), ["hello world!", "second line"]);
}
