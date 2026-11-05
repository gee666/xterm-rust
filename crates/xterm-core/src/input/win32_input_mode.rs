// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from common/input/Win32InputMode.ts.

//! Win32 protocol encoding with source-ordered, fallible event field access.
//! Object-map results retain their raw identity until final string interpolation.
//! Browser event sources provide realm-specific ordinary objects and coercion.

use super::keyboard_event_source::{code_point_at_zero, number_is_truthy, KeyboardEventSource};

#[path = "win32_key_codes.rs"]
mod key_codes;
#[path = "win32_lookup.rs"]
pub mod lookup;

use lookup::{Hint, Table, Value};

/// Windows console control-state flags used by the Win32 input protocol.
pub mod control_key_state {
    pub const RIGHT_ALT_PRESSED: u16 = 1;
    pub const LEFT_ALT_PRESSED: u16 = 2;
    pub const RIGHT_CTRL_PRESSED: u16 = 4;
    pub const LEFT_CTRL_PRESSED: u16 = 8;
    pub const SHIFT_PRESSED: u16 = 16;
    pub const NUMLOCK_ON: u16 = 32;
    pub const SCROLLLOCK_ON: u16 = 64;
    pub const CAPSLOCK_ON: u16 = 128;
    pub const ENHANCED_KEY: u16 = 256;
}

pub use super::keyboard::{
    KeyboardEvent as Win32KeyboardEvent, KeyboardResult as Win32KeyboardResult, KeyboardResultType,
};

/// Stateless encoding policy. Stateful ordinary-object lookups belong to the
/// event source's runtime, not to the platform-independent policy.
#[derive(Clone, Copy, Debug, Default)]
pub struct Win32InputMode;

impl Win32InputMode {
    /// Borrowed native events cannot throw; use the same live-field policy.
    pub fn evaluate_keyboard_event(
        &self,
        event: &Win32KeyboardEvent<'_>,
        is_key_down: bool,
    ) -> Win32KeyboardResult {
        match self.try_evaluate_keyboard_event(event, is_key_down) {
            Ok(result) => result,
            Err(never) => match never {},
        }
    }

    /// Read fields at their source expressions, preserving getter failures and
    /// repeated reads instead of caching an event snapshot before callbacks.
    pub fn try_evaluate_keyboard_event<E: KeyboardEventSource + ?Sized>(
        &self,
        event: &E,
        is_key_down: bool,
    ) -> Result<Win32KeyboardResult, E::Error> {
        let vk = event.win32_lookup(Table::VirtualKey, &event.code()?)?;
        let vk = if vk.is_undefined() {
            let code = event.key_code()?;
            Value::Number(if number_is_truthy(code) { code } else { 0.0 })
        } else {
            vk
        };
        let sc = event.win32_lookup(Table::ScanCode, &event.code()?)?;
        let sc = if sc.is_truthy() {
            sc
        } else {
            Value::Number(0.0)
        };
        let uc = unicode_char(event)?;
        let kd = u8::from(is_key_down);
        let cs = control_state(event)?;
        // User coercion observes later field reads and throws in Vk/Sc/Uc order.
        let mut key = vec![0x1b, u16::from(b'[')];
        vk.append(&mut key, Hint::String)?;
        key.push(u16::from(b';'));
        sc.append(&mut key, Hint::String)?;
        key.push(u16::from(b';'));
        uc.append(&mut key, Hint::String)?;
        key.extend(format!(";{kd};{cs};1_").encode_utf16());
        Ok(Win32KeyboardResult {
            result_type: KeyboardResultType::SendKey,
            cancel: true,
            key: Some(key),
        })
    }
}

fn equals(key: &[u16], value: &str) -> bool {
    key.iter().copied().eq(value.encode_utf16())
}

fn unicode_char<E: KeyboardEventSource + ?Sized>(event: &E) -> Result<Value<E::Error>, E::Error> {
    if event.ctrl_key()? && !event.alt_key()? && !event.meta_key()? {
        if equals(&event.key()?, "Enter") {
            return Ok(Value::Number(10.0));
        }
        if equals(&event.key()?, "Backspace") {
            return Ok(Value::Number(127.0));
        }
    }
    let control = event.win32_lookup(Table::ControlChar, &event.key()?)?;
    if !control.is_undefined() {
        return Ok(control);
    }
    if event.key()?.len() == 1 {
        let code = code_point_at_zero(&event.key()?).unwrap_or(0);
        if event.ctrl_key()? && !event.alt_key()? && !event.meta_key()? {
            if (0x41..=0x5a).contains(&code) {
                return Ok(Value::Number(f64::from(code - 0x40)));
            }
            if (0x61..=0x7a).contains(&code) {
                return Ok(Value::Number(f64::from(code - 0x60)));
            }
        }
        return Ok(Value::Number(f64::from(code)));
    }
    Ok(Value::Number(0.0))
}

fn control_state<E: KeyboardEventSource + ?Sized>(event: &E) -> Result<u16, E::Error> {
    use control_key_state::*;
    let mut state = if event.shift_key()? { SHIFT_PRESSED } else { 0 };
    if event.ctrl_key()? {
        state |= if equals(&event.code()?, "ControlRight") {
            RIGHT_CTRL_PRESSED
        } else {
            LEFT_CTRL_PRESSED
        };
    }
    if event.alt_key()? {
        state |= if equals(&event.code()?, "AltRight") {
            RIGHT_ALT_PRESSED
        } else {
            LEFT_ALT_PRESSED
        };
    }
    if key_codes::is_enhanced(&event.code()?) {
        state |= ENHANCED_KEY;
    }
    Ok(state)
}

#[cfg(test)]
#[path = "win32_input_mode_tests.rs"]
mod tests;
