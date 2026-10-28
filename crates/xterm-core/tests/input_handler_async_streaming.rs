// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::Cell, future::poll_fn, rc::Rc, task::Poll};
use xterm_core::{
    headless::{QueuedTerminal, Terminal},
    input::write_buffer::{WriteCallback, WriteData},
    parser::{escape::FunctionIdentifier, handler::HandlerResult, params::ParamValue},
};
async fn microtask() {
    let ready = Cell::new(false);
    poll_fn(|cx| {
        if ready.replace(true) {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await;
}
fn drain(host: &QueuedTerminal) {
    for _ in 0..32 {
        if host.pump().pending_tasks() == 0 {
            return;
        }
        host.pump().poll().unwrap();
    }
    panic!("test microtasks did not finish");
}
fn callback(
    count: &Rc<Cell<usize>>,
) -> Option<WriteCallback<xterm_core::headless::NativeWriteScheduler>> {
    let count = count.clone();
    Some(Rc::new(move |_| {
        count.set(count.get() + 1);
        Ok(())
    }))
}
fn run(dcs: bool, bytes: bool) {
    let text = if dcs {
        "文🙂\x1bP1;2asome data\x1b\\e\u{301}"
    } else {
        "文🙂\x1b]1000;some data\x07e\u{301}"
    };
    let length = if bytes {
        text.len()
    } else {
        text.encode_utf16().count()
    };
    for split in 0..=length {
        let host = QueuedTerminal::new(Rc::new(Terminal::new(20, 3, 10).unwrap())).unwrap();
        let calls = Rc::new(Cell::new(0));
        let output = calls.clone();
        host.terminal()
            .with_parser(|parser| {
                if dcs {
                    parser
                        .register_dcs_handler(FunctionIdentifier::new("a"), move |data, params| {
                            assert_eq!(data, "some data".encode_utf16().collect::<Vec<_>>());
                            assert_eq!(params, [ParamValue::Param(1), ParamValue::Param(2)]);
                            output.set(output.get() + 1);
                            HandlerResult::Pending(Box::pin(async {
                                microtask().await;
                                true
                            }))
                        })
                        .unwrap();
                } else {
                    parser
                        .register_osc_handler(1000.0, move |data| {
                            assert_eq!(data, "some data".encode_utf16().collect::<Vec<_>>());
                            output.set(output.get() + 1);
                            HandlerResult::Pending(Box::pin(async {
                                microtask().await;
                                true
                            }))
                        })
                        .unwrap();
                }
            })
            .unwrap();
        let callbacks = Rc::new(Cell::new(0));
        if bytes {
            let data = text.as_bytes();
            host.write(
                WriteData::Bytes(data[..split].to_vec()),
                callback(&callbacks),
            )
            .unwrap();
            host.write(
                WriteData::Bytes(data[split..].to_vec()),
                callback(&callbacks),
            )
            .unwrap();
        } else {
            let data: Vec<_> = text.encode_utf16().collect();
            host.write(
                WriteData::Text(data[..split].to_vec()),
                callback(&callbacks),
            )
            .unwrap();
            host.write(
                WriteData::Text(data[split..].to_vec()),
                callback(&callbacks),
            )
            .unwrap();
        }
        drain(&host);
        let row = host
            .terminal()
            .buffer()
            .unwrap()
            .active()
            .unwrap()
            .get_line(0)
            .unwrap()
            .unwrap();
        assert_eq!(
            row.borrow().translate_to_string(true, None, None, None),
            "文🙂e\u{301}".encode_utf16().collect::<Vec<_>>(),
            "dcs{dcs} bytes{bytes} split{split}"
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(callbacks.get(), 2);
    }
}
#[test]
fn osc_at_every_utf16_boundary() {
    run(false, false);
}
#[test]
fn osc_at_every_utf8_boundary() {
    run(false, true);
}
#[test]
fn dcs_at_every_utf16_boundary() {
    run(true, false);
}
#[test]
fn dcs_at_every_utf8_boundary() {
    run(true, true);
}
