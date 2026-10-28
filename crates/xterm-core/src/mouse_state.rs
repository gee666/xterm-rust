// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Adapted from common/services/MouseStateService.ts.

//! Platform-neutral mouse tracking protocols and encodings. Coordinates are
//! passed through as supplied; browser validation and coordinate conversion are
//! separate contracts. Restriction and encoding are independently callable.

#[path = "mouse_protocol.rs"]
mod protocol;
pub use protocol::{
    InvalidMouseCode, MouseAction, MouseButton, MouseEncoding, MouseEvent, MouseEventTypes,
    MouseProtocol, MouseRestriction, SharedMouseProtocol,
};

use crate::event::{EmitError, Emitter, Event};
use crate::lifecycle::Disposable;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

type WheelHandler<W> = Rc<dyn Fn(&W) -> bool>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MouseStateError {
    UnknownProtocol(String),
    UnknownEncoding(String),
    Dispatch(EmitError),
}
impl fmt::Display for MouseStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownProtocol(name) => write!(f, "unknown protocol \"{name}\""),
            Self::UnknownEncoding(name) => write!(f, "unknown encoding \"{name}\""),
            Self::Dispatch(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for MouseStateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dispatch(error) => Some(error),
            _ => None,
        }
    }
}
impl From<EmitError> for MouseStateError {
    fn from(error: EmitError) -> Self {
        Self::Dispatch(error)
    }
}

struct State<W> {
    protocols: Vec<(String, Rc<SharedMouseProtocol>)>,
    encodings: Vec<(String, MouseEncoding)>,
    active_protocol: String,
    active_encoding: String,
    wheel_handler: Option<WheelHandler<W>>,
}

/// The wheel payload type is chosen by the adapter. Browser callers can use a
/// real WheelEvent without coupling this crate to a DOM or a particular OS.
/// Weak references in callbacks avoid owning a cycle back to this service.
pub struct MouseStateService<W = ()> {
    state: RefCell<State<W>>,
    on_protocol_change: Emitter<MouseEventTypes>,
}

impl<W> Default for MouseStateService<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W> MouseStateService<W> {
    pub fn new() -> Self {
        Self {
            state: RefCell::new(State {
                protocols: protocol::default_protocols()
                    .into_iter()
                    .map(|(name, protocol)| (name, Rc::new(SharedMouseProtocol::new(protocol))))
                    .collect(),
                encodings: protocol::default_encodings(),
                active_protocol: "NONE".into(),
                active_encoding: "DEFAULT".into(),
                wheel_handler: None,
            }),
            on_protocol_change: Emitter::new(),
        }
    }

    pub fn on_protocol_change(&self) -> Event<MouseEventTypes> {
        self.on_protocol_change.event()
    }

    pub fn protocol_names(&self) -> Vec<String> {
        self.state
            .borrow()
            .protocols
            .iter()
            .map(|entry| entry.0.clone())
            .collect()
    }

    pub fn encoding_names(&self) -> Vec<String> {
        self.state
            .borrow()
            .encodings
            .iter()
            .map(|entry| entry.0.clone())
            .collect()
    }

    pub fn add_protocol(&self, name: impl Into<String>, protocol: MouseProtocol) {
        self.add_shared_protocol(name, Rc::new(SharedMouseProtocol::new(protocol)));
    }

    /// Retain the same protocol object in one or several services. External edits
    /// affect subsequent reads without replacing the active registration.
    pub fn add_shared_protocol(&self, name: impl Into<String>, protocol: Rc<SharedMouseProtocol>) {
        let name = name.into();
        let previous = {
            let mut state = self.state.borrow_mut();
            if let Some(entry) = state.protocols.iter_mut().find(|entry| entry.0 == name) {
                Some(std::mem::replace(&mut entry.1, protocol))
            } else {
                state.protocols.push((name, protocol));
                None
            }
        };
        drop(previous);
    }

    pub fn add_encoding(&self, name: impl Into<String>, encoding: MouseEncoding) {
        let name = name.into();
        let previous = {
            let mut state = self.state.borrow_mut();
            if let Some(entry) = state.encodings.iter_mut().find(|entry| entry.0 == name) {
                Some(std::mem::replace(&mut entry.1, encoding))
            } else {
                state.encodings.push((name, encoding));
                None
            }
        };
        drop(previous);
    }

    pub fn active_protocol(&self) -> String {
        self.state.borrow().active_protocol.clone()
    }

    pub fn active_encoding(&self) -> String {
        self.state.borrow().active_encoding.clone()
    }

    pub fn set_active_protocol(&self, name: &str) -> Result<(), MouseStateError> {
        let events = {
            let mut state = self.state.borrow_mut();
            let events = state
                .protocols
                .iter()
                .find(|entry| entry.0 == name)
                .map(|entry| entry.1.events())
                .ok_or_else(|| MouseStateError::UnknownProtocol(name.into()))?;
            state.active_protocol = name.into();
            events
        };
        self.on_protocol_change.fire(&events)?;
        Ok(())
    }

    pub fn set_active_encoding(&self, name: &str) -> Result<(), MouseStateError> {
        let mut state = self.state.borrow_mut();
        if !state.encodings.iter().any(|entry| entry.0 == name) {
            return Err(MouseStateError::UnknownEncoding(name.into()));
        }
        state.active_encoding = name.into();
        Ok(())
    }

    fn active_mouse_protocol(&self) -> MouseProtocol {
        let state = self.state.borrow();
        // Construction installs NONE and setters only accept registered names.
        // No API removes registrations, including during disposal.
        state
            .protocols
            .iter()
            .find(|entry| entry.0 == state.active_protocol)
            .expect("active protocol remains registered")
            .1
            .snapshot()
    }

    pub fn are_mouse_events_active(&self) -> bool {
        self.active_mouse_protocol().events != MouseEventTypes::NONE
    }

    pub fn is_default_encoding(&self) -> bool {
        self.state.borrow().active_encoding == "DEFAULT"
    }

    pub fn is_pixel_encoding(&self) -> bool {
        self.state.borrow().active_encoding == "SGR_PIXELS"
    }

    /// The callback is cloned before invocation so it may replace registrations
    /// or call other service methods without crossing a RefCell borrow.
    pub fn restrict_mouse_event(&self, event: &mut MouseEvent) -> bool {
        (self.active_mouse_protocol().restrict)(event)
    }

    pub fn encode_mouse_event(&self, event: &mut MouseEvent) -> Vec<u16> {
        let encoding = {
            let state = self.state.borrow();
            // The same registration invariant as active_mouse_protocol applies.
            Rc::clone(
                &state
                    .encodings
                    .iter()
                    .find(|entry| entry.0 == state.active_encoding)
                    .expect("active encoding remains registered")
                    .1,
            )
        };
        encoding(event)
    }

    pub fn reset(&self) -> Result<(), MouseStateError> {
        self.set_active_protocol("NONE")?;
        self.set_active_encoding("DEFAULT")
    }

    pub fn set_custom_wheel_event_handler(&self, handler: Option<WheelHandler<W>>) {
        let previous = std::mem::replace(&mut self.state.borrow_mut().wheel_handler, handler);
        drop(previous);
    }

    pub fn allow_custom_wheel_event(&self, event: &W) -> bool {
        let handler = self.state.borrow().wheel_handler.clone();
        handler.is_none_or(|handler| handler(event))
    }
}

impl<W> Disposable for MouseStateService<W> {
    /// Like the source DisposableStore, only the event emitter is registered
    /// for cleanup. Protocols, encodings, state and wheel hooks remain callable.
    fn dispose(&self) {
        self.on_protocol_change.dispose();
    }
}

impl<W> Drop for MouseStateService<W> {
    fn drop(&mut self) {
        self.dispose();
    }
}
