// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Raw streaming failures must escape immediately, not wait for end/unhook.
use std::{cell::RefCell, error::Error, rc::Rc};
use xterm_core::{
    event::EmitError,
    parser::{
        apc::ApcHandlerTrait,
        dcs::DcsHandlerTrait,
        escape::{EscapeSequenceParser, FunctionIdentifier},
        handler::{DispatchError, HandlerResult},
        osc::OscHandlerTrait,
        params::Params,
        table::ParserState,
    },
};
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Start,
    Put,
}
struct Raw {
    label: &'static str,
    phase: Phase,
    error: DispatchError,
    log: Rc<RefCell<Vec<String>>>,
}
impl Raw {
    fn record(&self, name: &str, phase: Phase) -> Result<(), DispatchError> {
        self.log.borrow_mut().push(format!("{}:{name}", self.label));
        if self.label == "new" && self.phase == phase {
            Err(self.error.clone())
        } else {
            Ok(())
        }
    }
}
impl OscHandlerTrait for Raw {
    fn start(&mut self) -> Result<(), DispatchError> {
        self.record("start", Phase::Start)
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        assert_eq!(data, [97, 98, 99]);
        self.record("put", Phase::Put)
    }
    fn end(&mut self, _: bool) -> HandlerResult {
        self.log.borrow_mut().push(format!("{}:end", self.label));
        HandlerResult::Ready(true)
    }
}
impl DcsHandlerTrait for Raw {
    fn hook(&mut self, params: &Params) -> Result<(), DispatchError> {
        assert_eq!(params.values(), [1]);
        self.record("hook", Phase::Start)
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        assert_eq!(data, [97, 98, 99]);
        self.record("put", Phase::Put)
    }
    fn unhook(&mut self, _: bool) -> HandlerResult {
        self.log.borrow_mut().push(format!("{}:end", self.label));
        HandlerResult::Ready(true)
    }
}
#[test]
fn raw_start_and_put_errors_stop_older_handlers_and_following_text_in_all_families() {
    for family in ["OSC", "DCS", "APC"] {
        for phase in [Phase::Start, Phase::Put] {
            let mut parser = EscapeSequenceParser::new();
            let log = Rc::new(RefCell::new(Vec::new()));
            let source: Rc<dyn Error> = Rc::new(std::io::Error::other("raw streaming sentinel"));
            let error = DispatchError::Callback(EmitError::Callback(Rc::clone(&source)));
            for label in ["old", "new"] {
                let raw = Raw {
                    label,
                    phase,
                    error: error.clone(),
                    log: Rc::clone(&log),
                };
                match family {
                    "OSC" => {
                        parser
                            .osc
                            .register_handler(52.0, Rc::new(RefCell::new(raw)));
                    }
                    "DCS" => {
                        parser
                            .register_dcs_handler(
                                FunctionIdentifier::new("q"),
                                Rc::new(RefCell::new(raw)),
                            )
                            .unwrap();
                    }
                    _ => {
                        let raw: Rc<RefCell<dyn ApcHandlerTrait>> = Rc::new(RefCell::new(raw));
                        parser
                            .register_apc_handler(FunctionIdentifier::new("X"), raw)
                            .unwrap();
                    }
                }
            }
            let output = Rc::clone(&log);
            parser.set_print_handler(move |_| output.borrow_mut().push("PRINT".to_owned()));
            parser.preceding_join_state.set(123);
            let text = match family {
                "OSC" => "\x1b]52;abc\x1b\\suffix",
                "DCS" => "\x1bP1qabc\x1b\\suffix",
                _ => "\x1b_Xabc\x1b\\suffix",
            };
            let data: Vec<u32> = text.chars().map(u32::from).collect();
            let error = parser.parse(&data, None).err().unwrap();
            let mut root: &dyn Error = &error;
            while let Some(next) = root.source() {
                root = next;
            }
            assert!(std::ptr::eq(root, source.as_ref()));
            let start = if family == "DCS" { "hook" } else { "start" };
            let expected = if phase == Phase::Start {
                vec![format!("new:{start}")]
            } else {
                vec![
                    format!("new:{start}"),
                    format!("old:{start}"),
                    "new:put".to_owned(),
                ]
            };
            assert_eq!(*log.borrow(), expected, "{family} immediate callback phase");
            let state = match (family, phase) {
                ("DCS", Phase::Start) => ParserState::DcsParam,
                ("DCS", Phase::Put) => ParserState::DcsPassthrough,
                ("APC", Phase::Start) => ParserState::ApcEntry,
                ("APC", Phase::Put) => ParserState::ApcPassthrough,
                _ => ParserState::OscString,
            };
            assert_eq!(parser.current_state, state);
            assert_eq!(parser.preceding_join_state.get(), 123);
        }
    }
}
