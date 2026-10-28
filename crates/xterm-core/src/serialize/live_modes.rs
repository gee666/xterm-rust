use super::{MouseTracking, SerializeModes};
use crate::headless::{ModesSnapshot, MouseTrackingMode};

impl From<&ModesSnapshot> for SerializeModes {
    fn from(modes: &ModesSnapshot) -> Self {
        Self {
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
        }
    }
}
