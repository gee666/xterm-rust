// Copyright (c) 2018 The xterm.js authors. All rights reserved. Licensed under MIT.
use super::*;

pub(super) fn executables() -> Vec<u32> {
    (0..0x18).chain([0x19]).chain(0x1c..0x20).collect()
}
pub(super) fn check(state: S, code: u32, next: S, events: Vec<Event>) {
    let mut fixture = Fixture::new();
    fixture.parser.current_state = state;
    fixture.one(code);
    assert_eq!(
        fixture.parser.current_state, next,
        "{state:?} code {code:x}"
    );
    assert_eq!(fixture.events(), events, "{state:?} code {code:x}");
}

#[test]
fn construction_initial_and_reset_states() {
    let mut fixture = Fixture::new();
    assert_eq!(fixture.parser.transitions(), &TransitionTable::vt500());
    assert_eq!(
        EscapeSequenceParser::with_transitions(TransitionTable::vt500()).transitions(),
        &TransitionTable::vt500()
    );
    assert_eq!(
        EscapeSequenceParser::with_transitions(TransitionTable::new(10))
            .transitions()
            .packed()
            .len(),
        10
    );
    assert_eq!(fixture.parser.initial_state, S::Ground);
    assert_eq!(fixture.parser.current_state, S::Ground);
    assert_eq!(fixture.parser.params().to_array(), vec![V::Param(0)]);
    assert_eq!(fixture.parser.collect(), 0);
    assert!(fixture.osc_data.borrow().is_empty());
    fixture.parse("\x1b[123#");
    assert_eq!(fixture.parser.params().values(), [123]);
    assert_eq!(fixture.parser.collect(), u32::from(b'#'));
    fixture.parser.reset().unwrap();
    assert_eq!(fixture.parser.current_state, S::Ground);
    assert_eq!(fixture.parser.params().to_array(), vec![V::Param(0)]);
    assert_eq!(fixture.parser.collect(), 0);
    // The source reset fixture writes '#' into a mock OSC buffer. Exercise the
    // same stale-payload risk through the real dispatcher rather than a mock.
    fixture.parse("\x1b]0;#");
    assert_eq!(*fixture.osc_data.borrow(), vec![u16::from(b'#')]);
    fixture.clear();
    fixture.parser.reset().unwrap();
    assert_eq!(fixture.parser.current_state, S::Ground);
    assert!(fixture.parser.osc.end(true, true).unwrap().is_none());
    assert!(
        fixture.events().is_empty(),
        "reset must discard the active OSC payload"
    );
    fixture.parse("\x1b]1;after\x07");
    assert_eq!(
        fixture.events(),
        vec![Event::Osc("1;after, success: true".into())]
    );
}

#[test]
fn original_executable_state_rules_and_ground_printables() {
    for state in [
        S::Ground,
        S::Escape,
        S::EscapeIntermediate,
        S::CsiEntry,
        S::CsiParam,
        S::CsiIntermediate,
        S::CsiIgnore,
    ] {
        for code in executables() {
            check(state, code, state, vec![Event::Execute(code)]);
        }
    }
    for code in 0x20..0x7f {
        check(
            S::Ground,
            code,
            S::Ground,
            vec![Event::Print(char::from_u32(code).unwrap().to_string())],
        );
    }
}

#[test]
fn original_anywhere_cancel_st_and_escape_clear_rules() {
    let codes = [0x18, 0x1a, 0x99, 0x9a]
        .into_iter()
        .chain(0x80..0x90)
        .chain(0x91..0x98);
    for code in codes {
        for state in S::ALL {
            let events = match (state, code) {
                (S::OscString, 0x18 | 0x1a) => vec![],
                (S::DcsPassthrough, 0x18 | 0x1a) => vec![Event::DcsEnd(false)],
                (S::ApcPassthrough, 0x18 | 0x1a) => vec![Event::ApcEnd(false)],
                _ => vec![Event::Execute(code)],
            };
            check(state, code, S::Ground, events);
        }
    }
    check(S::Ground, 0x9c, S::Ground, vec![]);
    for state in S::ALL {
        let mut fixture = Fixture::new();
        fixture.parse("\x1b[23#");
        fixture.parser.current_state = state;
        fixture.one(0x1b);
        assert_eq!(fixture.parser.current_state, S::Escape);
        assert_eq!(fixture.parser.params().values(), [0]);
        assert_eq!(fixture.parser.collect(), 0);
    }
}

#[test]
fn original_ignore_rules() {
    for state in [
        S::Escape,
        S::EscapeIntermediate,
        S::CsiEntry,
        S::CsiParam,
        S::CsiIntermediate,
        S::DcsPassthrough,
    ] {
        check(state, 0x7f, state, vec![]);
    }
    for state in [
        S::DcsEntry,
        S::DcsParam,
        S::DcsIntermediate,
        S::ApcEntry,
        S::ApcIntermediate,
    ] {
        for code in executables().into_iter().chain([0x7f]) {
            check(state, code, state, vec![]);
        }
    }
    for code in executables().into_iter().chain(0x20..0x80) {
        check(S::SosPmString, code, S::SosPmString, vec![]);
    }
    for code in executables().into_iter().chain(0x20..0x80) {
        check(S::DcsIgnore, code, S::DcsIgnore, vec![]);
    }
    for code in (0x20..0x40).chain([0x7f]) {
        check(S::CsiIgnore, code, S::CsiIgnore, vec![]);
    }
    for code in (0..8)
        .chain(0x0e..0x18)
        .chain([0x19])
        .chain(0x1c..0x20)
        .chain([0x7f])
    {
        check(S::ApcPassthrough, code, S::ApcPassthrough, vec![]);
    }
}

#[test]
fn original_esc_dispatch_ranges() {
    let direct = (0x30..0x50)
        .chain(0x51..0x58)
        .chain([0x59, 0x5a])
        .chain(0x60..0x7f);
    for code in direct {
        check(
            S::Escape,
            code,
            S::Ground,
            vec![Event::Esc(char::from_u32(code).unwrap().to_string())],
        );
    }
    for code in 0x30..0x7f {
        let expected = if code == 0x5c {
            vec![]
        } else {
            vec![Event::Esc(char::from_u32(code).unwrap().to_string())]
        };
        check(S::EscapeIntermediate, code, S::Ground, expected);
    }
}

#[test]
fn original_entry_clear_and_sos_pm_transitions() {
    for (code, next) in [
        (0x9b, S::CsiEntry),
        (0x90, S::DcsEntry),
        (0x9f, S::ApcEntry),
    ] {
        for state in S::ALL {
            let mut fixture = Fixture::new();
            fixture.parse("\x1b[123#");
            fixture.parser.current_state = state;
            fixture.one(code);
            assert_eq!(fixture.parser.current_state, next);
            assert_eq!(fixture.parser.params().values(), [0]);
            assert_eq!(fixture.parser.collect(), 0);
        }
    }
    for (prefix, next) in [
        ("\x1b[", S::CsiEntry),
        ("\x1bP", S::DcsEntry),
        ("\x1b_", S::ApcEntry),
        ("\x1b]", S::OscString),
        ("\x1bX", S::SosPmString),
        ("\x1b^", S::SosPmString),
    ] {
        let mut fixture = Fixture::new();
        fixture.parse(prefix);
        assert_eq!(fixture.parser.current_state, next);
    }
    for state in S::ALL {
        for (code, next) in [
            (0x98, S::SosPmString),
            (0x9e, S::SosPmString),
            (0x9d, S::OscString),
        ] {
            let mut fixture = Fixture::new();
            fixture.parser.current_state = state;
            fixture.one(code);
            assert_eq!(fixture.parser.current_state, next);
        }
    }
}
