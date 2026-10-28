// Licensed under MIT. Live browser keyboard fields and a static typed adapter.
use super::{DispatchKeyEvent, KeyEventKind};
use std::{borrow::Cow, marker::PhantomData};
use xterm_core::input::keyboard_event_source::KeyboardEventSource;

/// Null and undefined differ in the source's repeated `which` comparisons.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventNumber {
    Undefined,
    Null,
    Number(f64),
}
impl EventNumber {
    pub fn truthy(self) -> bool {
        matches!(self, Self::Number(number) if number != 0.0 && !number.is_nan())
    }
    pub fn is_zero(self) -> bool {
        matches!(self, Self::Number(number) if number == 0.0)
    }
    pub fn to_number(self) -> f64 {
        match self {
            Self::Number(number) => number,
            Self::Null => 0.0,
            Self::Undefined => f64::NAN,
        }
    }
}

/// Browser-only fields beside the common encoder's event. Calls are live and
/// fallible; a callback or getter may replace another DOM property between reads.
pub trait KeyEventSource: KeyboardEventSource {
    fn repeat(&self) -> Result<bool, Self::Error>;
    fn char_code(&self) -> Result<EventNumber, Self::Error>;
    fn which(&self) -> Result<EventNumber, Self::Error>;
    fn is_key_press(&self) -> Result<bool, Self::Error>;
    fn alt_graph(&self) -> Result<bool, Self::Error>;
}

/// Error-polymorphic borrowed adapter. Numeric optional fields represent absent
/// properties as undefined; DOM adapters can additionally preserve actual null.
pub struct StaticKeyEvent<'a, E> {
    event: &'a DispatchKeyEvent<'a>,
    error: PhantomData<fn() -> E>,
}
impl<'a, E> StaticKeyEvent<'a, E> {
    pub fn new(event: &'a DispatchKeyEvent<'a>) -> Self {
        Self {
            event,
            error: PhantomData,
        }
    }
}
impl<E> KeyboardEventSource for StaticKeyEvent<'_, E> {
    type Error = E;
    fn key_code(&self) -> Result<f64, E> {
        Ok(f64::from(self.event.key.key_code))
    }
    fn key(&self) -> Result<Cow<'_, [u16]>, E> {
        Ok(Cow::Borrowed(self.event.key.key))
    }
    fn code(&self) -> Result<Cow<'_, [u16]>, E> {
        Ok(Cow::Borrowed(self.event.key.code))
    }
    fn shift_key(&self) -> Result<bool, E> {
        Ok(self.event.key.shift_key)
    }
    fn alt_key(&self) -> Result<bool, E> {
        Ok(self.event.key.alt_key)
    }
    fn ctrl_key(&self) -> Result<bool, E> {
        Ok(self.event.key.ctrl_key)
    }
    fn meta_key(&self) -> Result<bool, E> {
        Ok(self.event.key.meta_key)
    }
}
impl<E> KeyEventSource for StaticKeyEvent<'_, E> {
    fn repeat(&self) -> Result<bool, E> {
        Ok(self.event.repeat)
    }
    fn char_code(&self) -> Result<EventNumber, E> {
        Ok(self
            .event
            .char_code
            .map(|number| EventNumber::Number(f64::from(number)))
            .unwrap_or(EventNumber::Undefined))
    }
    fn which(&self) -> Result<EventNumber, E> {
        Ok(self
            .event
            .which
            .map(|number| EventNumber::Number(f64::from(number)))
            .unwrap_or(EventNumber::Undefined))
    }
    fn is_key_press(&self) -> Result<bool, E> {
        Ok(self.event.kind == KeyEventKind::Press)
    }
    fn alt_graph(&self) -> Result<bool, E> {
        Ok(self.event.alt_graph)
    }
}
