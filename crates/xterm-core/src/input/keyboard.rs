// Copyright (c) 2014 The xterm.js authors. All rights reserved.
// Copyright (c) 2012-2013, Christopher Jeffrey (MIT License)
// Licensed under MIT. Ported from common/input/Keyboard.ts.

//! Legacy keyboard policy, independent of DOM and operating-system APIs.
//!
//! Strings use UTF-16 units so a lone surrogate remains a single JS character.
//! The event's `type` is omitted because upstream never reads it here. Key codes
//! use the DOM unsigned-long range. The caller performs any WebIDL conversion.

#[path = "keyboard_lowercase.rs"]
pub(crate) mod lowercase;

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

/// Evaluate one event without producing side effects or suppressing the event.
///
/// `cancel` is upstream's cancellation request, not an acknowledgement that
/// the browser event was canceled. Page and select actions have no key payload.
pub fn evaluate_keyboard_event(
    event: &KeyboardEvent<'_>,
    application_cursor_mode: bool,
    is_mac: bool,
    mac_option_is_meta: bool,
) -> KeyboardResult {
    let mut result = KeyboardResult::default();
    let modifiers = u8::from(event.shift_key)
        | (u8::from(event.alt_key) << 1)
        | (u8::from(event.ctrl_key) << 2)
        | (u8::from(event.meta_key) << 3);
    let key = match event.key_code {
        0 => mobile_arrow(event.key).map(|letter| cursor_key(letter, 0, application_cursor_mode)),
        8 => Some(with_alt(
            if event.ctrl_key { 8 } else { DEL },
            event.alt_key,
        )),
        9 if event.shift_key => Some(utf16("\x1b[Z")),
        9 => {
            result.cancel = true;
            Some(vec![9])
        }
        13 => {
            result.cancel = true;
            // iOS hardware Ctrl+C may report keyCode 13 instead of 67.
            Some(if equals(event.key, "c") && event.ctrl_key {
                vec![3]
            } else {
                with_alt(13, event.alt_key)
            })
        }
        27 => {
            result.cancel = true;
            Some(with_alt(ESC, event.alt_key))
        }
        37..=40 if event.meta_key => None,
        37..=40 => {
            let letter = match event.key_code {
                37 => 'D',
                38 => 'A',
                39 => 'C',
                _ => 'B',
            };
            Some(cursor_key(letter, modifiers, application_cursor_mode))
        }
        45 if !event.shift_key && !event.ctrl_key => Some(utf16("\x1b[2~")),
        45 => None,
        46 => Some(tilde_key(3, modifiers)),
        35 | 36 => Some(cursor_key(
            if event.key_code == 35 { 'F' } else { 'H' },
            modifiers,
            application_cursor_mode,
        )),
        33 | 34 => {
            if event.shift_key {
                result.result_type = if event.key_code == 33 {
                    KeyboardResultType::PageUp
                } else {
                    KeyboardResultType::PageDown
                };
                None
            } else {
                // Alt and Meta do not modify page keys unless Control is set.
                Some(tilde_key(
                    if event.key_code == 33 { 5 } else { 6 },
                    if event.ctrl_key { modifiers } else { 0 },
                ))
            }
        }
        112..=115 => {
            let letter = match event.key_code {
                112 => 'P',
                113 => 'Q',
                114 => 'R',
                _ => 'S',
            };
            // Function keys always use SS3 when unmodified, independent of mode.
            Some(cursor_key(letter, modifiers, true))
        }
        116..=123 => {
            let number = match event.key_code {
                116 => 15,
                117 => 17,
                118 => 18,
                119 => 19,
                120 => 20,
                121 => 21,
                122 => 23,
                _ => 24,
            };
            Some(tilde_key(number, modifiers))
        }
        _ => default_key(event, is_mac, mac_option_is_meta, &mut result),
    };
    result.key = key;
    result
}

fn default_key(
    event: &KeyboardEvent<'_>,
    is_mac: bool,
    mac_option_is_meta: bool,
    result: &mut KeyboardResult,
) -> Option<Vec<u16>> {
    if event.ctrl_key && !event.shift_key && !event.alt_key && !event.meta_key {
        let unit = match event.key_code {
            65..=90 => u16::try_from(event.key_code - 64).ok()?,
            32 => NUL,
            51..=55 => u16::try_from(event.key_code - 51 + u32::from(ESC)).ok()?,
            56 => DEL,
            _ if equals(event.key, "/") => US,
            219 => ESC,
            220 => 0x1c,
            221 => 0x1d,
            _ => return None,
        };
        Some(vec![unit])
    } else if (!is_mac || mac_option_is_meta) && event.alt_key && !event.meta_key {
        alt_key(event, result)
    } else if is_mac && !event.alt_key && !event.ctrl_key && !event.shift_key && event.meta_key {
        if event.key_code == 65 {
            result.result_type = KeyboardResultType::SelectAll;
        }
        None
    } else if !event.ctrl_key
        && !event.alt_key
        && !event.meta_key
        && event.key_code >= 48
        && event.key.len() == 1
    {
        Some(event.key.to_vec())
    } else if !event.key.is_empty() && event.ctrl_key && event.shift_key {
        let unit = if equals(event.code, "Minus") {
            US
        } else if equals(event.code, "Digit2") {
            NUL
        } else if equals(event.code, "Digit6") {
            RS
        } else {
            return None;
        };
        Some(vec![unit])
    } else {
        None
    }
}

fn alt_key(event: &KeyboardEvent<'_>, result: &mut KeyboardResult) -> Option<Vec<u16>> {
    if let Some((plain, shifted)) = key_mapping(event.key_code) {
        return Some(vec![
            ESC,
            u16::from(if event.shift_key { shifted } else { plain }),
        ]);
    }
    match event.key_code {
        65..=90 => {
            let code = if event.ctrl_key {
                event.key_code - 64
            } else if event.shift_key {
                event.key_code
            } else {
                event.key_code + 32
            };
            Some(vec![ESC, u16::try_from(code).ok()?])
        }
        32 => Some(vec![ESC, if event.ctrl_key { NUL } else { 32 }]),
        _ if equals(event.key, "Dead") && event.code.starts_with(&[75, 101, 121]) => {
            let mut key = vec![ESC];
            if let Some(&unit) = event.code.get(3) {
                // JS slice selects one UTF-16 unit, not a Unicode scalar.
                if event.shift_key {
                    key.push(unit);
                } else {
                    lowercase::append_lowercase(&mut key, unit);
                }
            }
            result.cancel = true;
            Some(key)
        }
        _ => None,
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

fn mobile_arrow(key: &[u16]) -> Option<char> {
    [
        ("UIKeyInputUpArrow", 'A'),
        ("UIKeyInputLeftArrow", 'D'),
        ("UIKeyInputRightArrow", 'C'),
        ("UIKeyInputDownArrow", 'B'),
    ]
    .into_iter()
    .find_map(|(name, letter)| equals(key, name).then_some(letter))
}

fn equals(units: &[u16], value: &str) -> bool {
    units.iter().copied().eq(value.encode_utf16())
}

fn utf16(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}

fn key_mapping(code: u32) -> Option<(u8, u8)> {
    Some(match code {
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
