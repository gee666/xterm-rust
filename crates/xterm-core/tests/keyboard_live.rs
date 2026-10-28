// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
#[path = "support/keyboard_live.rs"]
mod support;
use std::rc::Rc;
use support::{units, Source};
use xterm_core::keyboard::try_evaluate_keyboard_event;

#[test]
fn exact_live_getters_mutations_numbers_and_throw_identity_match_source() {
    let fixture = include_str!("fixtures/keyboard-live.tsv");
    let mut cases = 0;
    let mut failures = 0;
    let mut getter_reads = 0;
    for line in fixture.lines() {
        let p: Vec<_> = line.split('\t').collect();
        assert_eq!(p.len(), 14);
        let options: u8 = p[1].parse().unwrap();
        let source = Source::from_fields(&p, 3, 2);
        let actual = try_evaluate_keyboard_event(
            &source,
            options & 1 != 0,
            options & 2 != 0,
            options & 4 != 0,
        );
        assert_eq!(
            source.reads.borrow().as_str(),
            p[10],
            "{} throw{}",
            p[0],
            p[2]
        );
        getter_reads += source.reads.borrow().len();
        if p[11] == "throw" {
            failures += 1;
            let actual = actual.expect_err("source getter must fail");
            assert!(Rc::ptr_eq(&actual, &source.marker));
        } else {
            let actual = actual.expect("source case must succeed");
            assert_eq!(
                actual.result_type as u8,
                p[11].parse::<u8>().unwrap(),
                "{}",
                p[0]
            );
            assert_eq!(actual.cancel, p[12] == "1", "{}", p[0]);
            let expected = (p[13] != "!").then(|| units(p[13]));
            assert_eq!(actual.key, expected, "{}", p[0]);
        }
        cases += 1;
    }
    assert_eq!(cases, 47598);
    assert_eq!(failures, 42310);
    assert_eq!(getter_reads, 337507);
}
