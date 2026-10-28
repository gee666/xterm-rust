// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from common/input/KittyKeyboard.ts.

//! Kitty keyboard encoding policy. Input and output retain JavaScript UTF-16
//! units, including lone surrogates. Browser event dispatch is an adapter concern.

use super::keyboard::{KeyboardEvent, KeyboardResult};

mod inherited;
mod keys;
mod live;
mod live_fields;

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
    match try_evaluate(event, flags, event_type, mac_option_as_alt) {
        Ok(result) => result,
        Err(never) => match never {},
    }
}

/// Evaluate a live event without reading any field before its source expression.
/// Getter errors propagate without later reads or output mutations.
pub fn try_evaluate<E: crate::input::keyboard_event_source::KeyboardEventSource + ?Sized>(
    event: &E,
    flags: u32,
    event_type: EventType,
    mac_option_as_alt: bool,
) -> Result<KeyboardResult, E::Error> {
    live::evaluate(event, flags, event_type, mac_option_as_alt)
}

fn legacy_sequence(
    number: &str,
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
    let mut sequence = if number == "1" && modifiers == 0 && !needs_event {
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
