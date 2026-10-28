// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! Rust terminal engine. Async parser work is driven explicitly by the host.
mod commands;
mod construction;
mod controls;
mod csi;
mod csi_edits;
mod csi_erase;
mod dispatch;
use dispatch::dispatch_events;
mod error;
mod escape;
mod events;
mod kitty_modes;
mod mode_query;
mod modes;
mod operations;
mod osc;
mod osc_colors;
mod print;
mod register;
mod replies;
pub mod sgr;
mod state;
mod window;
mod windows_heuristics;
use crate::{
    input::text_decoder::{StringToUtf32, Utf8ToUtf32},
    lifecycle::{Disposable, DisposableHandle},
    log_service::LogRecord,
    parser::{escape::EscapeSequenceParser, handler::HandlerResult},
};
pub use error::TerminalError;
pub use events::{TerminalEvent, WindowReport};
pub use osc_colors::{ColorIndex, ColorRequest};
pub use state::TerminalState;
use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

pub type EventHandler = Rc<dyn Fn(TerminalEvent) -> Result<(), crate::event::EmitError>>;

pub struct CoreTerminal {
    state: Rc<RefCell<TerminalState>>,
    parser: EscapeSequenceParser,
    utf16: StringToUtf32,
    utf8: Utf8ToUtf32,
    decoded: Vec<u32>,
    pending: Option<HandlerResult>,
    awaiting: bool,
    cursor_start: (usize, usize),
    disposed: bool,
    listeners: Vec<DisposableHandle>,
    logs: Rc<RefCell<Vec<LogRecord>>>,
}
impl CoreTerminal {
    pub fn state(&self) -> Ref<'_, TerminalState> {
        self.state.borrow()
    }
    pub fn state_handle(&self) -> Rc<RefCell<TerminalState>> {
        Rc::clone(&self.state)
    }
    pub fn set_event_handler(&mut self, handler: Option<EventHandler>) {
        *self.state.borrow().event_handler.borrow_mut() = handler;
    }
    pub fn parser_mut(&mut self) -> &mut EscapeSequenceParser {
        &mut self.parser
    }
    pub fn cols(&self) -> usize {
        self.state.borrow().cols()
    }
    pub fn rows(&self) -> usize {
        self.state.borrow().rows()
    }
    pub fn is_disposed(&self) -> bool {
        self.disposed
    }
    pub fn drain_events(&mut self) -> Vec<TerminalEvent> {
        std::mem::take(&mut *self.state.borrow().events.borrow_mut())
    }
    pub fn drain_logs(&mut self) -> Vec<LogRecord> {
        std::mem::take(&mut *self.logs.borrow_mut())
    }
    pub fn input(&mut self, data: &[u16], was_user_input: bool) -> Result<(), TerminalError> {
        self.ensure_live()?;
        let core = Rc::clone(&self.state.borrow().core);
        core.trigger_data_event(data, was_user_input)?;
        dispatch_events(&self.state)
    }
    pub fn write_utf16(&mut self, data: &[u16]) -> Result<(), TerminalError> {
        self.begin_write()?;
        self.utf16.decode(data, &mut self.decoded);
        self.parse(None)
    }
    pub fn write_bytes(&mut self, data: &[u8]) -> Result<(), TerminalError> {
        self.begin_write()?;
        self.utf8.decode(data, &mut self.decoded);
        self.parse(None)
    }
    pub fn write(&mut self, data: &str) -> Result<(), TerminalError> {
        self.write_utf16(&data.encode_utf16().collect::<Vec<_>>())
    }
    fn begin_write(&mut self) -> Result<(), TerminalError> {
        self.ensure_live()?;
        if self.awaiting {
            return Err(TerminalError::Pending);
        }
        self.decoded.clear();
        let state = self.state.borrow();
        let b = state.active_buffer();
        let b = b.borrow();
        self.cursor_start = (b.x, b.y);
        drop(b);
        drop(state);
        self.state.borrow_mut().dirty = Some((self.cursor_start.1, self.cursor_start.1));
        Ok(())
    }
    /// Take and await the handler future, then call `resume` with its result.
    pub fn take_pending(&mut self) -> Option<HandlerResult> {
        self.pending.take()
    }
    /// Spawn these cleanup futures before polling the current pending action.
    pub fn take_cleanup(&mut self) -> Vec<HandlerResult> {
        self.parser.take_cleanup()
    }
    pub fn resume(&mut self, result: bool) -> Result<(), TerminalError> {
        self.ensure_live()?;
        if !self.awaiting {
            return Err(TerminalError::Parse(
                crate::parser::escape::ParseError::ImproperContinuation,
            ));
        }
        self.pending = None;
        self.awaiting = false;
        self.parse(Some(result))
    }
    fn parse(&mut self, result: Option<bool>) -> Result<(), TerminalError> {
        let result = self.parser.parse(&self.decoded, result);
        if let Some(error) = self.state.borrow_mut().error.take() {
            return Err(error);
        }
        match result? {
            Some(pending) => {
                self.pending = Some(pending);
                self.awaiting = true;
                dispatch_events(&self.state)?;
                Err(TerminalError::Pending)
            }
            None => {
                let mut state = self.state.borrow_mut();
                let b = state.active_buffer();
                let b = b.borrow();
                if self.cursor_start != (b.x, b.y) {
                    state.events.borrow_mut().push(TerminalEvent::CursorMove);
                }
                if let Some((start, end)) = state.dirty.take() {
                    let offset = b.ybase.saturating_sub(b.ydisp);
                    if start.saturating_add(offset) < state.rows() {
                        state.events.borrow_mut().push(TerminalEvent::Render {
                            start: start.saturating_add(offset).min(state.rows() - 1),
                            end: end.saturating_add(offset).min(state.rows() - 1),
                        });
                    }
                }
                state.events.borrow_mut().push(TerminalEvent::WriteParsed);
                drop(b);
                drop(state);
                self.decoded.clear();
                dispatch_events(&self.state)
            }
        }
    }
    pub(crate) fn ensure_live(&self) -> Result<(), TerminalError> {
        if self.disposed {
            Err(TerminalError::Disposed)
        } else {
            Ok(())
        }
    }
    pub fn dispose(&mut self) -> Result<(), TerminalError> {
        if self.disposed {
            return Ok(());
        }
        self.disposed = true;
        self.parser.dispose();
        self.pending = None;
        self.decoded.clear();
        for listener in self.listeners.drain(..) {
            listener.dispose();
        }
        let state = self.state.borrow();
        state.core.dispose();
        state.logger.dispose();
        state.mouse.dispose();
        state.unicode.dispose();
        state.buffer_service.dispose()?;
        self.awaiting = false;
        Ok(())
    }
}
impl Drop for CoreTerminal {
    fn drop(&mut self) {
        // Drop cannot return failures; retained state handles keep the cause inspectable.
        if let Err(error) = self.dispose() {
            self.state.borrow_mut().error = Some(error);
        }
    }
}
