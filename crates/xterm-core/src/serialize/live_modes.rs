// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{AddonSerializeError, MouseTracking, SerializeModes};
use crate::{
    headless::{ModesSnapshot, MouseTrackingMode},
    terminal::TerminalState,
};

/// Read modes and the active buffer's margins after buffer serialization ends.
pub fn serialize_live_modes(state: &TerminalState) -> Result<Vec<u16>, AddonSerializeError> {
    let modes = live_modes(state)?;
    let active = state.buffers.active();
    let active = active
        .try_borrow()
        .map_err(|_| AddonSerializeError::Borrowed)?;
    let mut output = Vec::new();
    super::terminal::append_modes(&mut output, &modes);
    if active.scroll_top != 0 || active.scroll_bottom != state.rows().saturating_sub(1) {
        super::append(
            &mut output,
            &format!(
                "\x1b[{};{}r",
                active.scroll_top + 1,
                active.scroll_bottom + 1
            ),
        );
    }
    Ok(output)
}

pub(super) fn live_modes(state: &TerminalState) -> Result<SerializeModes, AddonSerializeError> {
    let modes = ModesSnapshot::from_services(&state.core, &state.mouse)
        .map_err(|_| AddonSerializeError::Borrowed)?;
    Ok(SerializeModes {
        application_cursor_keys: modes.application_cursor_keys_mode,
        application_keypad: modes.application_keypad_mode,
        bracketed_paste: modes.bracketed_paste_mode,
        insert: modes.insert_mode,
        origin: modes.origin_mode,
        reverse_wraparound: modes.reverse_wraparound_mode,
        send_focus: modes.send_focus_mode,
        wraparound: modes.wraparound_mode,
        show_cursor: modes.show_cursor,
        mouse_tracking: match modes.mouse_tracking_mode {
            MouseTrackingMode::None => MouseTracking::None,
            MouseTrackingMode::X10 => MouseTracking::X10,
            MouseTrackingMode::Vt200 => MouseTracking::Vt200,
            MouseTrackingMode::Drag => MouseTracking::Drag,
            MouseTrackingMode::Any => MouseTracking::Any,
        },
    })
}
