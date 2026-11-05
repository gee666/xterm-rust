// Licensed under MIT. Raw ordinary-object values shared by keyboard encoders.
use super::number_is_truthy;

/// Addition uses the default hint; template interpolation uses the string hint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Hint {
    Default,
    String,
}

/// An opaque runtime handle. Tests and numeric equality never invoke conversion.
pub trait HostValue<E> {
    fn is_undefined(&self) -> bool;
    fn is_truthy(&self) -> bool;
    /// Return a primitive Number only, without coercing boxed or other values.
    fn number(&self) -> Option<f64>;
    /// Perform the language conversion for this hint and preserve thrown values.
    fn to_utf16(&self, hint: Hint) -> Result<Vec<u16>, E>;
}

/// Portable primitives or an owning foreign runtime handle. Text retains lone
/// surrogates. Hosts can use Host for every value to preserve realm semantics.
pub enum Value<E> {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    Text(Vec<u16>),
    Host(Box<dyn HostValue<E>>),
}

impl<E> Value<E> {
    pub fn is_undefined(&self) -> bool {
        match self {
            Self::Undefined => true,
            Self::Host(value) => value.is_undefined(),
            _ => false,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Undefined | Self::Null => false,
            Self::Boolean(value) => *value,
            Self::Number(value) => number_is_truthy(*value),
            Self::Text(value) => !value.is_empty(),
            Self::Host(value) => value.is_truthy(),
        }
    }

    pub fn is_number(&self, number: f64) -> bool {
        match self {
            Self::Number(value) => *value == number,
            Self::Host(value) => value.number() == Some(number),
            _ => false,
        }
    }

    pub(crate) fn append(&self, output: &mut Vec<u16>, hint: Hint) -> Result<(), E> {
        match self {
            Self::Undefined => output.extend("undefined".encode_utf16()),
            Self::Null => output.extend("null".encode_utf16()),
            Self::Boolean(value) => {
                output.extend(if *value { "true" } else { "false" }.encode_utf16())
            }
            Self::Number(value) => {
                output.extend(ryu_js::Buffer::new().format(*value).encode_utf16())
            }
            Self::Text(value) => output.extend(value),
            Self::Host(value) => output.extend(value.to_utf16(hint)?),
        }
        Ok(())
    }
}
