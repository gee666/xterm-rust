// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Key mappings from common/input/KittyKeyboard.ts.

use super::inherited::inherited_name;

pub(super) fn equals(units: &[u16], text: &str) -> bool {
    units.iter().copied().eq(text.encode_utf16())
}

pub(super) fn starts_with(units: &[u16], prefix: &str) -> bool {
    units
        .iter()
        .copied()
        .take(prefix.len())
        .eq(prefix.encode_utf16())
}

pub(super) fn modifier(code: &[u16]) -> Option<u32> {
    lookup(
        code,
        &[
            ("ShiftLeft", 57441),
            ("ShiftRight", 57447),
            ("ControlLeft", 57442),
            ("ControlRight", 57448),
            ("AltLeft", 57443),
            ("AltRight", 57449),
            ("MetaLeft", 57444),
            ("MetaRight", 57450),
        ],
    )
}

pub(super) fn functional(key: &[u16]) -> Option<u32> {
    if let Some(code) = modifier(key) {
        return Some(code);
    }
    lookup(
        key,
        &[
            ("Escape", 27),
            ("Enter", 13),
            ("Tab", 9),
            ("Backspace", 127),
            ("CapsLock", 57358),
            ("ScrollLock", 57359),
            ("NumLock", 57360),
            ("PrintScreen", 57361),
            ("Pause", 57362),
            ("ContextMenu", 57363),
            ("F13", 57376),
            ("F14", 57377),
            ("F15", 57378),
            ("F16", 57379),
            ("F17", 57380),
            ("F18", 57381),
            ("F19", 57382),
            ("F20", 57383),
            ("F21", 57384),
            ("F22", 57385),
            ("F23", 57386),
            ("F24", 57387),
            ("F25", 57388),
            ("KP_0", 57399),
            ("KP_1", 57400),
            ("KP_2", 57401),
            ("KP_3", 57402),
            ("KP_4", 57403),
            ("KP_5", 57404),
            ("KP_6", 57405),
            ("KP_7", 57406),
            ("KP_8", 57407),
            ("KP_9", 57408),
            ("KP_Decimal", 57409),
            ("KP_Divide", 57410),
            ("KP_Multiply", 57411),
            ("KP_Subtract", 57412),
            ("KP_Add", 57413),
            ("KP_Enter", 57414),
            ("KP_Equal", 57415),
            ("MediaPlayPause", 57430),
            ("MediaStop", 57432),
            ("MediaTrackNext", 57435),
            ("MediaTrackPrevious", 57436),
            ("AudioVolumeDown", 57438),
            ("AudioVolumeUp", 57439),
            ("AudioVolumeMute", 57440),
        ],
    )
}

fn lookup(units: &[u16], entries: &[(&str, u32)]) -> Option<u32> {
    entries
        .iter()
        .find_map(|(text, code)| equals(units, text).then_some(*code))
}

pub(super) fn legacy(key: &[u16]) -> Option<(u32, &'static str, bool)> {
    if let Some(suffix) = inherited_name(key) {
        return Some((1, suffix, false));
    }
    for (text, letter) in [
        ("ArrowUp", "A"),
        ("ArrowDown", "B"),
        ("ArrowRight", "C"),
        ("ArrowLeft", "D"),
        ("Home", "H"),
        ("End", "F"),
    ] {
        if equals(key, text) {
            return Some((1, letter, false));
        }
    }
    for (text, letter) in [("F1", "P"), ("F2", "Q"), ("F3", "R"), ("F4", "S")] {
        if equals(key, text) {
            return Some((1, letter, true));
        }
    }
    lookup(
        key,
        &[
            ("Insert", 2),
            ("Delete", 3),
            ("PageUp", 5),
            ("PageDown", 6),
            ("F5", 15),
            ("F6", 17),
            ("F7", 18),
            ("F8", 19),
            ("F9", 20),
            ("F10", 21),
            ("F11", 23),
            ("F12", 24),
        ],
    )
    .map(|code| (code, "~", false))
}

pub(super) fn numpad(code: &[u16]) -> Option<f64> {
    if !starts_with(code, "Numpad") {
        return None;
    }
    let suffix = &code[6..];
    // Upstream uses lexical string comparisons followed by parseInt rather
    // than checking a single digit. Retain even unusual synthetic DOM codes.
    if suffix >= [u16::from(b'0')].as_slice() && suffix <= [u16::from(b'9')].as_slice() {
        let digit_count = suffix.iter().take_while(|u| (48..=57).contains(*u)).count();
        let digits = String::from_utf16_lossy(&suffix[..digit_count]);
        // This branch guarantees at least one decimal digit. Overflow is
        // Infinity in JavaScript and in Rust's f64 parser.
        let value = digits
            .parse::<f64>()
            .expect("nonempty ASCII decimal digits parse as f64");
        return Some(57399.0 + value);
    }
    lookup(
        suffix,
        &[
            ("Decimal", 57409),
            ("Divide", 57410),
            ("Multiply", 57411),
            ("Subtract", 57412),
            ("Add", 57413),
            ("Enter", 57414),
            ("Equal", 57415),
        ],
    )
    .map(f64::from)
}
