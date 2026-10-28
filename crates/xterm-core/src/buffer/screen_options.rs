// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{Buffer, BufferError, MAX_BUFFER_SIZE};
use crate::options::{OptionValue, OptionsService, WindowsPty};
impl Buffer {
    pub(super) fn positive_option(
        &self,
        key: &'static str,
        allow_zero: bool,
    ) -> Result<usize, BufferError> {
        positive_option(&self.options, key, allow_zero)
    }
    pub(super) fn correct_buffer_length(&self, rows: usize) -> Result<usize, BufferError> {
        correct_buffer_length(self.has_scrollback, rows, &self.options)
    }
    pub(super) fn reflow_cursor_line(&self) -> Result<bool, BufferError> {
        Ok(self.options.get("reflowCursorLine")?.is_truthy())
    }
}
fn positive_option(
    options: &OptionsService,
    key: &'static str,
    allow_zero: bool,
) -> Result<usize, BufferError> {
    let value = options.get(key)?.number()?;
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
pub(super) fn correct_buffer_length(
    scrollback: bool,
    rows: usize,
    options: &OptionsService,
) -> Result<usize, BufferError> {
    if !scrollback {
        return Ok(rows);
    }
    Ok(rows
        .saturating_add(positive_option(options, "scrollback", true)?)
        .min(MAX_BUFFER_SIZE))
}
pub(super) fn is_reflow_enabled(
    scrollback: bool,
    options: &OptionsService,
) -> Result<bool, BufferError> {
    let pty = options.get("windowsPty")?;
    if pty.is_truthy() && pty_property(&pty, "buildNumber")?.is_truthy() {
        // Preserve truthiness before numeric coercion and both build getter calls.
        if !scrollback || pty_property(&pty, "backend")? != OptionValue::from("conpty") {
            return Ok(false);
        }
        return Ok(pty_property(&pty, "buildNumber")?.to_number()? >= 21376.0);
    }
    Ok(scrollback)
}
pub(super) fn windows_pty_configured(options: &OptionsService) -> Result<bool, BufferError> {
    if !matches!(
        pty_property(&options.get("windowsPty")?, "backend")?,
        OptionValue::Undefined
    ) {
        return Ok(true);
    }
    Ok(!matches!(
        pty_property(&options.get("windowsPty")?, "buildNumber")?,
        OptionValue::Undefined
    ))
}
fn pty_property(value: &OptionValue, key: &str) -> Result<OptionValue, BufferError> {
    // Native typed payloads retain their original contract. Browser objects use
    // live host getters, including prototypes and the actual thrown error object.
    if let OptionValue::Object(object) = value {
        if let Some(pty) = object.payload::<WindowsPty>() {
            return Ok(match key {
                "backend" => pty
                    .backend
                    .as_deref()
                    .map_or(OptionValue::Undefined, OptionValue::from),
                "buildNumber" => pty
                    .build_number
                    .map_or(OptionValue::Undefined, OptionValue::Number),
                _ => OptionValue::Undefined,
            });
        }
    }
    Ok(value.get_property(key)?)
}
