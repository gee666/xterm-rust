// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Diagnostic/fallback exceptions stop before suffix bytes and state cleanup.
use std::{
    cell::{Cell, RefCell},
    error::Error,
    rc::Rc,
};
use xterm_core::{
    event::EmitError,
    parser::{
        escape::{EscapeSequenceParser, ParseError},
        handler::DispatchError,
        table::ParserState,
    },
};
fn data(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}
fn failure() -> (ParseError, Rc<dyn Error>) {
    let source: Rc<dyn Error> = Rc::new(std::io::Error::other("diagnostic sentinel"));
    (
        ParseError::Dispatch(DispatchError::Callback(EmitError::Callback(Rc::clone(
            &source,
        )))),
        source,
    )
}
fn same_cause(error: &ParseError, expected: &Rc<dyn Error>) {
    let mut source: &dyn Error = error;
    while let Some(next) = source.source() {
        source = next;
    }
    assert!(std::ptr::eq(source, expected.as_ref()));
}

#[test]
fn prepacked_registrars_dispatch_and_clear_without_second_identifier_conversion() {
    for (esc, key, input) in [(true, 0x2547, "\x1b%G"), (false, 0x3f20216d, "\x1b[? !m")] {
        let mut parser = EscapeSequenceParser::new();
        let registrar = if esc {
            parser.esc_registrar()
        } else {
            parser.csi_registrar()
        };
        let calls = Rc::new(Cell::new(0));
        let target = Rc::clone(&calls);
        registrar
            .register_packed(
                key,
                Rc::new(RefCell::new(
                    move |_: &xterm_core::parser::params::Params| {
                        target.set(target.get() + 1);
                        xterm_core::parser::handler::HandlerResult::Ready(true)
                    },
                )),
            )
            .unwrap();
        assert!(parser.parse(&data(input), None).unwrap().is_none());
        assert_eq!(calls.get(), 1);
        registrar.clear_packed(key).unwrap();
        assert!(parser.parse(&data(input), None).unwrap().is_none());
        assert_eq!(calls.get(), 1);
        drop(parser);
        assert_eq!(
            registrar.clear_packed(key),
            Err(xterm_core::parser::escape::RegistrarError::ParserDropped)
        );
    }
}

#[test]
fn checked_execute_fallback_stops_fast_and_table_paths_before_join_reset() {
    for code in [10, 0x84] {
        let mut parser = EscapeSequenceParser::new();
        parser.preceding_join_state.set(123);
        let printed = Rc::new(Cell::new(false));
        let output = Rc::clone(&printed);
        parser.set_print_handler(move |_| output.set(true));
        let (error, source) = failure();
        parser.set_execute_handler_fallback_checked(move |actual| {
            assert_eq!(actual, code);
            Err(error.clone())
        });
        let input = [vec![code], data("suffix")].concat();
        let error = parser.parse(&input, None).err().unwrap();
        same_cause(&error, &source);
        assert!(!printed.get());
        assert_eq!(parser.preceding_join_state.get(), 123);
        assert_eq!(parser.current_state, ParserState::Ground);
    }
}
#[test]
fn checked_csi_fallback_stops_fast_and_split_paths_at_source_dispatch_state() {
    for split in [false, true] {
        let mut parser = EscapeSequenceParser::new();
        parser.preceding_join_state.set(123);
        let (error, source) = failure();
        parser.set_csi_handler_fallback_checked(move |id, params| {
            assert_eq!(id, u32::from(b'm'));
            assert_eq!(params.values(), [1]);
            Err(error.clone())
        });
        let printed = Rc::new(Cell::new(false));
        let output = Rc::clone(&printed);
        parser.set_print_handler(move |_| output.set(true));
        if split {
            assert!(parser.parse(&data("\x1b["), None).unwrap().is_none());
        }
        let input = if split { "1msuffix" } else { "\x1b[1msuffix" };
        let error = parser.parse(&data(input), None).err().unwrap();
        same_cause(&error, &source);
        assert!(!printed.get());
        assert_eq!(parser.preceding_join_state.get(), 123);
        assert_eq!(
            parser.current_state,
            if split {
                ParserState::CsiParam
            } else {
                ParserState::Ground
            }
        );
    }
}
#[test]
fn checked_esc_fallback_retains_collect_and_join_when_diagnostics_fail() {
    let mut parser = EscapeSequenceParser::new();
    parser.preceding_join_state.set(123);
    let (error, source) = failure();
    parser.set_esc_handler_fallback_checked(move |id| {
        assert_eq!(id, 0x2547);
        Err(error.clone())
    });
    let error = parser.parse(&data("\x1b%Gsuffix"), None).err().unwrap();
    same_cause(&error, &source);
    assert_eq!(parser.collect(), 0x25);
    assert_eq!(parser.preceding_join_state.get(), 123);
    assert_eq!(parser.current_state, ParserState::EscapeIntermediate);
}
#[test]
fn checked_error_handler_preserves_diagnostic_state_and_actual_error_identity() {
    let mut parser = EscapeSequenceParser::new();
    parser.preceding_join_state.set(123);
    let (error, source) = failure();
    parser.set_error_handler_checked(move |state| {
        assert_eq!(state.position, 3);
        assert_eq!(state.code, 0x20ac);
        assert_eq!(state.current_state, ParserState::CsiParam);
        assert_eq!(state.params.values(), [1]);
        Err(error.clone())
    });
    let error = parser.parse(&data("\x1b[1€suffix"), None).err().unwrap();
    same_cause(&error, &source);
    assert_eq!(parser.current_state, ParserState::CsiParam);
    assert_eq!(parser.preceding_join_state.get(), 123);
}
#[test]
fn checked_error_abort_and_recovery_match_original_abort_flag_semantics() {
    for abort in [false, true] {
        let mut parser = EscapeSequenceParser::new();
        let printed = Rc::new(RefCell::new(Vec::new()));
        let output = Rc::clone(&printed);
        parser.set_print_handler(move |text| output.borrow_mut().extend_from_slice(text));
        parser.set_error_handler_checked(move |_| Ok(abort));
        assert!(parser
            .parse(&data("\x1b[1€suffix"), None)
            .unwrap()
            .is_none());
        assert_eq!(
            parser.current_state,
            if abort {
                ParserState::CsiParam
            } else {
                ParserState::Ground
            }
        );
        assert_eq!(
            *printed.borrow(),
            if abort { Vec::new() } else { data("suffix") }
        );
    }
}
