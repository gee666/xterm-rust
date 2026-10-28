use super::{HeadlessError, Terminal};
use crate::{
    buffer::marker::Marker,
    public_api::{buffer::BufferNamespaceApi, parser::ParserApi},
    terminal::TerminalError,
};
use std::rc::Rc;

pub(super) fn registrations(
    engine: &mut crate::terminal::CoreTerminal,
    state: &Rc<std::cell::RefCell<crate::terminal::TerminalState>>,
) -> crate::public_api::ParserRegistrations {
    let state = Rc::downgrade(state);
    crate::public_api::ParserRegistrations::new(engine.parser_mut()).with_window_option_guard(
        move |params| {
            let state = state
                .upgrade()
                .ok_or_else(|| crate::event::EmitError::callback(TerminalError::Disposed))?;
            let options = {
                let state = state
                    .try_borrow()
                    .map_err(crate::event::EmitError::callback)?;
                Rc::clone(&state.options)
            };
            crate::terminal::window_option_allowed(&options, params)
                .map_err(crate::event::EmitError::callback)
        },
    )
}

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
        Ok((
            self.parser_registrations.osc_registrar(),
            self.parser_registrations.esc_registrar(),
        ))
    }
    /// Addons can retain the lazy slot without creating the namespace during
    /// activation or changing buffer-event registration order.
    pub fn buffer_cache_for_addon(
        &self,
    ) -> std::rc::Weak<std::cell::OnceCell<Rc<BufferNamespaceApi>>> {
        Rc::downgrade(&self.buffer_cache)
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
        let mut parser = ParserApi::from_registrations(&self.parser_registrations);
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
