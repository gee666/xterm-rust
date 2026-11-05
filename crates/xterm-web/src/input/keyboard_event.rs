// Licensed under MIT. Live browser keyboard fields and a static typed adapter.
use super::{DispatchKeyEvent, KeyEventKind};
use std::{borrow::Cow, marker::PhantomData};
use xterm_core::input::keyboard_event_source::KeyboardEventSource;

/// Null and undefined differ in the source's repeated `which` comparisons.
/// Raw keypress fields are tested for truthiness and strict zero before their
/// eventual numeric conversion. A host conversion may throw or reenter input.
pub enum EventNumber<E = std::convert::Infallible> {
    Undefined,
    Null,
    Number(f64),
    /// A non-number, non-nullish value. Truthiness is sampled without numeric
    /// conversion; strict comparison with the number zero must remain false.
    Other {
        truthy: bool,
        convert: std::rc::Rc<dyn Fn() -> Result<f64, E>>,
    },
}
impl<E> EventNumber<E> {
    pub fn truthy(&self) -> bool {
        match self {
            Self::Undefined | Self::Null => false,
            Self::Number(number) => *number != 0.0 && !number.is_nan(),
            Self::Other { truthy, .. } => *truthy,
        }
    }
    pub fn is_zero(&self) -> bool {
        matches!(self, Self::Number(number) if *number == 0.0)
    }
    pub fn to_number(&self) -> Result<f64, E> {
        match self {
            Self::Number(number) => Ok(*number),
            Self::Null => Ok(0.0),
            Self::Undefined => Ok(f64::NAN),
            Self::Other { convert, .. } => convert(),
        }
    }
}
impl<E> Clone for EventNumber<E> {
    fn clone(&self) -> Self {
        match self {
            Self::Undefined => Self::Undefined,
            Self::Null => Self::Null,
            Self::Number(value) => Self::Number(*value),
            Self::Other { truthy, convert } => Self::Other {
                truthy: *truthy,
                convert: std::rc::Rc::clone(convert),
            },
        }
    }
}
impl<E> std::fmt::Debug for EventNumber<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undefined => f.write_str("Undefined"),
            Self::Null => f.write_str("Null"),
            Self::Number(value) => f.debug_tuple("Number").field(value).finish(),
            Self::Other { truthy, .. } => f
                .debug_struct("Other")
                .field("truthy", truthy)
                .finish_non_exhaustive(),
        }
    }
}
impl<E> PartialEq for EventNumber<E> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Undefined, Self::Undefined) | (Self::Null, Self::Null) => true,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Other { convert: a, .. }, Self::Other { convert: b, .. }) => {
                std::rc::Rc::ptr_eq(a, b)
            }
            _ => false,
        }
    }
}

/// Browser-only fields beside the common encoder's event. Calls are live and
/// fallible; a callback or getter may replace another DOM property between reads.
pub trait KeyEventSource: KeyboardEventSource {
    fn repeat(&self) -> Result<bool, Self::Error>;
    fn char_code(&self) -> Result<EventNumber<Self::Error>, Self::Error>;
    fn which(&self) -> Result<EventNumber<Self::Error>, Self::Error>;
    /// The keypress fallback retains a raw keyCode without changing the
    /// numeric keyCode contract shared with keydown protocol encoders.
    fn key_press_code(&self) -> Result<EventNumber<Self::Error>, Self::Error> {
        self.key_code().map(EventNumber::Number)
    }
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
    fn char_code(&self) -> Result<EventNumber<E>, E> {
        Ok(self
            .event
            .char_code
            .map(|number| EventNumber::Number(f64::from(number)))
            .unwrap_or(EventNumber::Undefined))
    }
    fn which(&self) -> Result<EventNumber<E>, E> {
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
