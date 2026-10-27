// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Oracle generated from read-only ApcParser.ts.

use std::cell::RefCell;
use std::rc::Rc;
use xterm_core::parser::apc::ApcParser;
mod parser_support;
use parser_support::{report, Log, Reporter};

fn fallback(parser: &mut ApcParser, log: &Log) {
    let log = Rc::clone(log);
    parser.set_handler_fallback(move |id, action| report(&log, id, action));
}

#[test]
fn randomized_registration_lifecycle_and_streaming_matches_javascript() {
    let fixture = include_str!("fixtures/apc-oracle.tsv");
    assert_eq!(fixture.lines().count(), 256);
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
                "s" => parser.start(args[1].parse().unwrap()).unwrap(),
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
            "seed 0xa9c2025 case {index}: {operations}"
        );
    }
}
