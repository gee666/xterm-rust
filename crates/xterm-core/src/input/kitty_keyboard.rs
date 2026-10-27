// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from common/input/KittyKeyboard.ts.

//! Kitty keyboard encoding policy. Input and output retain JavaScript UTF-16
//! units, including lone surrogates. Browser event dispatch is an adapter concern.

use super::keyboard::{KeyboardEvent, KeyboardResult};

mod inherited;
mod keys;

/// Enhancement flag bits accepted by [`evaluate`]. Unknown bits are preserved
/// by callers and ignored by this encoder, as in the JavaScript implementation.
pub mod flags {
    pub const NONE: u32 = 0;
    pub const DISAMBIGUATE_ESCAPE_CODES: u32 = 1;
    pub const REPORT_EVENT_TYPES: u32 = 2;
    pub const REPORT_ALTERNATE_KEYS: u32 = 4;
    pub const REPORT_ALL_KEYS_AS_ESCAPE_CODES: u32 = 8;
    pub const REPORT_ASSOCIATED_TEXT: u32 = 16;
}

/// Protocol event values. The repeat bit from a DOM event is translated by the
/// browser adapter, not inferred by this policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum EventType {
    #[default]
    Press = 1,
    Repeat = 2,
    Release = 3,
}

/// Whether the active enhancement flags enable Kitty encoding.
pub fn should_use_protocol(flags: u32) -> bool {
    flags > 0
}

/// Evaluate one event, without side effects. `cancel` is the upstream request
/// to prevent the browser default action. An absent key differs from empty text.
/// `mac_option_as_alt` unwinds Option-composed characters using the physical code.
pub fn evaluate(
    event: &KeyboardEvent<'_>,
    flags: u32,
    event_type: EventType,
    mac_option_as_alt: bool,
) -> KeyboardResult {
    let mut result = KeyboardResult::default();
    let modifier_bits = u8::from(event.shift_key)
        | (u8::from(event.alt_key) << 1)
        | (u8::from(event.ctrl_key) << 2)
        | (u8::from(event.meta_key) << 3);
    let modifiers = if modifier_bits == 0 {
        0
    } else {
        modifier_bits + 1
    };
    let is_mod = ["Shift", "Control", "Alt", "Meta"]
        .iter()
        .any(|key| keys::equals(event.key, key));
    let is_lock = ["CapsLock", "NumLock", "ScrollLock"]
        .iter()
        .any(|key| keys::equals(event.key, key));
    let report_events = flags & flags::REPORT_EVENT_TYPES != 0;
    let report_all = flags & flags::REPORT_ALL_KEYS_AS_ESCAPE_CODES != 0;
    if (!report_events && event_type == EventType::Release) || ((is_mod || is_lock) && !report_all)
    {
        return result;
    }
    if let Some((number, suffix, ss3)) = keys::legacy(event.key) {
        result.key = Some(
            legacy_sequence(number, suffix, ss3, modifiers, event_type, report_events)
                .encode_utf16()
                .collect(),
        );
        result.cancel = true;
        return result;
    }
    let Some(code) = key_code(event, mac_option_as_alt) else {
        return result;
    };
    let special_key = code == 13.0 || code == 9.0 || code == 127.0;
    if special_key && event_type == EventType::Release && !report_all {
        return result;
    }
    let is_func = keys::functional(event.key).is_some() || keys::numpad(event.code).is_some();
    let use_csi_u = report_all
        || (report_events && event_type == EventType::Release)
        || ((flags & flags::DISAMBIGUATE_ESCAPE_CODES != 0 || report_events)
            && ((is_func && !special_key)
                || (modifiers > 0 && event.key.len() != 1)
                || modifiers > 2));
    if use_csi_u {
        result.key = Some(
            csi_u(event, code, modifiers, event_type, flags, is_func, is_mod)
                .encode_utf16()
                .collect(),
        );
        result.cancel = true;
    } else if special_key {
        result.key = Some(vec![code as u16]);
    } else if event.key.len() == 1 && !event.ctrl_key && !event.alt_key && !event.meta_key {
        result.key = Some(event.key.to_vec());
    }
    result
}

fn key_code(event: &KeyboardEvent<'_>, mac_option_as_alt: bool) -> Option<f64> {
    if let Some(code) = keys::numpad(event.code)
        .or_else(|| keys::modifier(event.code).map(f64::from))
        .or_else(|| keys::functional(event.key).map(f64::from))
    {
        return Some(code);
    }
    if event.shift_key || (mac_option_as_alt && event.alt_key) {
        if event.code.len() == 6 && keys::starts_with(event.code, "Digit") {
            let digit = event.code[5];
            if (u16::from(b'0')..=u16::from(b'9')).contains(&digit) {
                return Some(f64::from(digit));
            }
        }
        if event.code.len() == 4 && keys::starts_with(event.code, "Key") {
            return Some(f64::from(super::keyboard::lowercase::lowercase_first(
                event.code[3],
            )));
        }
    }
    if event.key.len() == 1 {
        let unit = event.key[0];
        let lower = if (65..=90).contains(&unit) {
            unit + 32
        } else {
            unit
        };
        return Some(f64::from(lower));
    }
    None
}

fn legacy_sequence(
    number: u32,
    suffix: &str,
    ss3: bool,
    modifiers: u8,
    event_type: EventType,
    report_events: bool,
) -> String {
    let needs_event = report_events && event_type != EventType::Press;
    if ss3 && modifiers == 0 && !needs_event {
        return format!("\x1bO{suffix}");
    }
    let mut sequence = if number == 1 && modifiers == 0 && !needs_event {
        "\x1b[".into()
    } else {
        format!("\x1b[{number}")
    };
    if modifiers > 0 || needs_event {
        sequence.push_str(&format!(";{}", modifiers.max(1)));
        if needs_event {
            sequence.push_str(&format!(":{}", event_type as u8));
        }
    }
    sequence.push_str(suffix);
    sequence
}

fn csi_u(
    event: &KeyboardEvent<'_>,
    code: f64,
    modifiers: u8,
    event_type: EventType,
    flags: u32,
    is_func: bool,
    is_mod: bool,
) -> String {
    let mut sequence = format!("\x1b[{}", ryu_js::Buffer::new().format(code));
    if flags & flags::REPORT_ALTERNATE_KEYS != 0
        && event.shift_key
        && event.key.len() == 1
        && !is_func
        && !is_mod
    {
        sequence.push_str(&format!(":{}", event.key[0]));
    }
    let text = flags & flags::REPORT_ASSOCIATED_TEXT != 0
        && event_type != EventType::Release
        && event.key.len() == 1
        && !is_func
        && !is_mod
        && !event.ctrl_key;
    let needs_event = flags & flags::REPORT_EVENT_TYPES != 0
        && event_type != EventType::Press
        && (event_type == EventType::Release || !text);
    if modifiers > 0 || needs_event || text {
        sequence.push(';');
        if modifiers > 0 {
            sequence.push_str(&modifiers.to_string());
        } else if needs_event {
            sequence.push('1');
        }
        if needs_event {
            sequence.push_str(&format!(":{}", event_type as u8));
        }
    }
    if text {
        sequence.push_str(&format!(";{}", event.key[0]));
    }
    sequence.push('u');
    sequence
}
