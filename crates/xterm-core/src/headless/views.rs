use super::{HeadlessError, Terminal};
use crate::{
    buffer::marker::Marker,
    public_api::{buffer::BufferNamespaceApi, parser::ParserApi},
    terminal::TerminalError,
};
use std::rc::Rc;

impl Terminal {
    /// Addons retain Weak handles when their lifetime may outlast the terminal.
    pub fn state_handle_for_addon(&self) -> Rc<std::cell::RefCell<crate::terminal::TerminalState>> {
        Rc::clone(&self.state)
    }
    /// Registration-only handles do not borrow the parser during callbacks.
    pub fn addon_registrars(
        &self,
    ) -> Result<
        (
            crate::parser::osc::OscRegistrar,
            crate::parser::escape::CommandRegistrar,
        ),
        HeadlessError,
    > {
        let mut engine = self
            .engine
            .try_borrow_mut()
            .map_err(|_| HeadlessError::Borrowed)?;
        let parser = engine.parser_mut();
        Ok((parser.osc.registrar(), parser.esc_registrar()))
    }
    pub fn buffer(&self) -> Result<Rc<BufferNamespaceApi>, HeadlessError> {
        if let Some(buffer) = self.buffer_cache.get() {
            return Ok(Rc::clone(buffer));
        }
        let core = self.with_state(|state| Rc::clone(&state.buffers))?;
        Ok(Rc::clone(
            self.buffer_cache
                .get_or_init(|| Rc::new(BufferNamespaceApi::new(core))),
        ))
    }
    /// The parser owns registrations. The transient borrowed facade replaces a
    /// cached JavaScript ParserApi object without copying parser state.
    pub fn with_parser<R>(
        &self,
        action: impl FnOnce(&mut ParserApi<'_>) -> R,
    ) -> Result<R, HeadlessError> {
        let mut engine = self
            .engine
            .try_borrow_mut()
            .map_err(|_| HeadlessError::Borrowed)?;
        let state = Rc::downgrade(&self.state);
        let mut parser = ParserApi::with_window_option_guard(engine.parser_mut(), move |params| {
            let state = state
                .upgrade()
                .ok_or_else(|| crate::event::EmitError::callback(TerminalError::Disposed))?;
            let state = state
                .try_borrow()
                .map_err(crate::event::EmitError::callback)?;
            state
                .window_option_allowed(params)
                .map_err(crate::event::EmitError::callback)
        });
        Ok(action(&mut parser))
    }
    /// Native marker lists are owned snapshots of shared marker identities, not
    /// JavaScript's mutable Array. Marker disposal and line changes remain live.
    pub fn markers(&self) -> Result<Vec<Marker>, HeadlessError> {
        let buffer = self.with_state(|state| state.active_buffer())?;
        let buffer = buffer.try_borrow().map_err(|_| HeadlessError::Borrowed)?;
        let markers = buffer
            .markers
            .try_borrow()
            .map_err(|_| HeadlessError::Borrowed)?;
        Ok(markers.clone())
    }
    pub fn register_marker(&self, offset: i64) -> Result<Marker, HeadlessError> {
        let buffer = self.with_state(|state| state.active_buffer())?;
        let mut buffer = buffer
            .try_borrow_mut()
            .map_err(|_| HeadlessError::Borrowed)?;
        let position = buffer
            .ybase
            .checked_add(buffer.y)
            .and_then(|value| i64::try_from(value).ok())
            .and_then(|value| value.checked_add(offset))
            .ok_or(TerminalError::InvalidDimensions)?;
        Ok(buffer.add_marker(position))
    }
    pub fn add_marker(&self, offset: i64) -> Result<Marker, HeadlessError> {
        self.register_marker(offset)
    }
}
