// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Source fallback exception and partial-state regressions.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    event::EmitError,
    parser::{
        escape::{EscapeSequenceParser, ParseError},
        handler::DispatchError,
        osc::{OscAction, OscParser},
    },
};
type Log = Rc<RefCell<Vec<String>>>;
fn action(action: &OscAction) -> String {
    match action {
        OscAction::Start => "s".into(),
        OscAction::Put(value) => format!(
            "p:{}",
            value
                .iter()
                .map(|unit| format!("{unit:x}"))
                .collect::<Vec<_>>()
                .join(",")
        ),
        OscAction::End(success) => format!("e:{}", u8::from(*success)),
    }
}
fn failing(phase: char) -> (OscParser, Log, DispatchError) {
    let mut parser = OscParser::new();
    let log = Log::default();
    let output = Rc::clone(&log);
    let remaining = Cell::new(true);
    let marker = DispatchError::Callback(EmitError::callback(std::io::Error::other(
        "fallback marker",
    )));
    let error = marker.clone();
    parser.set_handler_fallback_checked(move |id, value| {
        let message = action(&value);
        let matches = message.starts_with(phase);
        output.borrow_mut().push(format!("{id}:{message}"));
        if matches && remaining.replace(false) {
            Err(error.clone())
        } else {
            Ok(())
        }
    });
    (parser, log, marker)
}
fn failed<T>(result: Result<T, DispatchError>, marker: &DispatchError) {
    assert_eq!(result.err().as_ref(), Some(marker));
}
#[test]
fn every_fallback_throw_path_retains_source_retry_state_and_actual_cause() {
    for scenario in ["start_put", "start_end", "put", "end_payload", "end_id"] {
        let phase = if scenario.starts_with("start") {
            's'
        } else if scenario == "put" {
            'p'
        } else {
            'e'
        };
        let (mut parser, log, marker) = failing(phase);
        parser.start().unwrap();
        let expected = match scenario {
            "start_put" => {
                failed(parser.put(&[50, 59, 65]), &marker);
                parser.put(&[66]).unwrap();
                assert!(parser.end(true, true).unwrap().is_none());
                vec!["2:s", "2:p:42", "2:e:1"]
            }
            "start_end" => {
                parser.put(&[50]).unwrap();
                failed(parser.end(true, true), &marker);
                assert!(parser.end(true, true).unwrap().is_none());
                vec!["2:s", "2:s", "2:e:1"]
            }
            "put" => {
                parser.put(&[50, 59]).unwrap();
                failed(parser.put(&[65]), &marker);
                parser.put(&[66]).unwrap();
                assert!(parser.end(true, true).unwrap().is_none());
                vec!["2:s", "2:p:41", "2:p:42", "2:e:1"]
            }
            "end_payload" => {
                parser.put(&[50, 59, 65]).unwrap();
                failed(parser.end(true, true), &marker);
                assert!(parser.end(false, true).unwrap().is_none());
                vec!["2:s", "2:p:41", "2:e:1", "2:e:0"]
            }
            "end_id" => {
                parser.put(&[50]).unwrap();
                failed(parser.end(true, true), &marker);
                assert!(parser.end(true, true).unwrap().is_none());
                vec!["2:s", "2:e:1", "2:s", "2:e:1"]
            }
            _ => unreachable!(),
        };
        assert_eq!(*log.borrow(), expected, "scenario {scenario}");
    }
}
#[test]
fn reset_and_dispose_do_not_invent_fallback_cleanup_after_exception() {
    for phase in ['s', 'p', 'e'] {
        let (mut parser, log, marker) = failing(phase);
        parser.start().unwrap();
        if phase == 's' || phase == 'p' {
            failed(parser.put(&[50, 59, 65]), &marker);
        } else {
            parser.put(&[50, 59, 65]).unwrap();
            failed(parser.end(true, true), &marker);
        }
        let length = log.borrow().len();
        parser.reset().unwrap();
        assert_eq!(log.borrow().len(), length);
        parser.dispose();
        parser.start().unwrap();
        parser.put(&[50, 59, 66]).unwrap();
        assert!(parser.end(true, true).unwrap().is_none());
        assert_eq!(log.borrow().len(), length);
    }
}
#[test]
fn checked_fallback_failure_stops_real_outer_parse_before_suffix_and_reset_recovers() {
    for phase in ['s', 'p', 'e'] {
        let mut parser = EscapeSequenceParser::new();
        let output = Rc::new(RefCell::new(Vec::new()));
        let printed = Rc::clone(&output);
        parser.set_print_handler(move |data| printed.borrow_mut().extend_from_slice(data));
        parser.osc.set_handler_fallback_checked(move |_, value| {
            if action(&value).starts_with(phase) {
                Err(DispatchError::CallbackFailed)
            } else {
                Ok(())
            }
        });
        assert!(matches!(
            parser.parse(&[27, 93, 50, 59, 65, 7, 90], None),
            Err(ParseError::Dispatch(DispatchError::CallbackFailed))
        ));
        assert!(output.borrow().is_empty());
        parser.reset().unwrap();
        assert!(parser.parse(&[81], None).unwrap().is_none());
        assert_eq!(*output.borrow(), [81]);
    }
}
