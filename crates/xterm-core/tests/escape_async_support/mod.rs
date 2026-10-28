// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Helpers for faithful EscapeSequenceParser async assertions.
use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};
use xterm_core::{
    lifecycle::DisposableHandle,
    parser::{
        apc::ApcHandler,
        dcs::DcsHandler,
        escape::{
            CommandHandler, EscapeSequenceParser, FunctionIdentifier, HandlerKind, PauseInfo,
        },
        handler::HandlerResult,
        osc::OscHandler,
        params::Params,
    },
};

pub type Log = Rc<RefCell<Vec<String>>>;
pub const BASE_PAUSES: [(usize, HandlerKind, usize); 7] = [
    (6, HandlerKind::Csi, 0),
    (15, HandlerKind::Esc, 0),
    (20, HandlerKind::Esc, 0),
    (27, HandlerKind::Csi, 0),
    (41, HandlerKind::Dcs, 0),
    (54, HandlerKind::Osc, 0),
    (62, HandlerKind::Apc, 0),
];
pub fn assert_pauses(actual: &[PauseInfo], expected: &[(usize, HandlerKind, usize)]) {
    assert_eq!(
        actual
            .iter()
            .map(|p| (p.chunk_position, p.kind, p.handler_position))
            .collect::<Vec<_>>(),
        expected
    );
}
pub const INPUT: &str = "\x1b[1;31mhello \x1b%Gwor\x1bEld!\x1b[0m\r\n$>\x1bP1;2axyz\x1b\\\x1b]1;foo=bar\x1b\\\x1b_Xabc\x1b\\FIN";
pub const EXPECTED: [&str; 14] = [
    "SGR:[1, 31]",
    "PRINT:hello ",
    "ESC:%G",
    "PRINT:wor",
    "ESC:E",
    "PRINT:ld!",
    "SGR:[0]",
    "EXE:\r",
    "EXE:\n",
    "PRINT:$>",
    "DCS:xyz:[1, 2]",
    "OSC:foo=bar",
    "APC:abc",
    "PRINT:FIN",
];
pub fn data(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}
pub fn result(asynchronous: bool, value: bool) -> HandlerResult {
    if asynchronous {
        HandlerResult::Pending(Box::pin(async move { value }))
    } else {
        HandlerResult::Ready(value)
    }
}
struct Noop;
impl Wake for Noop {
    fn wake(self: Arc<Self>) {}
}
pub fn resolve(result: HandlerResult) -> bool {
    let mut future = Box::pin(result.resolve());
    let waker = Waker::from(Arc::new(Noop));
    match std::future::Future::poll(future.as_mut(), &mut Context::from_waker(&waker)) {
        Poll::Ready(value) => value.expect("unexpected callback failure"),
        Poll::Pending => panic!("test future unexpectedly pending"),
    }
}
pub fn drive(parser: &mut EscapeSequenceParser, chunk: &[u32]) -> Vec<PauseInfo> {
    let mut pauses = Vec::new();
    let mut previous = None;
    while let Some(pending) = parser.parse(chunk, previous).unwrap() {
        pauses.push(parser.pause_info().unwrap());
        previous = Some(resolve(pending));
    }
    pauses
}
pub fn setup(asynchronous: bool) -> (EscapeSequenceParser, Log) {
    let mut parser = EscapeSequenceParser::new();
    let log: Log = Rc::default();
    let target = Rc::clone(&log);
    parser.set_print_handler(move |chunk| {
        target.borrow_mut().push(format!(
            "PRINT:{}",
            chunk
                .iter()
                .copied()
                .map(|v| char::from_u32(v).unwrap())
                .collect::<String>()
        ))
    });
    let target = Rc::clone(&log);
    parser
        .register_csi_handler(
            FunctionIdentifier::new("m"),
            Rc::new(RefCell::new(move |params: &Params| {
                target
                    .borrow_mut()
                    .push(format!("SGR:{:?}", params.values()));
                result(asynchronous, true)
            })),
        )
        .unwrap();
    for (final_byte, intermediate) in [("G", Some("%")), ("E", None)] {
        let target = Rc::clone(&log);
        parser
            .register_esc_handler(
                FunctionIdentifier {
                    prefix: None,
                    intermediates: intermediate,
                    final_byte,
                },
                Rc::new(RefCell::new(move |_: &Params| {
                    target
                        .borrow_mut()
                        .push(format!("ESC:{}{final_byte}", intermediate.unwrap_or("")));
                    result(asynchronous, true)
                })),
            )
            .unwrap();
    }
    for code in [b'\r', b'\n'] {
        let target = Rc::clone(&log);
        parser.set_execute_handler(u32::from(code), move |_| {
            target
                .borrow_mut()
                .push(format!("EXE:{}", char::from(code)));
            true
        });
    }
    let target = Rc::clone(&log);
    parser.osc.register_handler(
        1.0,
        Rc::new(RefCell::new(OscHandler::new(move |payload| {
            target
                .borrow_mut()
                .push(format!("OSC:{}", String::from_utf16(&payload).unwrap()));
            result(asynchronous, true)
        }))),
    );
    let target = Rc::clone(&log);
    parser
        .register_dcs_handler(
            FunctionIdentifier::new("a"),
            Rc::new(RefCell::new(DcsHandler::new(move |payload, params| {
                target.borrow_mut().push(format!(
                    "DCS:{}:{:?}",
                    String::from_utf16(payload).unwrap(),
                    params.values()
                ));
                result(asynchronous, true)
            }))),
        )
        .unwrap();
    let target = Rc::clone(&log);
    parser
        .register_apc_handler(
            FunctionIdentifier::new("X"),
            Rc::new(RefCell::new(ApcHandler::new(move |payload| {
                target
                    .borrow_mut()
                    .push(format!("APC:{}", String::from_utf16(&payload).unwrap()));
                result(asynchronous, true)
            }))),
        )
        .unwrap();
    (parser, log)
}
pub fn extra(
    parser: &mut EscapeSequenceParser,
    kind: HandlerKind,
    log: &Log,
    label: &'static str,
    asynchronous: bool,
    handled: bool,
) -> DisposableHandle {
    let target = Rc::clone(log);
    let callback = move || {
        target.borrow_mut().push(label.to_owned());
        result(asynchronous, handled)
    };
    match kind {
        HandlerKind::Csi | HandlerKind::Esc => {
            let handler: CommandHandler = Rc::new(RefCell::new(move |_: &Params| callback()));
            if kind == HandlerKind::Csi {
                parser
                    .register_csi_handler(FunctionIdentifier::new("m"), handler)
                    .unwrap()
            } else {
                parser
                    .register_esc_handler(FunctionIdentifier::new("E"), handler)
                    .unwrap()
            }
        }
        HandlerKind::Osc => parser.osc.register_handler(
            1.0,
            Rc::new(RefCell::new(OscHandler::new(move |_| callback()))),
        ),
        HandlerKind::Dcs => parser
            .register_dcs_handler(
                FunctionIdentifier::new("a"),
                Rc::new(RefCell::new(DcsHandler::new(move |_, _| callback()))),
            )
            .unwrap(),
        HandlerKind::Apc => parser
            .register_apc_handler(
                FunctionIdentifier::new("X"),
                Rc::new(RefCell::new(ApcHandler::new(move |_| callback()))),
            )
            .unwrap(),
    }
}
