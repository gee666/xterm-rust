// Copyright (c) 2018 The xterm.js authors. All rights reserved. Licensed under MIT.
use super::registrations::{clear, Kind, INPUT};
use super::*;
use xterm_core::parser::{dcs::DcsHandlerTrait, osc::OscHandlerTrait};

#[test]
fn original_print_execute_and_esc_set_clear_handlers() {
    let mut parser = EscapeSequenceParser::new();
    let print = Rc::new(RefCell::new(String::new()));
    let out = Rc::clone(&print);
    parser
        .set_print_handler(move |codes| out.borrow_mut().push_str(&text(&utf32_to_string(codes))));
    parser.parse(&data(INPUT), None).unwrap();
    assert_eq!(*print.borrow(), "hello world!$>");
    parser.clear_print_handler();
    parser.clear_print_handler();
    print.borrow_mut().clear();
    parser.parse(&data(INPUT), None).unwrap();
    assert!(print.borrow().is_empty());
    let exes = Rc::new(RefCell::new(Vec::new()));
    for code in [10, 13] {
        let out = Rc::clone(&exes);
        parser.set_execute_handler(code, move |_| {
            out.borrow_mut().push(code);
            true
        });
    }
    parser.parse(&data(INPUT), None).unwrap();
    assert_eq!(*exes.borrow(), [13, 10]);
    parser.clear_execute_handler(13);
    parser.clear_execute_handler(13);
    exes.borrow_mut().clear();
    parser.parse(&data(INPUT), None).unwrap();
    assert_eq!(*exes.borrow(), [10]);
    let esc = Rc::new(RefCell::new(Vec::new()));
    for (intermediates, final_byte, value) in [(Some("%"), "G", "%G"), (None, "E", "E")] {
        let out = Rc::clone(&esc);
        parser
            .register_esc_handler(
                FunctionIdentifier {
                    prefix: None,
                    intermediates,
                    final_byte,
                },
                Rc::new(RefCell::new(move |_: &Params| {
                    out.borrow_mut().push(value);
                    HandlerResult::Ready(true)
                })),
            )
            .unwrap();
    }
    parser.parse(&data(INPUT), None).unwrap();
    assert_eq!(*esc.borrow(), ["%G", "E"]);
    clear(&mut parser, Kind::Esc);
    clear(&mut parser, Kind::Esc);
    esc.borrow_mut().clear();
    parser.parse(&data(INPUT), None).unwrap();
    assert_eq!(*esc.borrow(), ["E"]);
    parser
        .clear_esc_handler(FunctionIdentifier::new("E"))
        .unwrap();
    esc.borrow_mut().clear();
    parser.parse(&data(INPUT), None).unwrap();
    assert!(esc.borrow().is_empty());
}

struct Raw(Rc<RefCell<Vec<Event>>>);
impl DcsHandlerTrait for Raw {
    fn hook(&mut self, params: &Params) {
        self.0.borrow_mut().push(Event::DcsHook(params.to_array()));
    }
    fn put(&mut self, codes: &[u32]) {
        self.0
            .borrow_mut()
            .push(Event::DcsPut(text(&utf32_to_string(codes))));
    }
    fn unhook(&mut self, success: bool) -> HandlerResult {
        self.0.borrow_mut().push(Event::DcsEnd(success));
        HandlerResult::Ready(true)
    }
}
impl OscHandlerTrait for Raw {
    fn start(&mut self) {
        self.0.borrow_mut().push(Event::ApcStart);
    }
    fn put(&mut self, codes: &[u32]) {
        self.0
            .borrow_mut()
            .push(Event::ApcPut(text(&utf32_to_string(codes))));
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        self.0.borrow_mut().push(Event::ApcEnd(success));
        HandlerResult::Ready(true)
    }
}
#[test]
fn original_dcs_apc_raw_streaming_set_clear_handlers() {
    for kind in [Kind::Dcs, Kind::Apc] {
        let mut parser = EscapeSequenceParser::new();
        let events = Rc::new(RefCell::new(Vec::new()));
        let ident = FunctionIdentifier {
            prefix: None,
            intermediates: Some("+"),
            final_byte: "p",
        };
        let first = if matches!(kind, Kind::Dcs) {
            parser
                .register_dcs_handler(ident, Rc::new(RefCell::new(Raw(Rc::clone(&events)))))
                .unwrap();
            "\x1bP1;2;3+pabc"
        } else {
            parser
                .register_apc_handler(ident, Rc::new(RefCell::new(Raw(Rc::clone(&events)))))
                .unwrap();
            "\x1b_+pabc"
        };
        parser.parse(&data(first), None).unwrap();
        parser.parse(&data(";de\u{9c}"), None).unwrap();
        let expected = if matches!(kind, Kind::Dcs) {
            vec![
                Event::DcsHook(vec![V::Param(1), V::Param(2), V::Param(3)]),
                Event::DcsPut("abc".into()),
                Event::DcsPut(";de".into()),
                Event::DcsEnd(true),
            ]
        } else {
            vec![
                Event::ApcStart,
                Event::ApcPut("abc".into()),
                Event::ApcPut(";de".into()),
                Event::ApcEnd(true),
            ]
        };
        assert_eq!(*events.borrow(), expected);
        clear(&mut parser, kind);
        clear(&mut parser, kind);
        events.borrow_mut().clear();
        parser.parse(&data(first), None).unwrap();
        parser.parse(&data(";de\u{9c}"), None).unwrap();
        assert!(events.borrow().is_empty());
    }
}

#[test]
fn original_error_handler_diagnostic_and_clear() {
    let mut parser = EscapeSequenceParser::new();
    let observed = Rc::new(RefCell::new(None));
    let out = Rc::clone(&observed);
    parser.set_error_handler(move |state| {
        *out.borrow_mut() = Some((
            state.position,
            state.code,
            state.current_state,
            state.collect,
            state.params.to_array(),
        ));
        false
    });
    parser.parse(&data("\x1b[1;2;€;3m"), None).unwrap();
    assert_eq!(
        *observed.borrow(),
        Some((
            6,
            u32::from('€'),
            S::CsiParam,
            0,
            vec![V::Param(1), V::Param(2), V::Param(0)]
        ))
    );
    parser.clear_error_handler();
    parser.clear_error_handler();
    *observed.borrow_mut() = None;
    parser.parse(&data("\x1b[1;2;a;3m"), None).unwrap();
    assert!(observed.borrow().is_none());
}
