// Copyright (c) 2018 The xterm.js authors. All rights reserved. Licensed under MIT.
use super::*;

#[test]
fn original_collect_ranges() {
    for (state, next) in [
        (S::Escape, S::EscapeIntermediate),
        (S::EscapeIntermediate, S::EscapeIntermediate),
        (S::CsiEntry, S::CsiIntermediate),
        (S::CsiParam, S::CsiIntermediate),
        (S::CsiIntermediate, S::CsiIntermediate),
        (S::DcsEntry, S::DcsIntermediate),
        (S::DcsParam, S::DcsIntermediate),
        (S::DcsIntermediate, S::DcsIntermediate),
        (S::ApcEntry, S::ApcIntermediate),
    ] {
        for code in 0x20..0x30 {
            let mut fixture = Fixture::new();
            fixture.parser.current_state = state;
            fixture.one(code);
            assert_eq!(fixture.parser.current_state, next);
            assert_eq!(fixture.parser.collect(), code);
        }
    }
}

#[test]
fn original_parameter_and_prefix_rules() {
    for (state, next) in [
        (S::CsiEntry, S::CsiParam),
        (S::CsiParam, S::CsiParam),
        (S::DcsEntry, S::DcsParam),
        (S::DcsParam, S::DcsParam),
    ] {
        for code in 0x30..0x3a {
            let mut fixture = Fixture::new();
            fixture.parser.current_state = state;
            fixture.one(code);
            assert_eq!(fixture.parser.current_state, next);
            assert_eq!(fixture.parser.params().values(), [code as i32 - 48]);
        }
        for (code, expected) in [
            (0x3b, vec![V::Param(0), V::Param(0)]),
            (0x3a, vec![V::Param(0), V::SubParams(vec![-1])]),
        ] {
            let mut fixture = Fixture::new();
            fixture.parser.current_state = state;
            fixture.one(code);
            assert_eq!(fixture.parser.current_state, next);
            assert_eq!(fixture.parser.params().to_array(), expected);
        }
    }
    for (state, next) in [(S::CsiEntry, S::CsiParam), (S::DcsEntry, S::DcsParam)] {
        for code in 0x3c..0x40 {
            let mut fixture = Fixture::new();
            fixture.parser.current_state = state;
            fixture.one(code);
            assert_eq!(fixture.parser.current_state, next);
            assert_eq!(fixture.parser.collect(), code);
        }
    }
}

#[test]
fn original_ignore_entry_from_parameter_and_intermediate() {
    for (state, next) in [(S::CsiParam, S::CsiIgnore), (S::DcsParam, S::DcsIgnore)] {
        for code in 0x3c..0x40 {
            let mut fixture = Fixture::new();
            fixture.parser.current_state = state;
            fixture.one(0x3b);
            fixture.one(code);
            assert_eq!(fixture.parser.current_state, next);
            assert_eq!(fixture.parser.params().values(), [0, 0]);
        }
    }
    for (state, next) in [
        (S::CsiIntermediate, S::CsiIgnore),
        (S::DcsIntermediate, S::DcsIgnore),
    ] {
        for code in 0x30..0x40 {
            let mut fixture = Fixture::new();
            fixture.parser.current_state = state;
            fixture.one(code);
            assert_eq!(fixture.parser.current_state, next);
            assert_eq!(fixture.parser.params().values(), [0]);
        }
    }
    // The second original DCS_INTERMEDIATE case collects one more space first.
    for code in 0x30..0x40 {
        let mut fixture = Fixture::new();
        fixture.parser.current_state = S::DcsIntermediate;
        fixture.one(0x20);
        fixture.one(code);
        assert_eq!(fixture.parser.current_state, S::DcsIgnore);
        assert_eq!(fixture.parser.collect(), 0x20);
    }
}

#[test]
fn original_csi_dispatch_and_ignore_exit_rules() {
    for state in [S::CsiEntry, S::CsiParam, S::CsiIntermediate, S::CsiIgnore] {
        for code in 0x40..0x7f {
            let mut fixture = Fixture::new();
            if state != S::CsiEntry {
                fixture.parse("\x1b[0;1");
            }
            fixture.parser.current_state = state;
            fixture.one(code);
            assert_eq!(fixture.parser.current_state, S::Ground);
            let expected = if state == S::CsiIgnore {
                vec![]
            } else {
                vec![Event::Csi(
                    char::from_u32(code).unwrap().to_string(),
                    if state == S::CsiEntry {
                        vec![V::Param(0)]
                    } else {
                        vec![V::Param(0), V::Param(1)]
                    },
                )]
            };
            assert_eq!(fixture.events(), expected);
        }
    }
}
