// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Oracle generated from read-only ApcParser.ts.

use std::cell::RefCell;
use std::rc::Rc;
use xterm_core::parser::apc::{ApcAction, ApcHandlerTrait, ApcParser};
use xterm_core::parser::handler::HandlerResult;
use xterm_core::text_decoder::utf32_to_string;

type Log = Rc<RefCell<Vec<String>>>;
fn units(data: &[u16]) -> String {
    data.iter()
        .map(|unit| format!("{unit:x}"))
        .collect::<Vec<_>>()
        .join(",")
}
struct Reporter {
    name: String,
    handled: bool,
    log: Log,
}
impl ApcHandlerTrait for Reporter {
    fn start(&mut self) {
        self.log.borrow_mut().push(format!("h:{}", self.name));
    }
    fn put(&mut self, data: &[u32]) {
        self.log
            .borrow_mut()
            .push(format!("p:{}:{}", self.name, units(&utf32_to_string(data))));
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        self.log
            .borrow_mut()
            .push(format!("e:{}:{}", self.name, u8::from(success)));
        HandlerResult::Ready(self.handled)
    }
}
fn fallback(parser: &mut ApcParser, log: &Log) {
    let log = Rc::clone(log);
    parser.set_handler_fallback(move |id, action| {
        let event = match action {
            ApcAction::Start => "s".to_string(),
            ApcAction::Put(data) => format!("p:{}", units(&data)),
            ApcAction::End(success) => format!("e:{}", u8::from(success)),
        };
        log.borrow_mut().push(format!("f:{id}:{event}"));
    });
}

#[test]
fn randomized_registration_lifecycle_and_streaming_matches_javascript() {
    let fixture = include_str!("fixtures/apc-oracle.tsv");
    for (index, line) in fixture.lines().enumerate() {
        let (operations, expected) = line.split_once('\t').unwrap();
        let log = Log::default();
        let mut parser = ApcParser::new();
        let mut tokens = Vec::new();
        fallback(&mut parser, &log);
        for operation in operations.split(';') {
            let args: Vec<_> = operation.split(':').collect();
            match args[0] {
                "r" => {
                    tokens.push(parser.register_handler(
                        args[1].parse().unwrap(),
                        Rc::new(RefCell::new(Reporter {
                            name: args[2].to_owned(),
                            handled: args[3] == "1",
                            log: Rc::clone(&log),
                        })),
                    ));
                }
                "s" => parser.start(args[1].parse().unwrap()),
                "p" => {
                    let data: Vec<_> = args[1]
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(|s| u32::from_str_radix(s, 16).unwrap())
                        .collect();
                    parser.put(&data);
                }
                "e" => assert!(parser.end(args[1] == "1", true).is_none()),
                "x" => parser.reset(),
                "c" => parser.clear_handler(args[1].parse().unwrap()),
                "d" => tokens[args[1].parse::<usize>().unwrap()].dispose(),
                "q" => {
                    parser.dispose();
                    fallback(&mut parser, &log);
                }
                _ => panic!("unknown fixture operation {operation}"),
            }
        }
        assert_eq!(
            log.borrow().join("|"),
            expected,
            "seed 0xa9c2025 case {index}: {operations}"
        );
    }
}
