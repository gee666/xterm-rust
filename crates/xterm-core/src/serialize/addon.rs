// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{
    scrollback_start, serialize_html, serialize_terminal, CellRange, HtmlConfig, HtmlFontSize,
    HtmlOptions, HtmlRange, MouseTracking, SerializeError, SerializeModes, SerializeOptions,
    SerializeState,
};
use crate::{
    headless::{HeadlessError, ModesSnapshot, MouseTrackingMode, Terminal},
    options::{OptionError, OptionValue},
    public_api::{
        addon_manager::TerminalAddon,
        validation::{verify_proposed_api, ApiError},
    },
    terminal::{CoreTerminal, TerminalState},
};
use std::{
    cell::RefCell,
    fmt,
    rc::{Rc, Weak},
};

#[derive(Debug)]
pub enum AddonSerializeError {
    NotLoaded,
    TerminalReleased,
    Borrowed,
    SelectionUnavailable,
    Options(OptionError),
    Api(ApiError),
    Serialize(SerializeError),
}
impl fmt::Display for AddonSerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotLoaded => f.write_str("Cannot use addon until it has been loaded"),
            Self::TerminalReleased => f.write_str("terminal state is no longer available"),
            Self::Borrowed => f.write_str("terminal is already borrowed"),
            Self::SelectionUnavailable => {
                f.write_str("this._terminal?.getSelectionPosition is not a function")
            }
            Self::Options(error) => error.fmt(f),
            Self::Api(error) => error.fmt(f),
            Self::Serialize(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for AddonSerializeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Options(error) => Some(error),
            Self::Api(error) => Some(error),
            Self::Serialize(error) => Some(error),
            Self::NotLoaded
            | Self::TerminalReleased
            | Self::Borrowed
            | Self::SelectionUnavailable => None,
        }
    }
}
impl From<SerializeError> for AddonSerializeError {
    fn from(value: SerializeError) -> Self {
        Self::Serialize(value)
    }
}
impl From<OptionError> for AddonSerializeError {
    fn from(value: OptionError) -> Self {
        Self::Options(value)
    }
}
impl From<ApiError> for AddonSerializeError {
    fn from(value: ApiError) -> Self {
        Self::Api(value)
    }
}

/// The Rust host owns state lifetime. A weak link prevents cycles when option
/// getters or event callbacks retain this addon. Explicit addon disposal still
/// permits serialization while the host state exists; dropping the Rust host
/// can produce TerminalReleased, an explicit native ownership boundary.
#[derive(Default)]
pub struct SerializeAddon {
    state: RefCell<Option<Weak<RefCell<TerminalState>>>>,
}
impl SerializeAddon {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn activate_engine(&self, terminal: &CoreTerminal) -> Result<(), AddonSerializeError> {
        *self
            .state
            .try_borrow_mut()
            .map_err(|_| AddonSerializeError::Borrowed)? =
            Some(Rc::downgrade(&terminal.state_handle()));
        Ok(())
    }
    pub fn serialize(&self, options: &SerializeOptions) -> Result<Vec<u16>, AddonSerializeError> {
        let handle = self.current_state()?;
        let state = handle
            .try_borrow()
            .map_err(|_| AddonSerializeError::Borrowed)?;
        serialize_live_state(&state, options)
    }
    pub fn serialize_as_html(
        &self,
        options: &HtmlOptions,
    ) -> Result<Vec<u16>, AddonSerializeError> {
        let handle = self.current_state()?;
        // Theme property getters and option coercions can call back into the
        // terminal. Release its state borrow before invoking those host hooks.
        let (terminal_options, active, cols, rows) = {
            let state = handle
                .try_borrow()
                .map_err(|_| AddonSerializeError::Borrowed)?;
            (
                Rc::clone(&state.options),
                state.buffers.active(),
                state.cols(),
                state.rows(),
            )
        };
        verify_proposed_api(terminal_options.get("allowProposedApi")?.is_truthy())?;
        if options.only_selection && options.range.is_none() {
            return Err(AddonSerializeError::SelectionUnavailable);
        }
        // BaseSerializeHandler receives its buffer and numeric range before
        // header getters run. Retain those boundaries even if a getter resizes
        // or switches buffers, without retaining a mutable resource borrow.
        let mut effective = options.clone();
        if effective.range.is_none() {
            let length = active
                .try_borrow()
                .map_err(|_| AddonSerializeError::Borrowed)?
                .lines
                .len();
            let start = scrollback_start(length, rows, options.scrollback);
            effective.range = Some(if start < length {
                HtmlRange {
                    start_line: start,
                    end_line: length - 1,
                    start_col: 0,
                }
            } else {
                HtmlRange {
                    start_line: 1,
                    end_line: 0,
                    start_col: 0,
                }
            });
        }
        let (foreground, background) = if options.include_global_background {
            // Source re-reads theme for each property and defers conversion
            // until both getters complete. A foreground getter may replace it.
            let foreground = theme_value(&terminal_options.get("theme")?, "foreground")?;
            let background = theme_value(&terminal_options.get("theme")?, "background")?;
            (foreground, background)
        } else {
            (None, None)
        };
        let foreground = foreground.map(|value| value.concat_utf16()).transpose()?;
        let background = background.map(|value| value.concat_utf16()).transpose()?;
        let config = HtmlConfig {
            foreground,
            background,
            font_family: terminal_options.get("fontFamily")?.concat_utf16()?,
            font_size: HtmlFontSize::Text(terminal_options.get("fontSize")?.concat_utf16()?),
            ansi_colors: None,
        };
        let buffer = active
            .try_borrow()
            .map_err(|_| AddonSerializeError::Borrowed)?;
        Ok(serialize_html(
            &*buffer, cols, rows, None, &config, &effective,
        )?)
    }
    fn current_state(&self) -> Result<Rc<RefCell<TerminalState>>, AddonSerializeError> {
        self.state
            .try_borrow()
            .map_err(|_| AddonSerializeError::Borrowed)?
            .as_ref()
            .ok_or(AddonSerializeError::NotLoaded)?
            .upgrade()
            .ok_or(AddonSerializeError::TerminalReleased)
    }
}

fn theme_value(theme: &OptionValue, key: &str) -> Result<Option<OptionValue>, OptionError> {
    if matches!(theme, OptionValue::Null | OptionValue::Undefined) {
        return Ok(None);
    }
    let value = theme.get_property(key)?;
    if matches!(value, OptionValue::Null | OptionValue::Undefined) {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

/// Browser hosts supply their actual selection and parsed theme palette. Native
/// headless hosts use the default palette and have no selection API.
pub fn serialize_live_html(
    state: &TerminalState,
    options: &HtmlOptions,
    selection: Option<CellRange>,
    config: &HtmlConfig,
) -> Result<Vec<u16>, AddonSerializeError> {
    verify_proposed_api(state.options.get("allowProposedApi")?.is_truthy())?;
    let active = state.buffers.active();
    let active = active
        .try_borrow()
        .map_err(|_| AddonSerializeError::Borrowed)?;
    Ok(serialize_html(
        &*active,
        state.cols(),
        state.rows(),
        selection,
        config,
        options,
    )?)
}
impl TerminalAddon<Terminal> for SerializeAddon {
    type Error = HeadlessError;
    fn activate(&self, terminal: &Terminal) -> Result<(), HeadlessError> {
        *self
            .state
            .try_borrow_mut()
            .map_err(|_| HeadlessError::Borrowed)? =
            Some(Rc::downgrade(&terminal.state_handle_for_addon()));
        Ok(())
    }
    fn dispose(&self) -> Result<(), HeadlessError> {
        // Upstream dispose intentionally retains the activated terminal and has
        // no registrations to remove. Manager disposal removes its own entry.
        Ok(())
    }
}

/// Borrow current buffers and modes for one call; no cached terminal snapshots
/// can become stale after writes, resize, reset or alternate-buffer activation.
pub fn serialize_live_state(
    state: &TerminalState,
    options: &SerializeOptions,
) -> Result<Vec<u16>, AddonSerializeError> {
    verify_proposed_api(state.options.get("allowProposedApi")?.is_truthy())?;
    let normal = state.buffers.normal();
    let alt = state.buffers.alt();
    let active = state.buffers.active();
    let normal = normal
        .try_borrow()
        .map_err(|_| AddonSerializeError::Borrowed)?;
    let alternate = alt
        .try_borrow()
        .map_err(|_| AddonSerializeError::Borrowed)?;
    let active = active
        .try_borrow()
        .map_err(|_| AddonSerializeError::Borrowed)?;
    let modes = ModesSnapshot::from_services(&state.core, &state.mouse)
        .map_err(|_| AddonSerializeError::Borrowed)?;
    let active_alternate = Rc::ptr_eq(&state.buffers.active(), &state.buffers.alt());
    let modes = SerializeModes {
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
    };
    Ok(serialize_terminal(
        &SerializeState {
            normal: &*normal,
            alternate: &*alternate,
            active_alternate,
            cols: state.cols(),
            rows: state.rows(),
            current_attrs: &state.attributes,
            modes: &modes,
            scroll_top: active.scroll_top,
            scroll_bottom: active.scroll_bottom,
        },
        options,
    )?)
}
