// Copyright (c) 2014 The xterm.js authors. All rights reserved.
// Copyright (c) 2012-2013, Christopher Jeffrey (MIT License)
// Licensed under MIT. Live field reads follow common/input/Keyboard.ts.
use super::super::keyboard_event_source::{to_uint16, KeyboardEventSource};
use super::{
    cursor_key, equals, special_key, tilde_key, utf16, with_alt, KeyboardResult,
    KeyboardResultType, DEL, ESC,
};
#[path = "keyboard_default.rs"]
mod default;

/// Evaluate without snapshotting a potentially mutable or throwing event.
///
/// Every getter is called at the corresponding source expression. Errors are
/// returned unchanged before subsequent getters or branch effects are executed.
pub fn try_evaluate_keyboard_event<E: KeyboardEventSource + ?Sized>(
    event: &E,
    application_cursor_mode: bool,
    is_mac: bool,
    mac_option_is_meta: bool,
) -> Result<KeyboardResult, E::Error> {
    let mut result = KeyboardResult::default();
    let modifiers = u8::from(event.shift_key()?)
        | (u8::from(event.alt_key()?) << 1)
        | (u8::from(event.ctrl_key()?) << 2)
        | (u8::from(event.meta_key()?) << 3);
    // The switch reads keyCode once; the default branch deliberately reads it
    // again at each expression, since a getter may change between calls.
    let key = match special_key(event.key_code()?) {
        Some(0) => {
            mobile_arrow(event)?.map(|letter| cursor_key(letter, 0, application_cursor_mode))
        }
        Some(8) => Some(with_alt(
            if event.ctrl_key()? { 8 } else { DEL },
            event.alt_key()?,
        )),
        Some(9) => {
            if event.shift_key()? {
                Some(utf16("\x1b[Z"))
            } else {
                result.cancel = true;
                Some(vec![9])
            }
        }
        Some(13) => {
            let key = if equals(&event.key()?, "c") && event.ctrl_key()? {
                vec![3]
            } else {
                with_alt(13, event.alt_key()?)
            };
            result.cancel = true;
            Some(key)
        }
        Some(27) => {
            let key = with_alt(ESC, event.alt_key()?);
            result.cancel = true;
            Some(key)
        }
        Some(code @ 37..=40) => {
            if event.meta_key()? {
                None
            } else {
                let letter = match code {
                    37 => 'D',
                    38 => 'A',
                    39 => 'C',
                    _ => 'B',
                };
                Some(cursor_key(letter, modifiers, application_cursor_mode))
            }
        }
        Some(45) => {
            if !event.shift_key()? && !event.ctrl_key()? {
                Some(utf16("\x1b[2~"))
            } else {
                None
            }
        }
        Some(46) => Some(tilde_key(3, modifiers)),
        Some(code @ (35 | 36)) => Some(cursor_key(
            if code == 35 { 'F' } else { 'H' },
            modifiers,
            application_cursor_mode,
        )),
        Some(code @ (33 | 34)) => {
            if event.shift_key()? {
                result.result_type = if code == 33 {
                    KeyboardResultType::PageUp
                } else {
                    KeyboardResultType::PageDown
                };
                None
            } else if event.ctrl_key()? {
                Some(tilde_key(if code == 33 { 5 } else { 6 }, modifiers))
            } else {
                Some(tilde_key(if code == 33 { 5 } else { 6 }, 0))
            }
        }
        Some(code @ 112..=115) => Some(cursor_key(char::from(b'P' + code - 112), modifiers, true)),
        Some(code @ 116..=123) => {
            let number = match code {
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
        _ => default::default_key(event, is_mac, mac_option_is_meta, &mut result)?,
    };
    result.key = key;
    Ok(result)
}
fn mobile_arrow<E: KeyboardEventSource + ?Sized>(event: &E) -> Result<Option<char>, E::Error> {
    for (name, letter) in [
        ("UIKeyInputUpArrow", 'A'),
        ("UIKeyInputLeftArrow", 'D'),
        ("UIKeyInputRightArrow", 'C'),
        ("UIKeyInputDownArrow", 'B'),
    ] {
        if equals(&event.key()?, name) {
            return Ok(Some(letter));
        }
    }
    Ok(None)
}
