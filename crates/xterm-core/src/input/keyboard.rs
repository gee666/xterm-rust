// Copyright (c) 2014 The xterm.js authors. All rights reserved.
// Copyright (c) 2012-2013, Christopher Jeffrey (MIT License)
// Licensed under MIT. Ported from common/input/Keyboard.ts.

//! Legacy keyboard policy independent of DOM and operating-system APIs.
//! Native events borrow UTF-16 units; live browser adapters implement the
//! fallible sibling KeyboardEventSource contract without eager snapshots.

#[path = "keyboard_lowercase.rs"]
pub(crate) mod lowercase;
#[path = "keyboard_policy.rs"]
mod policy;
#[path = "keyboard_uppercase.rs"]
mod uppercase;
pub use policy::try_evaluate_keyboard_event;

/// Browser keyboard data used by the legacy keyboard policy.
#[derive(Clone, Copy, Debug, Default)]
pub struct KeyboardEvent<'a> {
    pub key_code: u32,
    pub key: &'a [u16],
    pub code: &'a [u16],
    pub shift_key: bool,
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub meta_key: bool,
}

/// Numeric discriminants match `KeyboardResultType` in common/Types.ts.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum KeyboardResultType {
    #[default]
    SendKey = 0,
    SelectAll = 1,
    PageUp = 2,
    PageDown = 3,
}

/// Result of legacy policy. `None` differs from an empty outgoing string.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KeyboardResult {
    pub result_type: KeyboardResultType,
    pub cancel: bool,
    pub key: Option<Vec<u16>>,
}

const ESC: u16 = 0x1b;
const DEL: u16 = 0x7f;
const NUL: u16 = 0;
const US: u16 = 0x1f;
const RS: u16 = 0x1e;

/// Infallible borrowed-event entrypoint. Live or throwing event properties use
/// try_evaluate_keyboard_event and retain their source expression read order.
pub fn evaluate_keyboard_event(
    event: &KeyboardEvent<'_>,
    application_cursor_mode: bool,
    is_mac: bool,
    mac_option_is_meta: bool,
) -> KeyboardResult {
    match try_evaluate_keyboard_event(event, application_cursor_mode, is_mac, mac_option_is_meta) {
        Ok(result) => result,
        Err(infallible) => match infallible {},
    }
}

fn special_key(code: f64) -> Option<u8> {
    if (0.0..=255.0).contains(&code) && code.fract() == 0.0 {
        // Only bounded integral values enter the byte-key switch. Defaults keep
        // the full f64 getter value and perform JS ToUint16 where required.
        Some(code as u8)
    } else {
        None
    }
}

fn cursor_key(letter: char, modifiers: u8, application_mode: bool) -> Vec<u16> {
    utf16(&if modifiers != 0 {
        format!("\x1b[1;{}{letter}", modifiers + 1)
    } else {
        format!("\x1b{}{letter}", if application_mode { 'O' } else { '[' })
    })
}

fn tilde_key(number: u8, modifiers: u8) -> Vec<u16> {
    utf16(&if modifiers == 0 {
        format!("\x1b[{number}~")
    } else {
        format!("\x1b[{number};{}~", modifiers + 1)
    })
}

fn with_alt(unit: u16, alt: bool) -> Vec<u16> {
    if alt {
        vec![ESC, unit]
    } else {
        vec![unit]
    }
}

fn equals(units: &[u16], value: &str) -> bool {
    units.iter().copied().eq(value.encode_utf16())
}

fn utf16(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}

fn key_mapping(code: f64) -> Option<(u8, u8)> {
    Some(match special_key(code)? {
        48 => (b'0', b')'),
        49 => (b'1', b'!'),
        50 => (b'2', b'@'),
        51 => (b'3', b'#'),
        52 => (b'4', b'$'),
        53 => (b'5', b'%'),
        54 => (b'6', b'^'),
        55 => (b'7', b'&'),
        56 => (b'8', b'*'),
        57 => (b'9', b'('),
        186 => (b';', b':'),
        187 => (b'=', b'+'),
        188 => (b',', b'<'),
        189 => (b'-', b'_'),
        190 => (b'.', b'>'),
        191 => (b'/', b'?'),
        192 => (b'`', b'~'),
        219 => (b'[', b'{'),
        220 => (b'\\', b'|'),
        221 => (b']', b'}'),
        222 => (b'\'', b'"'),
        _ => return None,
    })
}
