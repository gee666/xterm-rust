// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! Rust terminal engine. Async parser work is driven explicitly by the host.
mod commands;
mod construction;
mod controls;
mod csi;
mod csi_edits;
mod csi_erase;
pub mod diagnostics;
mod dispatch;
mod parse;
use dispatch::dispatch_events;
mod error;
mod escape;
mod events;
mod forward;
mod kitty_modes;
mod mode_query;
mod modes;
mod operations;
mod osc;
mod osc_colors;
mod print;
mod register;
mod replies;
mod reply_callbacks;
#[cfg(test)]
mod reply_tests;
pub mod sgr;
mod sgr_dispatch;
mod state;
mod window;
mod windows_heuristics;
use crate::{
    input::text_decoder::{StringToUtf32, Utf8ToUtf32},
    lifecycle::{Disposable, DisposableHandle},
    log_service::LogRecord,
    parser::{escape::EscapeSequenceParser, handler::HandlerResult},
};
pub use controls::line_feed_state;
pub use error::TerminalError;
pub use events::{TerminalEvent, WindowReport};
pub use operations::{
    clear_state_handle, mark_dirty_state, request_reset_state, reset_state_handle,
};
pub use osc_colors::{ColorIndex, ColorRequest};
pub use parse::{
    DecodeBufferOverflow, ParseBufferObserver, INITIAL_PARSE_BUFFER_LENGTH, MAX_PARSE_BUFFER_LENGTH,
};
pub use state::TerminalState;
use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};
pub use window::window_option_allowed;
pub(crate) use windows_heuristics::dispose as dispose_windows_wrapping;

pub type EventHandler = Rc<dyn Fn(TerminalEvent) -> Result<(), crate::event::EmitError>>;
/// Host virtual reset action. It calls ordinary reset, not request_reset_state.
pub type ResetRequestHandler = Rc<dyn Fn() -> Result<(), crate::event::EmitError>>;

pub struct CoreTerminal {
    state: Rc<RefCell<TerminalState>>,
    parser: EscapeSequenceParser,
    utf16: StringToUtf32,
    utf8: Utf8ToUtf32,
    decoded: Vec<u32>,
    decoded_length: usize,
    source: Option<Rc<dyn crate::input::parse_source::ParseInputSource>>,
    source_position: usize,
    parse_observer: Option<ParseBufferObserver>,
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
    /// The common emitter, including option-time Windows workaround hooks.
    pub fn on_line_feed(&self) -> crate::event::Event<()> {
        self.state.borrow().line_feed.event()
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
        self.dispose_common_events()?;
        let slot = Rc::clone(
            &self
                .state
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .event_handler,
        );
        self.disposed = true;
        let handler = slot.borrow_mut().take();
        drop(handler);
        for listener in self.listeners.drain(..) {
            listener.dispose();
        }
        // Service disposal can invoke marker callbacks. Keep every terminal-state
        // borrow out of that traversal, and retain source registration order.
        let (options, logger, buffers, core, mouse, unicode) = {
            let state = self
                .state
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?;
            (
                Rc::clone(&state.options),
                Rc::clone(&state.logger),
                Rc::clone(&state.buffer_service),
                Rc::clone(&state.core),
                Rc::clone(&state.mouse),
                Rc::clone(&state.unicode),
            )
        };
        options.dispose();
        logger.dispose();
        buffers.dispose()?;
        core.dispose();
        mouse.dispose();
        unicode.dispose();
        operations::clear_reset_request_handler(&self.state)?;
        self.finish_parser_dispose();
        Ok(())
    }
    /// Finalize reentrant host disposal without revisiting shared services or
    /// markers that callers registered after that traversal returned.
    pub fn finish_dispose_after_shared_services(&mut self) -> Result<(), TerminalError> {
        self.dispose_common_events()?;
        let slot = Rc::clone(
            &self
                .state
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .event_handler,
        );
        self.disposed = true;
        let handler = slot.borrow_mut().take();
        drop(handler);
        for listener in self.listeners.drain(..) {
            listener.dispose();
        }
        operations::clear_reset_request_handler(&self.state)?;
        self.finish_parser_dispose();
        Ok(())
    }
    fn dispose_common_events(&self) -> Result<(), TerminalError> {
        dispose_windows_wrapping(&self.state)?;
        let line_feed = Rc::clone(
            &self
                .state
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .line_feed,
        );
        line_feed.dispose();
        Ok(())
    }
    fn finish_parser_dispose(&mut self) {
        self.parser.dispose();
        self.pending = None;
        self.source = None;
        self.parse_observer = None;
        self.decoded.clear();
        self.decoded_length = 0;
        self.source_position = 0;
        self.awaiting = false;
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
