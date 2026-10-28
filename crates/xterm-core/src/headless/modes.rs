// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Adapted from headless/public/Terminal.ts.

use crate::{core_service::CoreService, mouse_state::MouseStateService};
use std::cell::BorrowError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseTrackingMode {
    None,
    X10,
    Vt200,
    Drag,
    Any,
}
impl MouseTrackingMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::X10 => "x10",
            Self::Vt200 => "vt200",
            Self::Drag => "drag",
            Self::Any => "any",
        }
    }
}

/// Owned public snapshot. Changing it cannot change the engine, and changing
/// modes later cannot rewrite an earlier observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModesSnapshot {
    pub application_cursor_keys_mode: bool,
    pub application_keypad_mode: bool,
    pub bracketed_paste_mode: bool,
    pub insert_mode: bool,
    pub mouse_tracking_mode: MouseTrackingMode,
    pub origin_mode: bool,
    pub reverse_wraparound_mode: bool,
    pub send_focus_mode: bool,
    pub show_cursor: bool,
    pub synchronized_output_mode: bool,
    pub win32_input_mode: bool,
    pub wraparound_mode: bool,
}
impl ModesSnapshot {
    pub fn from_services<W>(
        core: &CoreService,
        mouse: &MouseStateService<W>,
    ) -> Result<Self, BorrowError> {
        let modes = core.modes.try_borrow()?;
        let private = core.dec_private_modes.try_borrow()?;
        let mouse_tracking_mode = match mouse.active_protocol().as_str() {
            "X10" => MouseTrackingMode::X10,
            "VT200" => MouseTrackingMode::Vt200,
            "DRAG" => MouseTrackingMode::Drag,
            "ANY" => MouseTrackingMode::Any,
            _ => MouseTrackingMode::None,
        };
        Ok(Self {
            application_cursor_keys_mode: private.application_cursor_keys,
            application_keypad_mode: private.application_keypad,
            bracketed_paste_mode: private.bracketed_paste_mode,
            insert_mode: modes.insert_mode,
            mouse_tracking_mode,
            origin_mode: private.origin,
            reverse_wraparound_mode: private.reverse_wraparound,
            send_focus_mode: private.send_focus,
            show_cursor: !core.is_cursor_hidden.get(),
            synchronized_output_mode: private.synchronized_output,
            win32_input_mode: private.win32_input_mode,
            wraparound_mode: private.wraparound,
        })
    }
}
