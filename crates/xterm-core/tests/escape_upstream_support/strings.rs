// Copyright (c) 2018 The xterm.js authors. All rights reserved. Licensed under MIT.
use super::states::{check, executables};
use super::*;

#[test]
fn original_dcs_apc_start_put_and_end_rules() {
    for state in [S::DcsEntry, S::DcsParam, S::DcsIntermediate] {
        for code in 0x40..0x7f {
            check(
                state,
                code,
                S::DcsPassthrough,
                vec![Event::DcsHook(vec![V::Param(0)])],
            );
        }
    }
    for code in executables().into_iter().chain(0x20..0x7f) {
        check(
            S::DcsPassthrough,
            code,
            S::DcsPassthrough,
            vec![Event::DcsPut(char::from_u32(code).unwrap().to_string())],
        );
    }
    for state in [S::ApcEntry, S::ApcIntermediate] {
        for code in 0x30..0x7f {
            check(state, code, S::ApcPassthrough, vec![Event::ApcStart]);
        }
    }
    for code in (8..0x0e).chain(0x20..0x7f) {
        check(
            S::ApcPassthrough,
            code,
            S::ApcPassthrough,
            vec![Event::ApcPut(char::from_u32(code).unwrap().to_string())],
        );
    }
    for (code, next, success) in [
        (0x9c, S::Ground, true),
        (0x18, S::Ground, false),
        (0x1a, S::Ground, false),
        (0x1b, S::Escape, true),
    ] {
        check(S::ApcPassthrough, code, next, vec![Event::ApcEnd(success)]);
    }
}

#[test]
fn original_osc_raw_put_and_ignore_expectations_with_real_dispatch() {
    for code in 0x20..0x80 {
        let mut fixture = Fixture::new();
        fixture.parse("\x1b]0;");
        fixture.clear();
        fixture.one(code);
        assert_eq!(fixture.parser.current_state, S::OscString);
        assert_eq!(*fixture.osc_data.borrow(), vec![code as u16]);
    }
    for code in executables().into_iter().filter(|&code| code != 7) {
        let mut fixture = Fixture::new();
        fixture.parse("\x1b]0;");
        fixture.clear();
        fixture.one(code);
        assert_eq!(fixture.parser.current_state, S::OscString);
        assert!(fixture.osc_data.borrow().is_empty());
    }
}

#[test]
fn original_unicode_coverage_cases() {
    for state in [S::CsiIgnore, S::DcsIgnore] {
        let mut fixture = Fixture::new();
        fixture.parser.current_state = state;
        fixture.parse("€öäü");
        assert_eq!(fixture.parser.current_state, state);
        assert!(fixture.events().is_empty());
    }
    let mut fixture = Fixture::new();
    fixture.parser.current_state = S::DcsPassthrough;
    fixture.parse("\u{90}1;2;3+$a€öäü");
    assert_eq!(fixture.parser.current_state, S::DcsPassthrough);
    assert_eq!(
        fixture.events(),
        vec![
            Event::DcsHook(vec![V::Param(1), V::Param(2), V::Param(3)]),
            Event::DcsPut("€öäü".into())
        ]
    );
    check(S::Ground, 0x9c, S::Ground, vec![]);
}
