// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from browser/CoreBrowserTerminal.ts input methods.

//! Synchronous state/effects and live event reads, with no borrow held over a callback.
pub use super::keyboard_dispatch_types::*;
use std::{cell::Cell, convert::Infallible};
use xterm_core::input::{
    keyboard::KeyboardResultType,
    keyboard_event_source::{number_is_truthy, to_uint16},
};

#[derive(Debug, Default)]
pub struct KeyboardDispatch {
    key_down_handled: Cell<bool>,
    key_down_seen: Cell<bool>,
    key_press_handled: Cell<bool>,
    unprocessed_dead_key: Cell<bool>,
}

impl KeyboardDispatch {
    pub fn state(&self) -> DispatchState {
        DispatchState {
            key_down_handled: self.key_down_handled.get(),
            key_down_seen: self.key_down_seen.get(),
            key_press_handled: self.key_press_handled.get(),
            unprocessed_dead_key: self.unprocessed_dead_key.get(),
        }
    }
    pub fn key_down<H: KeyboardHost>(
        &self,
        event: &DispatchKeyEvent<'_>,
        host: &H,
    ) -> Result<Option<bool>, H::Error> {
        self.key_down_source(&StaticKeyEvent::<H::Error>::new(event), host)
    }
    pub fn key_up<H: KeyboardHost>(
        &self,
        event: &DispatchKeyEvent<'_>,
        host: &H,
    ) -> Result<(), H::Error> {
        self.key_up_source(&StaticKeyEvent::<H::Error>::new(event), host)
    }
    pub fn key_press<H: KeyboardHost>(
        &self,
        event: &DispatchKeyEvent<'_>,
        host: &H,
    ) -> Result<bool, H::Error> {
        self.key_press_source(&StaticKeyEvent::<H::Error>::new(event), host)
    }

    /// None is JavaScript's undefined return, distinct from false and true.
    pub fn key_down_source<H: KeyboardHost>(
        &self,
        event: &dyn KeyEventSource<Error = H::Error>,
        host: &H,
    ) -> Result<Option<bool>, H::Error> {
        self.key_down_handled.set(false);
        self.key_down_seen.set(true);
        if !host.custom_key_handler(event)? {
            return Ok(Some(false));
        }
        let ignore_composition = host.is_mac()? && host.mac_option_is_meta()? && event.alt_key()?;
        if !ignore_composition && !host.composition_key_down(event)? {
            if host.scroll_on_user_input()? {
                let (ybase, ydisp) = host.scroll_position()?;
                if ybase != ydisp {
                    host.perform(Some(event), KeyboardAction::ScrollToBottom)?;
                }
            }
            return Ok(Some(false));
        }
        if !ignore_composition
            && (equals(&event.key()?, "Dead") || equals(&event.key()?, "AltGraph"))
        {
            self.unprocessed_dead_key.set(true);
        }
        let result = host.evaluate_key_down(event)?;
        host.perform(Some(event), KeyboardAction::UpdateCursorStyle)?;
        if matches!(
            result.result_type,
            KeyboardResultType::PageDown | KeyboardResultType::PageUp
        ) {
            let count = i64::from(host.rows()?) - 1;
            let count = if result.result_type == KeyboardResultType::PageUp {
                -count
            } else {
                count
            };
            host.perform(Some(event), KeyboardAction::ScrollLines(count))?;
            host.perform(Some(event), KeyboardAction::Cancel)?;
            return Ok(Some(false));
        }
        if result.result_type == KeyboardResultType::SelectAll {
            host.perform(Some(event), KeyboardAction::SelectAll)?;
        }
        if third_level_shift_source(host, event)? {
            return Ok(Some(true));
        }
        if result.cancel {
            host.perform(Some(event), KeyboardAction::Cancel)?;
        }
        let Some(key) = result.key.as_deref().filter(|key| !key.is_empty()) else {
            return Ok(Some(true));
        };
        if !host.use_kitty()?
            && !host.use_win32()?
            && !event.key()?.is_empty()
            && !event.ctrl_key()?
            && !event.alt_key()?
            && !event.meta_key()?
            && event.key()?.len() == 1
            && first_unit(&event.key()?) >= 65.0
            && first_unit(&event.key()?) <= 90.0
        {
            return Ok(Some(true));
        }
        if self.unprocessed_dead_key.get() {
            self.unprocessed_dead_key.set(false);
            return Ok(Some(true));
        }
        if key == [3] || key == [13] {
            host.perform(Some(event), KeyboardAction::ClearTextarea)?;
        }
        let modifier_only = host.use_win32()? && modifier_key_only_source(event)?;
        host.perform(Some(event), KeyboardAction::OnKey(key))?;
        host.perform(Some(event), KeyboardAction::ShowCursor)?;
        host.perform(
            Some(event),
            KeyboardAction::Data {
                key,
                user_input: !modifier_only,
            },
        )?;
        if !host.screen_reader_mode()? || event.alt_key()? || event.ctrl_key()? {
            host.perform(Some(event), KeyboardAction::Cancel)?;
            return Ok(Some(false));
        }
        self.key_down_handled.set(true);
        Ok(None)
    }

    pub fn key_up_source<H: KeyboardHost>(
        &self,
        event: &dyn KeyEventSource<Error = H::Error>,
        host: &H,
    ) -> Result<(), H::Error> {
        self.key_down_seen.set(false);
        if !host.custom_key_handler(event)? {
            return Ok(());
        }
        if !modifier_key_only_source(event)? {
            host.perform(Some(event), KeyboardAction::Focus)?;
        }
        if let Some(result) = host.evaluate_key_up(event)? {
            if let Some(key) = result.key.as_deref().filter(|key| !key.is_empty()) {
                let modifier_only = host.use_win32()? && modifier_key_only_source(event)?;
                host.perform(
                    Some(event),
                    KeyboardAction::Data {
                        key,
                        user_input: !modifier_only,
                    },
                )?;
            }
        }
        host.perform(Some(event), KeyboardAction::UpdateCursorStyle)?;
        self.key_press_handled.set(false);
        Ok(())
    }

    pub fn key_press_source<H: KeyboardHost>(
        &self,
        event: &dyn KeyEventSource<Error = H::Error>,
        host: &H,
    ) -> Result<bool, H::Error> {
        self.key_press_handled.set(false);
        if self.key_down_handled.get() || !host.custom_key_handler(event)? {
            return Ok(false);
        }
        let code = if event.char_code()?.truthy() {
            event.char_code()?
        } else if event.which()? == EventNumber::Null || event.which()? == EventNumber::Undefined {
            EventNumber::Number(event.key_code()?)
        } else if !event.which()?.is_zero() && !event.char_code()?.is_zero() {
            event.which()?
        } else {
            return Ok(false);
        };
        if !code.truthy()
            || ((event.alt_key()? || event.ctrl_key()? || event.meta_key()?)
                && !third_level_shift_source(host, event)?)
        {
            return Ok(false);
        }
        let key = [to_uint16(code.to_number())];
        host.perform(Some(event), KeyboardAction::OnKey(&key))?;
        host.perform(Some(event), KeyboardAction::ShowCursor)?;
        host.perform(
            Some(event),
            KeyboardAction::Data {
                key: &key,
                user_input: true,
            },
        )?;
        self.key_press_handled.set(true);
        self.unprocessed_dead_key.set(false);
        Ok(true)
    }

    pub fn input<H: KeyboardHost>(
        &self,
        event: TextInputEvent<'_>,
        host: &H,
    ) -> Result<bool, H::Error> {
        let Some(key) = event.data.filter(|key| !key.is_empty()) else {
            return Ok(false);
        };
        if !event.insert_text
            || (event.composed && self.key_down_seen.get())
            || host.screen_reader_mode()?
            || self.key_press_handled.get()
        {
            return Ok(false);
        }
        self.unprocessed_dead_key.set(false);
        host.perform(
            None,
            KeyboardAction::Data {
                key,
                user_input: true,
            },
        )?;
        Ok(true)
    }
}

pub fn third_level_shift<H: KeyboardHost>(
    host: &H,
    event: &DispatchKeyEvent<'_>,
) -> Result<bool, H::Error> {
    third_level_shift_source(host, &StaticKeyEvent::<H::Error>::new(event))
}
pub fn third_level_shift_source<H: KeyboardHost>(
    host: &H,
    event: &dyn KeyEventSource<Error = H::Error>,
) -> Result<bool, H::Error> {
    let third = (host.is_mac()?
        && !host.mac_option_is_meta()?
        && event.alt_key()?
        && !event.ctrl_key()?
        && !event.meta_key()?)
        || (host.is_windows()? && event.alt_key()? && event.ctrl_key()? && !event.meta_key()?)
        || (host.is_windows()? && host.alt_graph_state(event)?);
    if event.is_key_press()? {
        return Ok(third);
    }
    Ok(third && (!number_is_truthy(event.key_code()?) || event.key_code()? > 47.0))
}

pub fn modifier_key_only(event: &DispatchKeyEvent<'_>) -> bool {
    match modifier_key_only_source(&StaticKeyEvent::<Infallible>::new(event)) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub fn modifier_key_only_source<E>(event: &dyn KeyEventSource<Error = E>) -> Result<bool, E> {
    Ok(event.key_code()? == 16.0
        || event.key_code()? == 17.0
        || event.key_code()? == 18.0
        || event.key_code()? == 91.0
        || event.key_code()? == 92.0
        || event.key_code()? == 93.0
        || event.key_code()? == 224.0
        || equals(&event.key()?, "Meta"))
}
fn equals(units: &[u16], text: &str) -> bool {
    units.iter().copied().eq(text.encode_utf16())
}
fn first_unit(units: &[u16]) -> f64 {
    units
        .first()
        .map(|unit| f64::from(*unit))
        .unwrap_or(f64::NAN)
}
