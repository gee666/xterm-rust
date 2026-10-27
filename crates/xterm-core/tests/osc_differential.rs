// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Oracle generated from read-only OscParser.ts.
mod parser_support;
use parser_support::{report, Log, Reporter};
use std::cell::RefCell;
use std::rc::Rc;
use xterm_core::parser::osc::OscParser;

fn fallback(parser: &mut OscParser, log: &Log) {
    let log = Rc::clone(log);
    parser.set_handler_fallback(move |id, action| {
        // Rust spells infinity "inf", JavaScript spells it "Infinity".
        let id = if id.is_infinite() {
            "Infinity".to_owned()
        } else {
            id.to_string()
        };
        report(&log, id, action);
    });
}
#[test]
fn randomized_registration_lifecycle_and_streaming_matches_javascript() {
    let fixture = include_str!("fixtures/osc-oracle.tsv");
    for (index, line) in fixture.lines().enumerate() {
        let (operations, expected) = line.split_once('\t').unwrap();
        let log = Log::default();
        let mut parser = OscParser::new();
        let mut tokens = Vec::new();
        fallback(&mut parser, &log);
        for operation in operations.split(';') {
            let args: Vec<_> = operation.split(':').collect();
            match args[0] {
                "r" => tokens.push(parser.register_handler(
                    args[1].parse().unwrap(),
                    Rc::new(RefCell::new(Reporter {
                        name: args[2].to_owned(),
                        handled: args[3] == "1",
                        log: Rc::clone(&log),
                    })),
                )),
                "s" => parser.start().unwrap(),
                "p" => {
                    let data: Vec<_> = args[1]
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(|s| u32::from_str_radix(s, 16).unwrap())
                        .collect();
                    parser.put(&data).unwrap();
                }
                "e" => assert!(parser.end(args[1] == "1", true).unwrap().is_none()),
                "x" => parser.reset().unwrap(),
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
            "seed 0x05c2019 case {index}: {operations}"
        );
    }
}
