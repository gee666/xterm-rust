// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{Buffer, BufferError, MAX_BUFFER_SIZE};
use crate::options::{OptionValue, WindowsPty};

impl Buffer {
    pub(super) fn positive_option(
        &self,
        key: &'static str,
        allow_zero: bool,
    ) -> Result<usize, BufferError> {
        let value = self.options.get(key)?.number()?;
        if !value.is_finite()
            || value < 0.0
            || (!allow_zero && value == 0.0)
            || value.fract() != 0.0
            || value > MAX_BUFFER_SIZE as f64
        {
            return Err(BufferError::InvalidOption(key));
        }
        Ok(value as usize)
    }
    pub(super) fn correct_buffer_length(&self, rows: usize) -> Result<usize, BufferError> {
        if !self.has_scrollback {
            return Ok(rows);
        }
        Ok(rows
            .saturating_add(self.positive_option("scrollback", true)?)
            .min(MAX_BUFFER_SIZE))
    }
    pub(super) fn windows_pty(&self) -> Result<WindowsPty, BufferError> {
        match self.options.get("windowsPty")? {
            OptionValue::Null | OptionValue::Undefined => Ok(WindowsPty::default()),
            OptionValue::Object(value) => value
                .payload::<WindowsPty>()
                .cloned()
                .ok_or(BufferError::InvalidOption("windowsPty")),
            _ => Err(BufferError::InvalidOption("windowsPty")),
        }
    }
    pub(super) fn is_reflow_enabled(&self) -> Result<bool, BufferError> {
        let pty = self.windows_pty()?;
        if let Some(build) = pty.build_number.filter(|b| *b != 0.0 && !b.is_nan()) {
            return Ok(self.has_scrollback
                && pty.backend.as_deref() == Some("conpty")
                && build >= 21376.0);
        }
        Ok(self.has_scrollback)
    }
    pub(super) fn reflow_cursor_line(&self) -> Result<bool, BufferError> {
        Ok(self.options.get("reflowCursorLine")?.is_truthy())
    }
}
