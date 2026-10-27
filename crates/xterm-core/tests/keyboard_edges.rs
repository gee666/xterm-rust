// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Additional coverage for branches absent from upstream tests.
use xterm_core::keyboard::{
    evaluate_keyboard_event, KeyboardEvent, KeyboardResult, KeyboardResultType,
};

fn evaluate(
    key_code: u32,
    key: &[u16],
    code: &[u16],
    modifiers: u8,
    options: u8,
) -> KeyboardResult {
    evaluate_keyboard_event(
        &KeyboardEvent {
            key_code,
            key,
            code,
            shift_key: modifiers & 1 != 0,
            alt_key: modifiers & 2 != 0,
            ctrl_key: modifiers & 4 != 0,
            meta_key: modifiers & 8 != 0,
        },
        options & 1 != 0,
        options & 2 != 0,
        options & 4 != 0,
    )
}

fn utf16(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}

#[test]
fn utf16_key_length_and_identity() {
    for unit in [0, 0x61, 0x130, 0xd800, 0xdfff, 0xffff] {
        assert_eq!(evaluate(65, &[unit], &[], 0, 0).key, Some(vec![unit]));
    }
    for key in [vec![], utf16("😀"), utf16("Dead"), utf16("Enter")] {
        assert_eq!(evaluate(65, &key, &[], 0, 0).key, None);
    }
}

#[test]
fn cancellation_is_requested_only_for_the_upstream_branches() {
    for (key_code, modifiers, cancel) in [
        (8, 0, false),
        (9, 0, true),
        (9, 1, false),
        (13, 0, true),
        (27, 0, true),
        (33, 1, false),
        (45, 0, false),
    ] {
        assert_eq!(evaluate(key_code, &[], &[], modifiers, 0).cancel, cancel);
    }
    let result = evaluate(255, &utf16("Dead"), &utf16("KeyN"), 2, 0);
    assert!(result.cancel);
    assert_eq!(result.key, Some(utf16("\x1bn")));
}

#[test]
fn page_and_select_actions_preserve_priority() {
    for (key_code, result_type) in [
        (33, KeyboardResultType::PageUp),
        (34, KeyboardResultType::PageDown),
    ] {
        for modifiers in (1..16).step_by(2) {
            let result = evaluate(key_code, &[], &[], modifiers, 0);
            assert_eq!(result.result_type, result_type);
            assert_eq!(result.key, None);
            assert!(!result.cancel);
        }
    }
    let result = evaluate(65, &utf16("a"), &[], 8, 2);
    assert_eq!(result.result_type, KeyboardResultType::SelectAll);
    assert_eq!(result.key, None);
    assert_eq!(
        evaluate(65, &utf16("a"), &[], 9, 2).result_type,
        KeyboardResultType::SendKey
    );
    assert_eq!(
        evaluate(65, &utf16("a"), &[], 8, 0).result_type,
        KeyboardResultType::SendKey
    );
}

#[test]
fn meta_arrows_are_not_sent_but_home_and_function_keys_are() {
    for key_code in 37..=40 {
        for modifiers in 8..16 {
            assert_eq!(evaluate(key_code, &[], &[], modifiers, 1).key, None);
        }
    }
    assert_eq!(evaluate(36, &[], &[], 8, 0).key, Some(utf16("\x1b[1;9H")));
    assert_eq!(evaluate(112, &[], &[], 8, 0).key, Some(utf16("\x1b[1;9P")));
}

#[test]
fn control_punctuation_and_ios_ctrl_c() {
    for (key_code, expected) in [
        (32, 0),
        (51, 27),
        (52, 28),
        (53, 29),
        (54, 30),
        (55, 31),
        (56, 127),
        (219, 27),
        (220, 28),
        (221, 29),
    ] {
        assert_eq!(evaluate(key_code, &[], &[], 4, 0).key, Some(vec![expected]));
    }
    assert_eq!(evaluate(191, &utf16("/"), &[], 4, 0).key, Some(vec![31]));
    // Slash is tested before the legacy 219 bracket code, as in JS.
    assert_eq!(evaluate(219, &utf16("/"), &[], 4, 0).key, Some(vec![31]));
    assert_eq!(evaluate(13, &utf16("c"), &[], 6, 0).key, Some(vec![3]));
    assert_eq!(evaluate(13, &utf16("C"), &[], 4, 0).key, Some(vec![13]));
}

#[test]
fn dead_keys_slice_one_code_unit_and_use_pinned_unicode_lowercase() {
    for (code, expected) in [
        (utf16("Keyİ"), vec![27, 0x69, 0x307]),
        (utf16("Key😀"), vec![27, 0xd83d]),
        (vec![75, 101, 121, 0xdfff], vec![27, 0xdfff]),
        (utf16("Key"), vec![27]),
        (utf16("Key꟎"), vec![27, 0xa7cf]),
    ] {
        let result = evaluate(255, &utf16("Dead"), &code, 2, 0);
        assert_eq!(result.key, Some(expected));
        assert!(result.cancel);
    }
    assert_eq!(
        evaluate(255, &utf16("Dead"), &utf16("Keyİ"), 3, 0).key,
        Some(vec![27, 0x130])
    );
    assert_eq!(
        evaluate(255, &utf16("Dead"), &utf16("KeyN"), 2, 2).key,
        None
    );
    assert_eq!(
        evaluate(255, &utf16("Dead"), &utf16("KeyN"), 2, 6).key,
        Some(utf16("\x1bn"))
    );
}

#[test]
fn numeric_boundaries_and_control_alphabet_are_exact() {
    for code in [0, 47, 48, 255, u32::MAX] {
        assert_eq!(
            evaluate(code, &[120], &[], 0, 0).key,
            (code >= 48).then(|| vec![120])
        );
    }
    for code in 64..=91 {
        let expected = if (65..=90).contains(&code) {
            Some(vec![u16::try_from(code - 64).unwrap()])
        } else {
            None
        };
        assert_eq!(evaluate(code, &[], &[], 4, 0).key, expected);
    }
    assert_eq!(evaluate(111, &utf16("F0"), &[], 0, 0).key, None);
    assert_eq!(evaluate(124, &utf16("F13"), &[], 0, 0).key, None);
    assert_eq!(evaluate(112, &[], &[], 0, 0).key, Some(utf16("\x1bOP")));
    assert_eq!(evaluate(112, &[], &[], 0, 1).key, Some(utf16("\x1bOP")));
}

#[test]
fn control_shift_punctuation_keeps_alt_and_meta_branch_precedence() {
    for (code, key, expected) in [("Minus", "_", 31), ("Digit2", "@", 0), ("Digit6", "^", 30)] {
        assert_eq!(
            evaluate(255, &utf16(key), &utf16(code), 5, 0).key,
            Some(vec![expected])
        );
        assert_eq!(evaluate(255, &[], &utf16(code), 5, 0).key, None);
        // Non-Mac Alt branch is selected before Ctrl+Shift fallback, even when
        // it cannot map this synthetic keyCode. Mac Option disabled skips it.
        assert_eq!(evaluate(255, &utf16(key), &utf16(code), 7, 0).key, None);
        assert_eq!(
            evaluate(255, &utf16(key), &utf16(code), 7, 2).key,
            Some(vec![expected])
        );
        assert_eq!(
            evaluate(255, &utf16(key), &utf16(code), 13, 2).key,
            Some(vec![expected])
        );
    }
    assert_eq!(
        evaluate(255, &utf16("x"), &utf16("Unknown"), 5, 0).key,
        None
    );
}

#[test]
fn insert_and_page_keys_ignore_only_the_documented_modifiers() {
    for modifiers in [0, 2, 8, 10] {
        assert_eq!(
            evaluate(45, &[], &[], modifiers, 0).key,
            Some(utf16("\x1b[2~"))
        );
    }
    for modifiers in [1, 4, 5, 15] {
        assert_eq!(evaluate(45, &[], &[], modifiers, 0).key, None);
    }
    assert_eq!(evaluate(33, &[], &[], 2, 0).key, Some(utf16("\x1b[5~")));
    assert_eq!(evaluate(33, &[], &[], 6, 0).key, Some(utf16("\x1b[5;7~")));
}
