// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Adapted from common/services/CoreService.ts.

//! Core modes and ordered input events. UTF-16 input preserves JavaScript string
//! code units, including lone surrogates. Buffer position is read from the active
//! buffer through a host-independent callback on each admitted send.

use crate::event::{EmitError, Emitter, Event};
use crate::lifecycle::Disposable;
use crate::log_service::{LogError, LogLevel, LogParameter, LogService, LogValue};
use crate::options::{OptionError, OptionValue, OptionsService};
use std::cell::{Cell, RefCell};
use std::fmt;
use std::rc::Rc;

mod mode_capture;
pub use mode_capture::PrivateModesCapture;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Modes {
    pub insert_mode: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecPrivateModes {
    pub application_cursor_keys: bool,
    pub application_keypad: bool,
    pub bracketed_paste_mode: bool,
    pub color_scheme_updates: bool,
    pub cursor_blink: Option<bool>,
    pub cursor_style: Option<CursorStyle>,
    pub origin: bool,
    pub reverse_wraparound: bool,
    pub send_focus: bool,
    pub synchronized_output: bool,
    pub win32_input_mode: bool,
    pub wraparound: bool,
}
impl Default for DecPrivateModes {
    fn default() -> Self {
        Self {
            application_cursor_keys: false,
            application_keypad: false,
            bracketed_paste_mode: false,
            color_scheme_updates: false,
            cursor_blink: None,
            cursor_style: None,
            origin: false,
            reverse_wraparound: false,
            send_focus: false,
            synchronized_output: false,
            win32_input_mode: false,
            wraparound: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KittyKeyboardState {
    pub flags: u32,
    pub main_flags: u32,
    pub alt_flags: u32,
    pub main_stack: Vec<u32>,
    pub alt_stack: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BufferPosition {
    pub ybase: usize,
    pub ydisp: usize,
}

#[derive(Debug)]
pub enum CoreError {
    Option(OptionError),
    Log(LogError),
    Dispatch(EmitError),
}
impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Option(error) => error.fmt(f),
            Self::Log(error) => error.fmt(f),
            Self::Dispatch(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for CoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Option(error) => error,
            Self::Log(error) => error,
            Self::Dispatch(error) => error,
        })
    }
}
impl From<OptionError> for CoreError {
    fn from(error: OptionError) -> Self {
        Self::Option(error)
    }
}
impl From<LogError> for CoreError {
    fn from(error: LogError) -> Self {
        Self::Log(error)
    }
}
impl From<EmitError> for CoreError {
    fn from(error: EmitError) -> Self {
        Self::Dispatch(error)
    }
}

/// Preserve the constructor's nullish fallback before the first typed boolean
/// write. JS raw options can contain a nonboolean value despite their typings.
/// Adapters expose `value`; engine visibility checks use JavaScript truthiness.
pub struct CursorInitialization(RefCell<OptionValue>);
impl CursorInitialization {
    fn new(value: OptionValue) -> Self {
        Self(RefCell::new(match value {
            OptionValue::Undefined | OptionValue::Null => false.into(),
            value => value,
        }))
    }
    pub fn get(&self) -> bool {
        self.0.borrow().is_truthy()
    }
    pub fn set(&self, value: bool) {
        self.replace(value);
    }
    pub fn replace(&self, value: bool) -> bool {
        let previous = self.0.replace(value.into());
        previous.is_truthy()
    }
    pub fn value(&self) -> OptionValue {
        self.0.borrow().clone()
    }
}

pub struct CoreService {
    pub is_cursor_initialized: CursorInitialization,
    pub is_cursor_hidden: Cell<bool>,
    pub modes: RefCell<Modes>,
    pub dec_private_modes: RefCell<DecPrivateModes>,
    pub kitty_keyboard: RefCell<KittyKeyboardState>,
    options: Rc<OptionsService>,
    logger: Rc<LogService>,
    buffer_position: Box<dyn Fn() -> BufferPosition>,
    private_modes_captures: mode_capture::ModeCaptures,
    data: Emitter<Vec<u16>>,
    user_input: Emitter<()>,
    binary: Emitter<Vec<u16>>,
    scroll_to_bottom: Emitter<()>,
}

impl CoreService {
    pub fn new(
        options: Rc<OptionsService>,
        logger: Rc<LogService>,
        buffer_position: impl Fn() -> BufferPosition + 'static,
    ) -> Result<Self, OptionError> {
        Ok(Self {
            is_cursor_initialized: CursorInitialization::new(options.get("showCursorImmediately")?),
            is_cursor_hidden: Cell::new(false),
            modes: RefCell::new(Modes::default()),
            dec_private_modes: RefCell::new(DecPrivateModes::default()),
            kitty_keyboard: RefCell::new(KittyKeyboardState::default()),
            options,
            logger,
            buffer_position: Box::new(buffer_position),
            private_modes_captures: RefCell::new(Vec::new()),
            data: Emitter::new(),
            user_input: Emitter::new(),
            binary: Emitter::new(),
            scroll_to_bottom: Emitter::new(),
        })
    }

    /// Cursor flags and listener identities deliberately survive terminal reset.
    pub fn reset(&self) {
        self.freeze_private_modes_captures();
        *self.modes.borrow_mut() = Modes::default();
        *self.dec_private_modes.borrow_mut() = DecPrivateModes::default();
        *self.kitty_keyboard.borrow_mut() = KittyKeyboardState::default();
    }

    pub fn on_data(&self) -> Event<Vec<u16>> {
        self.data.event()
    }
    pub fn on_user_input(&self) -> Event<()> {
        self.user_input.event()
    }
    pub fn on_binary(&self) -> Event<Vec<u16>> {
        self.binary.event()
    }
    pub fn on_request_scroll_to_bottom(&self) -> Event<()> {
        self.scroll_to_bottom.event()
    }

    pub fn trigger_data_event(&self, data: &[u16], was_user_input: bool) -> Result<(), CoreError> {
        if self.options.get("disableStdin")?.is_truthy() {
            return Ok(());
        }
        let buffer = (self.buffer_position)();
        if was_user_input
            && self.options.get("scrollOnUserInput")?.is_truthy()
            && buffer.ybase != buffer.ydisp
        {
            self.scroll_to_bottom.fire(&())?;
        }
        if was_user_input {
            self.user_input.fire(&())?;
        }
        self.log_send(data, false)?;
        self.data.fire(&data.to_vec())?;
        Ok(())
    }

    pub fn trigger_binary_event(&self, data: &[u16]) -> Result<(), CoreError> {
        if self.options.get("disableStdin")?.is_truthy() {
            return Ok(());
        }
        self.log_send(data, true)?;
        self.binary.fire(&data.to_vec())?;
        Ok(())
    }

    fn log_send(&self, data: &[u16], binary: bool) -> Result<(), LogError> {
        let kind = if binary { "binary" } else { "data" };
        let mut message: Vec<_> = format!("sending {kind} \"").encode_utf16().collect();
        message.extend_from_slice(data);
        message.push(u16::from(b'"'));
        self.logger.log(LogLevel::Debug, &message, Vec::new())?;
        self.logger.log(
            LogLevel::Trace,
            &format!("sending {kind} (codes)")
                .encode_utf16()
                .collect::<Vec<_>>(),
            vec![LogParameter::Lazy(Box::new(|| {
                LogValue::CodeUnits(data.to_vec())
            }))],
        )
    }
}

impl Disposable for CoreService {
    fn dispose(&self) {
        self.data.dispose();
        self.user_input.dispose();
        self.binary.dispose();
        self.scroll_to_bottom.dispose();
    }
}
impl Drop for CoreService {
    fn drop(&mut self) {
        self.dispose();
    }
}
