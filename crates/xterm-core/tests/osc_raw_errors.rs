// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Exact original start/put exception and recovery traces.
use std::{cell::RefCell, rc::Rc};
use xterm_core::parser::escape::{EscapeSequenceParser, ParseError};
use xterm_core::{
    event::EmitError,
    parser::{
        handler::{DispatchError, HandlerResult},
        osc::{OscHandlerTrait, OscParser},
    },
};
type Log = Rc<RefCell<Vec<String>>>;
struct Handler {
    name: usize,
    failing: usize,
    phase: char,
    failed: bool,
    marker: DispatchError,
    log: Log,
}
impl Handler {
    fn report(&mut self, phase: char, message: String) -> Result<(), DispatchError> {
        self.log.borrow_mut().push(message);
        if self.name == self.failing && self.phase == phase && !self.failed {
            self.failed = true;
            return Err(self.marker.clone());
        }
        Ok(())
    }
}
impl OscHandlerTrait for Handler {
    fn start(&mut self) -> Result<(), DispatchError> {
        self.report('s', format!("s{}", self.name))
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        let text = xterm_core::text_decoder::utf32_to_string(data)
            .iter()
            .map(|unit| format!("{unit:x}"))
            .collect::<Vec<_>>()
            .join(",");
        self.report('p', format!("p{}:{text}", self.name))
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        self.log
            .borrow_mut()
            .push(format!("e{}:{}", self.name, u8::from(success)));
        HandlerResult::Ready(true)
    }
}
#[test]
fn raw_phase_error_stops_real_outer_parser_without_rolling_back_printed_prefix() {
    for phase in ['s', 'p'] {
        let log = Log::default();
        let marker = DispatchError::Callback(EmitError::callback(std::io::Error::other(
            "outer raw marker",
        )));
        let mut parser = EscapeSequenceParser::new();
        let printed = Rc::new(RefCell::new(Vec::new()));
        let output = Rc::clone(&printed);
        parser.set_print_handler(move |data| output.borrow_mut().extend_from_slice(data));
        for name in 0..3 {
            parser.osc.register_handler(
                2.0,
                Rc::new(RefCell::new(Handler {
                    name,
                    failing: 1,
                    phase,
                    failed: false,
                    marker: marker.clone(),
                    log: Rc::clone(&log),
                })),
            );
        }
        assert_eq!(
            parser.parse(&[88, 27, 93, 50, 59, 65, 7, 90], None).err(),
            Some(ParseError::Dispatch(marker))
        );
        assert_eq!(*printed.borrow(), [88]);
        parser.reset().unwrap();
        assert!(parser.parse(&[81], None).unwrap().is_none());
        assert_eq!(*printed.borrow(), [88, 81]);
    }
}
#[test]
fn raw_start_put_errors_stop_immediately_preserve_cause_and_match_source_recovery() {
    let mut cases = 0;
    for line in include_str!("fixtures/osc-raw-failures.tsv").lines() {
        let fields: Vec<_> = line.split('\t').collect();
        let scenario = fields[0].chars().next().unwrap();
        let phase = if scenario == 'i' { 's' } else { scenario };
        let failing = fields[1].parse().unwrap();
        let reset = fields[2] == "1";
        let log = Log::default();
        let marker = DispatchError::Callback(EmitError::callback(std::io::Error::other(
            "raw phase marker",
        )));
        let mut parser = OscParser::new();
        for name in 0..3 {
            parser.register_handler(
                2.0,
                Rc::new(RefCell::new(Handler {
                    name,
                    failing,
                    phase,
                    failed: false,
                    marker: marker.clone(),
                    log: Rc::clone(&log),
                })),
            );
        }
        parser.start().unwrap();
        if scenario == 'i' {
            parser.put(&[50]).unwrap();
            assert_eq!(parser.end(true, true).err(), Some(marker));
        } else {
            assert_eq!(parser.put(&[50, 59, 65]).err(), Some(marker));
        }
        if reset {
            parser.reset().unwrap();
            parser.start().unwrap();
            parser.put(&[50, 59, 66]).unwrap();
        } else if scenario != 'i' {
            parser.put(&[66]).unwrap();
        }
        assert!(parser.end(true, true).unwrap().is_none());
        assert_eq!(
            log.borrow().join("|"),
            fields[3],
            "phase{phase} failing{failing} reset{reset}"
        );
        cases += 1;
    }
    assert_eq!(cases, 18);
}
