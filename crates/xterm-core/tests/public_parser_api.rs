use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};
use xterm_core::{
    parser::{
        escape::{EscapeSequenceParser, FunctionIdentifier},
        handler::HandlerResult,
        params::ParamValue,
    },
    public_api::ParserApi,
};

fn input(value: &str) -> Vec<u32> {
    value.chars().map(u32::from).collect()
}
fn parse(parser: &mut EscapeSequenceParser, data: &[u32]) {
    assert!(parser.parse(data, None).unwrap().is_none());
}

#[test]
fn csi_gets_owned_nested_parameter_snapshots_and_removable_alias() {
    let mut parser = EscapeSequenceParser::new();
    let observed = Rc::new(RefCell::new(Vec::new()));
    let captured = observed.clone();
    let handle = ParserApi::new(&mut parser)
        .add_csi_handler(FunctionIdentifier::new("m"), move |params| {
            captured.borrow_mut().push(params);
            HandlerResult::Ready(true)
        })
        .unwrap();
    parse(&mut parser, &input("\x1b[1;38:2::10:20:30m\x1b[0m"));
    assert_eq!(
        *observed.borrow(),
        vec![
            vec![
                ParamValue::Param(1),
                ParamValue::Param(38),
                ParamValue::SubParams(vec![2, -1, 10, 20, 30])
            ],
            vec![ParamValue::Param(0)]
        ]
    );
    handle.dispose();
    handle.dispose();
    parse(&mut parser, &input("\x1b[5m"));
    assert_eq!(observed.borrow().len(), 2);
}

#[test]
fn all_five_public_handler_families_bubble_newest_first_and_stop_on_true() {
    for family in 0..5 {
        let mut parser = EscapeSequenceParser::new();
        let calls = Rc::new(RefCell::new(Vec::new()));
        {
            let mut api = ParserApi::new(&mut parser);
            for label in ["a", "b", "c"] {
                let captured = calls.clone();
                let callback = move || {
                    captured.borrow_mut().push(label);
                    HandlerResult::Ready(label == "b")
                };
                match family {
                    0 => {
                        api.register_csi_handler(FunctionIdentifier::new("Z"), move |_| callback())
                            .unwrap();
                    }
                    1 => {
                        api.add_dcs_handler(FunctionIdentifier::new("Z"), move |data, params| {
                            assert_eq!(data, "payload".encode_utf16().collect::<Vec<_>>());
                            assert_eq!(params, vec![ParamValue::Param(1), ParamValue::Param(2)]);
                            callback()
                        })
                        .unwrap();
                    }
                    2 => {
                        api.add_esc_handler(FunctionIdentifier::new("Z"), callback)
                            .unwrap();
                    }
                    3 => {
                        api.add_osc_handler(1234.0, move |data| {
                            assert_eq!(data, "payload".encode_utf16().collect::<Vec<_>>());
                            callback()
                        })
                        .unwrap();
                    }
                    4 => {
                        api.register_apc_handler(FunctionIdentifier::new("Z"), move |data| {
                            assert_eq!(data, "payload".encode_utf16().collect::<Vec<_>>());
                            callback()
                        })
                        .unwrap();
                    }
                    _ => unreachable!(),
                }
            }
        }
        let sequence = [
            "\x1b[Z",
            "\x1bP1;2Zpayload\x1b\\",
            "\x1bZ",
            "\x1b]1234;payload\x07",
            "\x1b_Zpayload\x1b\\",
        ][family];
        parse(&mut parser, &input(sequence));
        assert_eq!(*calls.borrow(), ["c", "b"], "family {family}");
    }
}

struct NoWake;
impl Wake for NoWake {
    fn wake(self: Arc<Self>) {}
}

#[test]
fn pending_public_handlers_pause_then_resume_before_following_print() {
    for family in 0..5 {
        let mut parser = EscapeSequenceParser::new();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let printed = Rc::new(RefCell::new(Vec::new()));
        let captured = printed.clone();
        parser.set_print_handler(move |data| captured.borrow_mut().extend_from_slice(data));
        {
            let mut api = ParserApi::new(&mut parser);
            for label in ["a", "b", "c"] {
                let captured = calls.clone();
                let callback = move || {
                    captured.borrow_mut().push(label);
                    if label == "b" {
                        HandlerResult::Pending(Box::pin(async { false }))
                    } else {
                        HandlerResult::Ready(false)
                    }
                };
                match family {
                    0 => {
                        api.register_csi_handler(FunctionIdentifier::new("Z"), move |_| callback())
                            .unwrap();
                    }
                    1 => {
                        api.register_dcs_handler(FunctionIdentifier::new("Z"), move |_, _| {
                            callback()
                        })
                        .unwrap();
                    }
                    2 => {
                        api.register_esc_handler(FunctionIdentifier::new("Z"), callback)
                            .unwrap();
                    }
                    3 => {
                        api.register_osc_handler(666.0, move |_| callback())
                            .unwrap();
                    }
                    4 => {
                        api.register_apc_handler(FunctionIdentifier::new("Z"), move |_| callback())
                            .unwrap();
                    }
                    _ => unreachable!(),
                }
            }
        }
        let data = input(
            [
                "\x1b[Zafter",
                "\x1bPZdata\x1b\\after",
                "\x1bZafter",
                "\x1b]666;data\x07after",
                "\x1b_Zdata\x1b\\after",
            ][family],
        );
        let Some(HandlerResult::Pending(mut pending)) = parser.parse(&data, None).unwrap() else {
            panic!("expected pending family {family}");
        };
        assert_eq!(*calls.borrow(), ["c", "b"]);
        assert!(printed.borrow().is_empty());
        let waker = Waker::from(Arc::new(NoWake));
        assert_eq!(
            pending.as_mut().poll(&mut Context::from_waker(&waker)),
            Poll::Ready(false)
        );
        assert!(parser.parse(&data, Some(false)).unwrap().is_none());
        assert_eq!(*calls.borrow(), ["c", "b", "a"]);
        assert_eq!(*printed.borrow(), input("after"));
    }
}

#[test]
fn apc_payload_is_utf16_without_identifier_and_survives_all_chunk_splits() {
    let data = [0x1b, 0x5f, 0x5a, 0x61, 0x1f600, 0xd800, 0x1b, 0x5c];
    for split in 0..=data.len() {
        let mut parser = EscapeSequenceParser::new();
        let payloads = Rc::new(RefCell::new(Vec::new()));
        let captured = payloads.clone();
        ParserApi::new(&mut parser)
            .register_apc_handler(FunctionIdentifier::new("Z"), move |data| {
                captured.borrow_mut().push(data);
                HandlerResult::Ready(true)
            })
            .unwrap();
        parse(&mut parser, &data[..split]);
        parse(&mut parser, &data[split..]);
        assert_eq!(
            *payloads.borrow(),
            vec![vec![0x61, 0xd83d, 0xde00, 0xd800]],
            "split {split}"
        );
    }
}

#[test]
fn invalid_identifiers_are_rejected_without_registration() {
    let mut parser = EscapeSequenceParser::new();
    for final_byte in ["", "ab", "\u{80}", "?"] {
        assert!(ParserApi::new(&mut parser)
            .register_csi_handler(FunctionIdentifier::new(final_byte), |_| {
                HandlerResult::Ready(true)
            })
            .is_err());
    }
}
