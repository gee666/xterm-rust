// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT.
use xterm_core::input::keyboard::KeyboardEvent;
use xterm_core::input::kitty_keyboard::{evaluate, flags, EventType};

#[test]
fn utf16_length_is_not_unicode_scalar_count() {
    let event = KeyboardEvent {
        key: &[0xd83d, 0xde00],
        ..KeyboardEvent::default()
    };
    assert_eq!(evaluate(&event, 31, EventType::Press, false).key, None);
    let event = KeyboardEvent {
        key: &[0xd800],
        ..KeyboardEvent::default()
    };
    assert_eq!(
        evaluate(
            &event,
            flags::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
            EventType::Press,
            false
        )
        .key,
        Some("\x1b[55296u".encode_utf16().collect())
    );
    assert_eq!(
        evaluate(
            &event,
            flags::DISAMBIGUATE_ESCAPE_CODES,
            EventType::Press,
            false
        )
        .key,
        Some(vec![0xd800])
    );
}

#[test]
fn shifted_physical_key_uses_first_lowercase_utf16_unit() {
    let code: Vec<_> = "Keyİ".encode_utf16().collect();
    let event = KeyboardEvent {
        key: &[65],
        code: &code,
        shift_key: true,
        ..KeyboardEvent::default()
    };
    assert_eq!(
        evaluate(
            &event,
            flags::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
            EventType::Press,
            false
        )
        .key,
        Some("\x1b[105;2u".encode_utf16().collect())
    );
}

#[test]
fn numpad_retains_upstream_lexical_parse_int_behavior() {
    for (code, expected) in [
        ("Numpad123", "\x1b[57522u"),
        ("Numpad8foo", "\x1b[57407u"),
        ("Numpad9foo", "\x1b[97u"),
        ("Numpad0x10", "\x1b[57399u"),
        ("Numpad88888888888888888888", "\x1b[88888888888888950000u"),
    ] {
        let code: Vec<_> = code.encode_utf16().collect();
        let event = KeyboardEvent {
            key: &[97],
            code: &code,
            ..KeyboardEvent::default()
        };
        assert_eq!(
            evaluate(
                &event,
                flags::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
                EventType::Press,
                false
            )
            .key,
            Some(expected.encode_utf16().collect()),
            "{code:?}"
        );
    }
}

#[test]
fn inherited_object_names_follow_plain_object_lookup() {
    for (key, suffix) in [
        ("toString", "function toString() { [native code] }"),
        ("__proto__", "[object Object]"),
    ] {
        let key: Vec<_> = key.encode_utf16().collect();
        let event = KeyboardEvent {
            key: &key,
            ..KeyboardEvent::default()
        };
        let result = evaluate(&event, 1, EventType::Press, false);
        assert_eq!(
            result.key,
            Some(format!("\x1b[{suffix}").encode_utf16().collect())
        );
        assert!(result.cancel);
    }
}

#[test]
fn associated_text_repeat_omits_event_suffix_but_release_does_not() {
    let event = KeyboardEvent {
        key: &[97],
        ..KeyboardEvent::default()
    };
    assert_eq!(
        evaluate(&event, 31, EventType::Repeat, false).key,
        Some("\x1b[97;;97u".encode_utf16().collect())
    );
    assert_eq!(
        evaluate(&event, 31, EventType::Release, false).key,
        Some("\x1b[97;1:3u".encode_utf16().collect())
    );
}
