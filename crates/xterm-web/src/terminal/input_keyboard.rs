//! Individual option reads preserve short-circuiting and getter error order.
use super::{error, EngineInputHost};
use crate::input::{
    keyboard::KeyboardModes,
    keyboard_dispatch::{KeyEventSource, KeyboardReadHost, KeyboardSnapshot},
};
use wasm_bindgen::JsValue;
use xterm_core::input::{
    keyboard::{try_evaluate_keyboard_event, KeyboardResult},
    kitty_keyboard::{self, flags, EventType},
    win32_input_mode::Win32InputMode,
};

impl EngineInputHost {
    fn kitty_flags(&self) -> Result<u32, JsValue> {
        Ok(self.core.kitty_keyboard.try_borrow().map_err(error)?.flags)
    }
}

impl KeyboardReadHost for EngineInputHost {
    type Error = JsValue;

    /// An explicit diagnostic snapshot. Dispatch overrides every individual read
    /// below, so it never evaluates unrelated getters through this method.
    fn snapshot(&self) -> Result<KeyboardSnapshot, JsValue> {
        let (ybase, ydisp) = self.scroll_position()?;
        let dec = self
            .core
            .dec_private_modes
            .try_borrow()
            .map_err(error)?
            .clone();
        Ok(KeyboardSnapshot {
            modes: KeyboardModes {
                win32_extension: self.extension("win32InputMode")?,
                win32_mode: dec.win32_input_mode,
                kitty_extension: self.extension("kittyKeyboard")?,
                kitty_flags: self.kitty_flags()?,
                application_cursor_keys: dec.application_cursor_keys,
                is_mac: self.platform.mac,
                mac_option_is_meta: self.mac_option_is_meta()?,
            },
            is_mac: self.platform.mac,
            is_windows: self.platform.windows,
            screen_reader_mode: self.screen_reader_mode()?,
            scroll_on_user_input: self.scroll_on_user_input()?,
            rows: self.rows()?,
            ybase,
            ydisp,
        })
    }
    fn is_mac(&self) -> Result<bool, JsValue> {
        Ok(self.platform.mac)
    }
    fn is_windows(&self) -> Result<bool, JsValue> {
        Ok(self.platform.windows)
    }
    fn mac_option_is_meta(&self) -> Result<bool, JsValue> {
        self.option_bool("macOptionIsMeta")
    }
    fn scroll_on_user_input(&self) -> Result<bool, JsValue> {
        self.option_bool("scrollOnUserInput")
    }
    fn screen_reader_mode(&self) -> Result<bool, JsValue> {
        self.option_bool("screenReaderMode")
    }
    fn rows(&self) -> Result<u32, JsValue> {
        Ok(self.buffer.rows() as u32)
    }
    fn scroll_position(&self) -> Result<(u32, u32), JsValue> {
        let buffer = self.buffer.buffer();
        let buffer = buffer.try_borrow().map_err(error)?;
        Ok((buffer.ybase as u32, buffer.ydisp as u32))
    }
    fn use_kitty(&self) -> Result<bool, JsValue> {
        let flags = self.kitty_flags()?;
        Ok(self.extension("kittyKeyboard")? && kitty_keyboard::should_use_protocol(flags))
    }
    fn use_win32(&self) -> Result<bool, JsValue> {
        Ok(self.extension("win32InputMode")?
            && self
                .core
                .dec_private_modes
                .try_borrow()
                .map_err(error)?
                .win32_input_mode)
    }
    fn evaluate_key_down(
        &self,
        event: &dyn KeyEventSource<Error = JsValue>,
    ) -> Result<KeyboardResult, JsValue> {
        if self.use_win32()? {
            return Win32InputMode.try_evaluate_keyboard_event(event, true);
        }
        let flags = self.kitty_flags()?;
        if self.use_kitty()? {
            kitty_keyboard::try_evaluate(
                event,
                flags,
                if event.repeat()? {
                    EventType::Repeat
                } else {
                    EventType::Press
                },
                self.platform.mac && self.mac_option_is_meta()?,
            )
        } else {
            let application_cursor = self
                .core
                .dec_private_modes
                .try_borrow()
                .map_err(error)?
                .application_cursor_keys;
            try_evaluate_keyboard_event(
                event,
                application_cursor,
                self.platform.mac,
                self.mac_option_is_meta()?,
            )
        }
    }
    fn evaluate_key_up(
        &self,
        event: &dyn KeyEventSource<Error = JsValue>,
    ) -> Result<Option<KeyboardResult>, JsValue> {
        if self.use_win32()? {
            return Win32InputMode
                .try_evaluate_keyboard_event(event, false)
                .map(Some);
        }
        let flags = self.kitty_flags()?;
        if self.use_kitty()? && flags & flags::REPORT_EVENT_TYPES != 0 {
            kitty_keyboard::try_evaluate(
                event,
                flags,
                EventType::Release,
                self.platform.mac && self.mac_option_is_meta()?,
            )
            .map(Some)
        } else {
            Ok(None)
        }
    }
}
