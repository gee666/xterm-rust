// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from common/input/Win32InputMode.ts.

//! Win32 keyboard protocol encoding, independent of the host operating system.
//!
//! Scan codes intentionally use upstream's approximate US layout, not the host
//! layout. `key` contains UTF-16 units because JS counts an astral key as two
//! units and therefore does not emit it as a single-character Unicode value.

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

// Both keyboard policies share event and result contracts. The aliases retain
// the initial Win32 API names without duplicating browser event representation.
pub use super::keyboard::{
    KeyboardEvent as Win32KeyboardEvent, KeyboardResult as Win32KeyboardResult, KeyboardResultType,
};

/// Stateless encoder for `CSI Vk ; Sc ; Uc ; Kd ; Cs ; Rc _` records.
#[derive(Clone, Copy, Debug, Default)]
pub struct Win32InputMode;

impl Win32InputMode {
    /// Encode a keydown or keyup with the original cancellation semantics.
    pub fn evaluate_keyboard_event(
        &self,
        event: &Win32KeyboardEvent<'_>,
        is_key_down: bool,
    ) -> Win32KeyboardResult {
        let vk = key_codes::virtual_key(event.code).unwrap_or(event.key_code);
        let sc = key_codes::scan_code(event.code);
        let uc = unicode_char(event);
        let kd = u8::from(is_key_down);
        let cs = control_state(event);
        Win32KeyboardResult {
            result_type: KeyboardResultType::SendKey,
            cancel: true,
            key: Some(
                format!("\x1b[{vk};{sc};{uc};{kd};{cs};1_")
                    .encode_utf16()
                    .collect(),
            ),
        }
    }
}

fn unicode_char(event: &Win32KeyboardEvent<'_>) -> u16 {
    let ctrl_only = event.ctrl_key && !event.alt_key && !event.meta_key;
    // Avoid lossy conversion: a single unpaired surrogate is a legal JS key.
    if event.key.iter().copied().eq("Enter".encode_utf16()) {
        return if ctrl_only { 0x0a } else { 0x0d };
    }
    if event.key.iter().copied().eq("Backspace".encode_utf16()) {
        return if ctrl_only { 0x7f } else { 0x08 };
    }
    if event.key.iter().copied().eq("Tab".encode_utf16()) {
        return 0x09;
    }
    if event.key.iter().copied().eq("Escape".encode_utf16()) {
        return 0x1b;
    }
    let [codepoint] = event.key else {
        return 0;
    };
    if ctrl_only {
        match codepoint {
            0x41..=0x5a => return codepoint - 0x40,
            0x61..=0x7a => return codepoint - 0x60,
            _ => {}
        }
    }
    *codepoint
}

fn control_state(event: &Win32KeyboardEvent<'_>) -> u16 {
    use control_key_state::*;
    let mut state = if event.shift_key { SHIFT_PRESSED } else { 0 };
    if event.ctrl_key {
        state |= if event.code.iter().copied().eq("ControlRight".encode_utf16()) {
            RIGHT_CTRL_PRESSED
        } else {
            LEFT_CTRL_PRESSED
        };
    }
    if event.alt_key {
        state |= if event.code.iter().copied().eq("AltRight".encode_utf16()) {
            RIGHT_ALT_PRESSED
        } else {
            LEFT_ALT_PRESSED
        };
    }
    if key_codes::is_enhanced(event.code) {
        state |= ENHANCED_KEY;
    }
    state
}

#[cfg(test)]
#[path = "win32_input_mode_tests.rs"]
mod tests;
