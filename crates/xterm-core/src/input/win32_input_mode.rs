// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from common/input/Win32InputMode.ts.

//! Win32 protocol encoding with source-ordered, fallible event field access.
//! The fixed scan-code table intentionally uses upstream's approximate US layout.
//! Inherited object-map coercions remain tracked by tasks 184/185 and
//! docs/keyboard-map-lookup.md; this module does not establish full equivalence.

use super::keyboard_event_source::{code_point_at_zero, number_is_truthy, KeyboardEventSource};

#[path = "win32_key_codes.rs"]
mod key_codes;

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

/// Stateless encoder for `CSI Vk ; Sc ; Uc ; Kd ; Cs ; Rc _` records.
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
        let vk = if let Some(vk) = key_codes::virtual_key(&event.code()?) {
            f64::from(vk)
        } else {
            let code = event.key_code()?;
            if number_is_truthy(code) {
                code
            } else {
                0.0
            }
        };
        let sc = key_codes::scan_code(&event.code()?);
        let uc = unicode_char(event)?;
        let kd = u8::from(is_key_down);
        let cs = control_state(event)?;
        let vk = ryu_js::Buffer::new().format(vk).to_owned();
        Ok(Win32KeyboardResult {
            result_type: KeyboardResultType::SendKey,
            cancel: true,
            key: Some(
                format!("\x1b[{vk};{sc};{uc};{kd};{cs};1_")
                    .encode_utf16()
                    .collect(),
            ),
        })
    }
}

fn equals(key: &[u16], value: &str) -> bool {
    key.iter().copied().eq(value.encode_utf16())
}

fn unicode_char<E: KeyboardEventSource + ?Sized>(event: &E) -> Result<u32, E::Error> {
    if event.ctrl_key()? && !event.alt_key()? && !event.meta_key()? {
        if equals(&event.key()?, "Enter") {
            return Ok(10);
        }
        if equals(&event.key()?, "Backspace") {
            return Ok(127);
        }
    }
    let control = event.key()?;
    let code = if equals(&control, "Enter") {
        Some(13)
    } else if equals(&control, "Backspace") {
        Some(8)
    } else if equals(&control, "Tab") {
        Some(9)
    } else if equals(&control, "Escape") {
        Some(27)
    } else {
        None
    };
    if let Some(code) = code {
        return Ok(code);
    }
    if event.key()?.len() == 1 {
        let code = code_point_at_zero(&event.key()?).unwrap_or(0);
        if event.ctrl_key()? && !event.alt_key()? && !event.meta_key()? {
            if (0x41..=0x5a).contains(&code) {
                return Ok(code - 0x40);
            }
            if (0x61..=0x7a).contains(&code) {
                return Ok(code - 0x60);
            }
        }
        return Ok(code);
    }
    Ok(0)
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
