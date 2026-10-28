// Copyright (c) 2018 The xterm.js authors. All rights reserved. Licensed under MIT.
use super::*;
use xterm_core::parser::{dcs::DcsHandler, osc::OscHandler};

#[test]
fn original_prefix_intermediate_and_final_limits() {
    for byte in 0x3c..=0x3f {
        let value = char::from(byte).to_string();
        let id = FunctionIdentifier {
            prefix: Some(&value),
            intermediates: None,
            final_byte: "z",
        }
        .packed(false)
        .unwrap();
        assert_eq!(EscapeSequenceParser::ident_to_string(id), value + "z");
    }
    for (value, message) in [
        (";", "prefix must be in range 0x3c .. 0x3f"),
        ("@", "prefix must be in range 0x3c .. 0x3f"),
        ("??", "only one byte as prefix supported"),
    ] {
        let error = FunctionIdentifier {
            prefix: Some(value),
            intermediates: None,
            final_byte: "z",
        }
        .packed(false)
        .unwrap_err();
        assert_eq!(error.to_string(), message);
    }
    for byte in 0x20..=0x2f {
        let value = char::from(byte).to_string().repeat(2);
        let id = FunctionIdentifier {
            prefix: None,
            intermediates: Some(&value),
            final_byte: "z",
        }
        .packed(false)
        .unwrap();
        assert_eq!(EscapeSequenceParser::ident_to_string(id), value + "z");
    }
    for (value, message) in [
        ("\u{1f}", "intermediate must be in range 0x20 .. 0x2f"),
        ("0", "intermediate must be in range 0x20 .. 0x2f"),
        ("!!!", "only two bytes as intermediates are supported"),
    ] {
        let error = FunctionIdentifier {
            prefix: None,
            intermediates: Some(value),
            final_byte: "z",
        }
        .packed(false)
        .unwrap_err();
        assert_eq!(error.to_string(), message);
    }
    for byte in 0x40..=0x7e {
        let value = char::from(byte).to_string();
        assert_eq!(
            EscapeSequenceParser::ident_to_string(
                FunctionIdentifier::new(&value).packed(false).unwrap()
            ),
            value
        );
    }
    for (value, message) in [
        ("?", "final must be in range 64 .. 126"),
        ("\u{7f}", "final must be in range 64 .. 126"),
        ("zz", "final must be a single byte"),
    ] {
        assert_eq!(
            FunctionIdentifier::new(value)
                .packed(false)
                .unwrap_err()
                .to_string(),
            message
        );
    }
    for byte in 0x30..=0x7e {
        let value = char::from(byte).to_string();
        let id = FunctionIdentifier::new(&value);
        let mut parser = EscapeSequenceParser::new();
        let esc = parser
            .register_esc_handler(
                id,
                Rc::new(RefCell::new(|_: &Params| HandlerResult::Ready(true))),
            )
            .unwrap();
        esc.dispose();
        let apc = parser
            .register_apc_handler(
                id,
                Rc::new(RefCell::new(OscHandler::new(|_: Vec<u16>| {
                    HandlerResult::Ready(true)
                }))),
            )
            .unwrap();
        apc.dispose();
    }
    for value in ["/", "\u{7f}"] {
        assert_eq!(
            FunctionIdentifier::new(value)
                .packed(true)
                .unwrap_err()
                .to_string(),
            "final must be in range 48 .. 126"
        );
        let mut parser = EscapeSequenceParser::new();
        let id = FunctionIdentifier::new(value);
        let esc = parser.register_esc_handler(
            id,
            Rc::new(RefCell::new(|_: &Params| HandlerResult::Ready(true))),
        );
        assert_eq!(
            esc.err().unwrap().to_string(),
            "final must be in range 48 .. 126"
        );
        let apc = parser.register_apc_handler(
            id,
            Rc::new(RefCell::new(OscHandler::new(|_: Vec<u16>| {
                HandlerResult::Ready(true)
            }))),
        );
        assert_eq!(
            apc.err().unwrap().to_string(),
            "final must be in range 48 .. 126"
        );
    }
}

#[test]
fn original_identifier_packing_order() {
    for (prefix, intermediates, expected) in [
        (None, None, "z"),
        (Some("?"), None, "?z"),
        (None, Some("!"), "!z"),
        (Some("?"), Some("!"), "?!z"),
        (Some("?"), Some("!!"), "?!!z"),
    ] {
        let id = FunctionIdentifier {
            prefix,
            intermediates,
            final_byte: "z",
        }
        .packed(false)
        .unwrap();
        assert_eq!(EscapeSequenceParser::ident_to_string(id), expected);
    }
}

#[test]
fn original_identifier_invocations_all_four_families() {
    for family in ["ESC", "CSI", "DCS", "APC"] {
        let mut parser = EscapeSequenceParser::new();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut tokens = Vec::new();
        let mut input = String::new();
        let mut expected = Vec::new();
        for prefix in [None, Some("?")] {
            if prefix.is_some() && matches!(family, "ESC" | "APC") {
                continue;
            }
            for intermediates in [None, Some("!"), Some("!!")] {
                let id = FunctionIdentifier {
                    prefix,
                    intermediates,
                    final_byte: "z",
                };
                let label = format!("{}{}z", prefix.unwrap_or(""), intermediates.unwrap_or(""));
                let output_label = label.clone();
                let out = Rc::clone(&calls);
                let token = match family {
                    "ESC" | "CSI" => {
                        let is_csi = family == "CSI";
                        let callback = Rc::new(RefCell::new(move |params: &Params| {
                            out.borrow_mut().push((
                                output_label.clone(),
                                if is_csi { params.to_array() } else { vec![] },
                                String::new(),
                            ));
                            HandlerResult::Ready(true)
                        }));
                        if is_csi {
                            parser.register_csi_handler(id, callback).unwrap()
                        } else {
                            parser.register_esc_handler(id, callback).unwrap()
                        }
                    }
                    "DCS" => parser
                        .register_dcs_handler(
                            id,
                            Rc::new(RefCell::new(DcsHandler::new(move |payload, params| {
                                out.borrow_mut().push((
                                    output_label.clone(),
                                    params.to_array(),
                                    text(payload),
                                ));
                                HandlerResult::Ready(true)
                            }))),
                        )
                        .unwrap(),
                    "APC" => parser
                        .register_apc_handler(
                            id,
                            Rc::new(RefCell::new(OscHandler::new(move |payload: Vec<u16>| {
                                out.borrow_mut().push((
                                    output_label.clone(),
                                    vec![],
                                    text(&payload),
                                ));
                                HandlerResult::Ready(true)
                            }))),
                        )
                        .unwrap(),
                    _ => unreachable!(),
                };
                tokens.push(token);
                let params = if matches!(family, "CSI" | "DCS") {
                    vec![V::Param(1), V::Param(0)]
                } else {
                    vec![]
                };
                expected.push((
                    label,
                    params,
                    if matches!(family, "DCS" | "APC") {
                        "AB".into()
                    } else {
                        String::new()
                    },
                ));
                input.push_str(match family {
                    "ESC" => "\x1b",
                    "CSI" => "\x1b[",
                    "DCS" => "\x1bP",
                    _ => "\x1b_",
                });
                input.push_str(prefix.unwrap_or(""));
                if matches!(family, "CSI" | "DCS") {
                    input.push_str("1;");
                }
                input.push_str(intermediates.unwrap_or(""));
                input.push('z');
                if matches!(family, "DCS" | "APC") {
                    input.push_str("AB\x1b\\");
                }
            }
        }
        assert!(parser.parse(&data(&input), None).unwrap().is_none());
        assert_eq!(*calls.borrow(), expected, "{family}");
        for token in tokens {
            token.dispose();
        }
        assert!(parser.parse(&data(&input), None).unwrap().is_none());
        assert_eq!(*calls.borrow(), expected, "{family} after disposal");
    }
}
