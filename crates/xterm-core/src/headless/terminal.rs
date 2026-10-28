// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Public native wrapper over the real Rust terminal engine. Synchronous writes
//! do not emulate JavaScript's event loop. A pending parser handler must be
//! resumed explicitly before a subsequent synchronous write can be accepted.
use super::{
    events::Events, HeadlessError, ModesSnapshot, PublicOptions, RenderEvent, ResizeEvent,
};
use crate::{
    event::Event,
    options::OptionsService,
    public_api::addon_manager::{AddonHandle, AddonManager, TerminalAddon},
    terminal::{CoreTerminal, TerminalError, TerminalEvent, TerminalState},
    unicode_service::UnicodeService,
};
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};

pub struct Terminal {
    pub(super) engine: RefCell<CoreTerminal>,
    options: PublicOptions,
    disposed: std::cell::Cell<bool>,
    deferred_parser_disposal: std::cell::Cell<bool>,
    dispose_request: crate::event::Emitter<()>,
    pub(super) events: Rc<Events>,
    pub(super) state: Rc<RefCell<TerminalState>>,
    addons: AddonManager<Terminal, HeadlessError>,
    pub(super) parser_registrations: crate::public_api::ParserRegistrations,
    pub(super) buffer_cache: Rc<OnceCell<Rc<crate::public_api::buffer::BufferNamespaceApi>>>,
}
impl Terminal {
    pub fn new(cols: usize, rows: usize, scrollback: usize) -> Result<Self, HeadlessError> {
        Self::from_engine(CoreTerminal::new(cols, rows, scrollback)?)
    }
    pub fn with_options(options: Rc<OptionsService>) -> Result<Self, HeadlessError> {
        Self::from_engine(CoreTerminal::with_options(options)?)
    }
    fn from_engine(mut engine: CoreTerminal) -> Result<Self, HeadlessError> {
        let options = PublicOptions::new(Rc::clone(&engine.state().options));
        // Construction events precede public listener registration upstream.
        engine.drain_events();
        let state = engine.state_handle();
        let parser_registrations = super::views::registrations(&mut engine, &state);
        let events = Rc::new(Events::default());
        let output = Rc::clone(&events);
        engine.set_event_handler(Some(Rc::new(move |event| output.dispatch(event))));
        Ok(Self {
            engine: RefCell::new(engine),
            options,
            disposed: std::cell::Cell::new(false),
            deferred_parser_disposal: std::cell::Cell::new(false),
            dispose_request: crate::event::Emitter::new(),
            events,
            state,
            addons: AddonManager::new(),
            parser_registrations,
            buffer_cache: Rc::new(OnceCell::new()),
        })
    }
    pub fn options(&self) -> &PublicOptions {
        &self.options
    }
    /// Native embeddings can drain the default captured diagnostics or install
    /// a live console sink through the shared LogService before parsing.
    pub fn drain_logs(&self) -> Result<Vec<crate::log_service::LogRecord>, HeadlessError> {
        Ok(self
            .engine
            .try_borrow_mut()
            .map_err(|_| HeadlessError::Borrowed)?
            .drain_logs())
    }
    pub fn cols(&self) -> Result<usize, HeadlessError> {
        self.with_state(|state| state.cols())
    }
    pub fn rows(&self) -> Result<usize, HeadlessError> {
        self.with_state(|state| state.rows())
    }
    pub fn with_state<R>(
        &self,
        read: impl FnOnce(&TerminalState) -> R,
    ) -> Result<R, HeadlessError> {
        let state = self
            .state
            .try_borrow()
            .map_err(|_| HeadlessError::Borrowed)?;
        Ok(read(&state))
    }
    pub fn unicode(&self) -> Result<Rc<UnicodeService>, HeadlessError> {
        self.options.check_proposed_api()?;
        self.with_state(|state| Rc::clone(&state.unicode))
    }
    pub fn modes(&self) -> Result<ModesSnapshot, HeadlessError> {
        self.with_state(|state| ModesSnapshot::from_services(&state.core, &state.mouse))?
            .map_err(|_| HeadlessError::Borrowed)
    }
    pub fn write(&self, text: &str) -> Result<(), HeadlessError> {
        if self.disposed.get() {
            return Ok(());
        }
        self.operate(|engine| engine.write(text))
    }
    pub fn write_utf16(&self, text: &[u16]) -> Result<(), HeadlessError> {
        if self.disposed.get() {
            return Ok(());
        }
        self.operate(|engine| engine.write_utf16(text))
    }
    pub fn write_bytes(&self, bytes: &[u8]) -> Result<(), HeadlessError> {
        if self.disposed.get() {
            return Ok(());
        }
        self.operate(|engine| engine.write_bytes(bytes))
    }
    /// Completion follows parsing and forwarded notifications. This native
    /// synchronous method returns Pending instead of silently dropping callbacks.
    pub fn write_with_callback(
        &self,
        text: &str,
        callback: impl FnOnce(&Self) -> Result<(), HeadlessError>,
    ) -> Result<(), HeadlessError> {
        self.write(text)?;
        if self.disposed.get() {
            return Ok(());
        }
        callback(self)
    }
    pub fn write_bytes_with_callback(
        &self,
        bytes: &[u8],
        callback: impl FnOnce(&Self) -> Result<(), HeadlessError>,
    ) -> Result<(), HeadlessError> {
        self.write_bytes(bytes)?;
        if self.disposed.get() {
            return Ok(());
        }
        callback(self)
    }
    pub fn writeln(&self, text: &str) -> Result<(), HeadlessError> {
        self.write(text)?;
        self.write("\r\n")
    }
    pub fn writeln_bytes(&self, bytes: &[u8]) -> Result<(), HeadlessError> {
        self.write_bytes(bytes)?;
        self.write("\r\n")
    }
    pub fn writeln_with_callback(
        &self,
        text: &str,
        callback: impl FnOnce(&Self) -> Result<(), HeadlessError>,
    ) -> Result<(), HeadlessError> {
        self.writeln(text)?;
        if self.disposed.get() {
            return Ok(());
        }
        callback(self)
    }
    pub fn input(&self, data: &[u16], was_user_input: bool) -> Result<(), HeadlessError> {
        let core = self.with_state(|state| Rc::clone(&state.core))?;
        core.trigger_data_event(data, was_user_input)
            .map_err(TerminalError::from)?;
        self.flush_notifications()
    }
    pub fn input_binary(&self, data: &[u16]) -> Result<(), HeadlessError> {
        let core = self.with_state(|state| Rc::clone(&state.core))?;
        core.trigger_binary_event(data)
            .map_err(TerminalError::from)?;
        self.flush_notifications()
    }
    pub(super) fn flush_notifications(&self) -> Result<(), HeadlessError> {
        let queue = self.with_state(|state| Rc::clone(&state.events))?;
        let events = std::mem::take(&mut *queue.borrow_mut());
        for event in events {
            self.events.dispatch(event)?;
        }
        Ok(())
    }
    pub fn resize(&self, cols: usize, rows: usize) -> Result<(), HeadlessError> {
        if cols == self.cols()? && rows == self.rows()? {
            return Ok(());
        }
        if self.disposed.get() || self.engine.try_borrow_mut().is_err() {
            if cols > i32::MAX as usize || rows > i32::MAX as usize {
                return Err(TerminalError::InvalidDimensions.into());
            }
            let service = self.with_state(|state| Rc::clone(&state.buffer_service))?;
            service
                .resize(cols.max(2), rows.max(1))
                .map_err(TerminalError::from)?;
            return self.flush_notifications();
        }
        self.operate(|engine| engine.resize(cols, rows))
    }
    pub fn clear(&self) -> Result<(), HeadlessError> {
        if !self.disposed.get() && self.engine.try_borrow_mut().is_ok() {
            return self.operate(CoreTerminal::clear);
        }
        crate::terminal::clear_state_handle(&self.state)?;
        self.flush_notifications()
    }
    pub fn reset(&self) -> Result<(), HeadlessError> {
        if !self.disposed.get() && self.engine.try_borrow_mut().is_ok() {
            return self.operate(CoreTerminal::reset);
        }
        // A synchronous parser callback may reset terminal state without
        // resetting or borrowing the parser currently executing that callback.
        crate::terminal::reset_state_handle(&self.state).map_err(Into::into)
    }
    pub fn take_pending(
        &self,
    ) -> Result<Option<crate::parser::handler::HandlerResult>, HeadlessError> {
        Ok(self
            .engine
            .try_borrow_mut()
            .map_err(|_| HeadlessError::Borrowed)?
            .take_pending())
    }
    pub fn resume(&self, result: bool) -> Result<(), HeadlessError> {
        self.operate(|engine| engine.resume(result))
    }
    pub fn load_addon(
        &self,
        addon: Rc<dyn TerminalAddon<Self, Error = HeadlessError>>,
    ) -> Result<AddonHandle<Self, HeadlessError>, HeadlessError> {
        Ok(self.addons.load_addon(self, addon)?)
    }
    pub fn dispose(&self) -> Result<(), HeadlessError> {
        if self.disposed.replace(true) {
            return Ok(());
        }
        self.events.dispose_core();
        if let Ok(mut engine) = self.engine.try_borrow_mut() {
            engine.dispose()?;
        } else {
            // The active parser finishes its current chunk upstream. Dispose
            // shared services now; parser ownership is released after that call.
            let (options, logger, buffers, core, mouse, unicode) = self.with_state(|state| {
                (
                    Rc::clone(&state.options),
                    Rc::clone(&state.logger),
                    Rc::clone(&state.buffer_service),
                    Rc::clone(&state.core),
                    Rc::clone(&state.mouse),
                    Rc::clone(&state.unicode),
                )
            })?;
            use crate::lifecycle::Disposable;
            options.dispose();
            logger.dispose();
            buffers.dispose().map_err(TerminalError::from)?;
            core.dispose();
            mouse.dispose();
            unicode.dispose();
            self.deferred_parser_disposal.set(true);
        }
        self.dispose_request.fire(&())?;
        crate::lifecycle::Disposable::dispose(&self.dispose_request);
        self.events.dispose();
        self.addons.dispose()?;
        if let Some(buffer) = self.buffer_cache.get() {
            crate::lifecycle::Disposable::dispose(buffer.as_ref());
        }
        Ok(())
    }
    pub fn is_disposed(&self) -> Result<bool, HeadlessError> {
        Ok(self.disposed.get())
    }
    pub fn on_dispose_request(&self) -> Event<()> {
        self.dispose_request.event()
    }
    pub(super) fn operate(
        &self,
        action: impl FnOnce(&mut CoreTerminal) -> Result<(), TerminalError>,
    ) -> Result<(), HeadlessError> {
        let (result, events) = {
            let mut engine = self
                .engine
                .try_borrow_mut()
                .map_err(|_| HeadlessError::Borrowed)?;
            let result = action(&mut engine);
            if self.deferred_parser_disposal.replace(false) {
                engine.finish_dispose_after_shared_services()?;
            }
            (result, engine.drain_events())
        };
        // The engine dispatches parser notifications synchronously. Operations
        // that only enqueue notifications are forwarded after their borrow ends.
        for event in events {
            self.events.dispatch(event)?;
        }
        result.map_err(Into::into)
    }
    pub fn on_bell(&self) -> Event<()> {
        self.events.bell()
    }
    pub fn on_binary(&self) -> Event<Vec<u16>> {
        self.events.binary()
    }
    pub fn on_cursor_move(&self) -> Event<()> {
        self.events.cursor_move()
    }
    pub fn on_data(&self) -> Event<Vec<u16>> {
        self.events.data()
    }
    pub fn on_line_feed(&self) -> Event<()> {
        self.events.line_feed()
    }
    pub fn on_render(&self) -> Event<RenderEvent> {
        self.events.render()
    }
    pub fn on_resize(&self) -> Event<ResizeEvent> {
        self.events.resize()
    }
    pub fn on_scroll(&self) -> Event<usize> {
        self.events.scroll()
    }
    pub fn on_title_change(&self) -> Event<Vec<u16>> {
        self.events.title()
    }
    pub fn on_write_parsed(&self) -> Event<()> {
        self.events.write_parsed()
    }
    pub fn on_event(&self) -> Event<TerminalEvent> {
        self.events.all()
    }
}
impl crate::unicode_addons::UnicodeAddonHost for Terminal {
    type Error = HeadlessError;
    fn unicode_service_for_addon(&self) -> Result<Rc<UnicodeService>, Self::Error> {
        self.unicode()
    }
}
