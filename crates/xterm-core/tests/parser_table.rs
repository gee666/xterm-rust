// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use xterm_core::parser::table::{
    ParserAction as A, ParserState as S, Transition, TransitionTable, NON_ASCII_PRINTABLE,
    PAYLOAD_LIMIT, STATE_COUNT,
};

#[test]
fn exhaustive_vt500_oracle() {
    let table = TransitionTable::vt500();
    let expected: Vec<u16> = include_str!("fixtures/vt500-table.txt")
        .lines()
        .map(|line| line.parse().unwrap())
        .collect();
    assert_eq!(table.packed(), expected);
    assert_eq!(table.packed().len(), 4257);
    for state in S::ALL {
        for code in 0..=u8::MAX {
            let index = ((state as usize) << 8) | usize::from(code);
            let actual = table.get(code, state);
            if let Some(value) = expected.get(index) {
                assert_eq!(
                    actual.unwrap().packed(),
                    *value,
                    "state {state:?} byte {code}"
                );
            } else {
                assert_eq!(actual, None);
            }
        }
    }
    assert_eq!(TransitionTable::default(), table);
}

#[test]
fn custom_table_mutation_matches_source() {
    let mut table = TransitionTable::new(512);
    assert!(table.packed().iter().all(|value| *value == 0));
    table.set_default(Transition::new(A::Error, S::Ground));
    assert!(table.packed().iter().all(|value| *value == 256));
    table
        .add(65, S::Ground, Transition::new(A::Print, S::Escape))
        .unwrap();
    assert_eq!(table.packed()[65], 513);
    table
        .add_many(
            &[1, 2, 3],
            S::Escape,
            Transition::new(A::Execute, S::Ground),
        )
        .unwrap();
    for code in [1, 2, 3] {
        assert_eq!(table.packed()[256 + code], 768);
    }
}

#[test]
fn typed_constants_and_packed_roundtrip() {
    assert_eq!(STATE_COUNT, 17);
    assert_eq!(PAYLOAD_LIMIT, 10_000_000);
    assert_eq!(NON_ASCII_PRINTABLE, 0xa0);
    for (index, state) in S::ALL.iter().enumerate() {
        assert_eq!(*state as usize, index);
        assert_eq!(S::from_code(index as u8), Some(*state));
        for (action_code, action) in A::ALL.iter().enumerate() {
            assert_eq!(*action as usize, action_code);
            let value = Transition::new(*action, *state);
            assert_eq!(Transition::from_packed(value.packed()), Some(value));
        }
    }
    for invalid in 17..=u8::MAX {
        assert_eq!(S::from_code(invalid), None);
    }
    for invalid in 18..=u8::MAX {
        assert_eq!(A::from_code(invalid), None);
    }
    assert_eq!(Transition::from_packed(0xff00), None);
    assert_eq!(Transition::from_packed(0x00ff), None);
}

#[test]
fn unicode_classification_and_known_transitions() {
    let table = TransitionTable::vt500();
    for state in S::ALL {
        for value in [0xa0, 0xff, 0x100, 0x1f600, 0x10ffff, u32::MAX] {
            assert_eq!(table.for_codepoint(value, state), table.get(0xa0, state));
        }
        for value in 0..0xa0 {
            assert_eq!(
                table.for_codepoint(value, state),
                table.get(value as u8, state)
            );
        }
    }
    for (byte, state, action, next) in [
        (b'a', S::Ground, A::Print, S::Ground),
        (0x1b, S::Ground, A::Clear, S::Escape),
        (b'[', S::Escape, A::Clear, S::CsiEntry),
        (b'3', S::CsiEntry, A::Param, S::CsiParam),
        (b'm', S::CsiParam, A::CsiDispatch, S::Ground),
        (b']', S::Escape, A::OscStart, S::OscString),
        (0x07, S::OscString, A::OscEnd, S::Ground),
        (0x1b, S::DcsPassthrough, A::DcsUnhook, S::Ground),
        (0x1b, S::ApcPassthrough, A::ApcEnd, S::Ground),
    ] {
        assert_eq!(table.get(byte, state), Some(Transition::new(action, next)));
    }
}

#[test]
fn clone_and_vt500_instances_do_not_alias_and_untouched_entries_remain() {
    let original = TransitionTable::vt500();
    let mut copy = original.clone();
    let before = copy.packed().to_vec();
    let changed = Transition::new(A::Error, S::ApcEntry);
    copy.add_many(b"ab", S::Ground, changed).unwrap();
    for (index, &value) in copy.packed().iter().enumerate() {
        assert_eq!(
            value,
            if index == usize::from(b'a') || index == usize::from(b'b') {
                changed.packed()
            } else {
                before[index]
            }
        );
    }
    assert_eq!(original.packed(), before);
    assert_eq!(TransitionTable::vt500(), original);
    assert_ne!(copy, original);
}

#[test]
fn invalid_custom_writes_are_errors_without_partial_changes() {
    let mut table = TransitionTable::new(3);
    let original = table.clone();
    let transition = Transition::new(A::Print, S::Escape);
    let error = table.add(4, S::Ground, transition).unwrap_err();
    assert_eq!((error.index, error.length), (4, 3));
    assert_eq!(table, original);
    assert!(table
        .add_many(&[0, 1, 2, 3], S::Ground, transition)
        .is_err());
    assert_eq!(table, original);
    assert!(table.add(0, S::ApcPassthrough, transition).is_err());
    assert_eq!(table.get(0, S::ApcPassthrough), None);
    assert!(TransitionTable::new(0)
        .add_many(&[], S::Ground, transition)
        .is_ok());
}
