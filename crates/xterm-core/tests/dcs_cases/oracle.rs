// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Expected reports are generated only by the unchanged upstream DCS parser.

use crate::dcs_support::*;

#[test]
fn seeded_upstream_differential_traces() {
    let corpus = include_str!("../fixtures/dcs-oracle.tsv");
    let mut count = 0;
    for (case, line) in corpus.lines().enumerate() {
        let (operations, expected) = line.split_once('\t').unwrap();
        let (mut parser, reports) = parser();
        let mut disposables = Vec::new();
        for operation in operations.split('|') {
            let fields: Vec<_> = operation.split(':').collect();
            match fields[0] {
                "r" => disposables.push(parser.register_handler(
                    IDENT,
                    handler(&reports, fields[1], fields[2] == "1", fields[3] == "1"),
                )),
                "h" => parser.hook(IDENT, &params(&[1, 2, 3])).unwrap(),
                "c" => parser.clear_handler(IDENT),
                "d" => disposables[fields[1].parse::<usize>().unwrap()].dispose(),
                "p" => {
                    let data: Vec<u32> = fields[1]
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.parse().unwrap())
                        .collect();
                    parser.put(&data).unwrap();
                }
                "u" => unhook(&mut parser, fields[1] == "1"),
                "x" => parser.reset().unwrap(),
                other => panic!("unknown oracle operation {other}"),
            }
        }
        assert_eq!(
            reports.borrow().join("|"),
            expected,
            "seed 0xdcc512 trace {case}"
        );
        count += 1;
    }
    assert_eq!(count, 512);
}
