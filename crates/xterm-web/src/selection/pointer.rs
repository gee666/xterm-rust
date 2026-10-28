// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Pure selection pointer policies shared by native assertions and DOM listeners.
use super::service::SelectionPlatform;

pub const DRAG_SCROLL_MAX_THRESHOLD: f64 = 50.0;
pub const DRAG_SCROLL_MAX_SPEED: f64 = 15.0;
pub const DRAG_SCROLL_INTERVAL_MS: i32 = 50;
pub const ALT_CLICK_MOVE_CURSOR_TIME_MS: f64 = 500.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct Modifiers {
    pub alt: bool,
    pub shift: bool,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct SelectionMouseOptions {
    pub mouse_events_require_alt: bool,
    pub mac_option_click_forces_selection: bool,
    pub mouse_events_active: bool,
}
pub fn should_force_selection(
    platform: SelectionPlatform,
    options: SelectionMouseOptions,
    modifiers: Modifiers,
) -> bool {
    if options.mouse_events_require_alt && options.mouse_events_active {
        return !modifiers.alt;
    }
    if platform.mac {
        return modifiers.alt && options.mac_option_click_forces_selection;
    }
    modifiers.shift
}
pub fn should_column_select(
    platform: SelectionPlatform,
    options: SelectionMouseOptions,
    modifiers: Modifiers,
) -> bool {
    if options.mouse_events_require_alt && options.mouse_events_active {
        return false;
    }
    modifiers.alt && !(platform.mac && options.mac_option_click_forces_selection)
}
pub fn drag_scroll_amount(mut offset_y: f64, terminal_height: f64) -> f64 {
    if offset_y >= 0.0 && offset_y <= terminal_height {
        return 0.0;
    }
    if offset_y > terminal_height {
        offset_y -= terminal_height;
    }
    offset_y = offset_y.clamp(-DRAG_SCROLL_MAX_THRESHOLD, DRAG_SCROLL_MAX_THRESHOLD)
        / DRAG_SCROLL_MAX_THRESHOLD;
    offset_y.signum() + (offset_y * (DRAG_SCROLL_MAX_SPEED - 1.0) + 0.5).floor()
}
