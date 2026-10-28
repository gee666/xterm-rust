// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
//! One portable JavaScript Number/String value for marker line arithmetic.
//!
//! Public wrapper validation and object coercion precede construction. Keeping
//! the raw primitive prevents numeric-looking strings from becoming integers
//! before source addition, strict equality or Map-key lookup.
use crate::options::{OptionError, OptionValue};
use std::{cmp::Ordering, fmt};

#[derive(Debug)]
pub enum MarkerLineError {
    NotNumberOrString,
    Coercion(OptionError),
}
impl fmt::Display for MarkerLineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotNumberOrString => {
                f.write_str("marker line must be a Number or UTF-16 String primitive")
            }
            Self::Coercion(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for MarkerLineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Coercion(error) => Some(error),
            Self::NotNumberOrString => None,
        }
    }
}
impl From<OptionError> for MarkerLineError {
    fn from(error: OptionError) -> Self {
        Self::Coercion(error)
    }
}

/// The private field enforces the Number/String invariant at every constructor.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkerLine(OptionValue);

impl MarkerLine {
    pub fn number(value: f64) -> Self {
        Self(OptionValue::Number(value))
    }
    pub fn string(value: Vec<u16>) -> Self {
        Self(OptionValue::String(value))
    }
    pub fn as_value(&self) -> &OptionValue {
        &self.0
    }
    pub fn into_value(self) -> OptionValue {
        self.0
    }
    pub fn as_number(&self) -> Option<f64> {
        match &self.0 {
            OptionValue::Number(value) => Some(*value),
            _ => None,
        }
    }
    pub fn as_string(&self) -> Option<&[u16]> {
        match &self.0 {
            OptionValue::String(value) => Some(value),
            _ => None,
        }
    }
    pub fn to_number(&self) -> f64 {
        // Only Number/String values can be stored here. Their conversion never
        // invokes user code or throws; invalid numeric strings yield NaN.
        self.0
            .to_number()
            .expect("primitive Number/String conversion cannot fail")
    }
    pub fn to_utf16(&self) -> Vec<u16> {
        // Primitive conversion is infallible and preserves lone string units.
        self.0
            .to_utf16()
            .expect("primitive Number/String formatting cannot fail")
    }
    pub fn strict_eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
    pub fn strict_eq_number(&self, rhs: f64) -> bool {
        self.as_number().is_some_and(|value| value == rhs)
    }
    fn relational(&self, rhs: &Self) -> Option<Ordering> {
        match (self.as_string(), rhs.as_string()) {
            (Some(left), Some(right)) => Some(left.cmp(right)),
            _ => self.to_number().partial_cmp(&rhs.to_number()),
        }
    }
    pub fn rel_lt(&self, rhs: &Self) -> bool {
        self.relational(rhs) == Some(Ordering::Less)
    }
    pub fn rel_gt(&self, rhs: &Self) -> bool {
        self.relational(rhs) == Some(Ordering::Greater)
    }
    pub fn rel_le(&self, rhs: &Self) -> bool {
        matches!(self.relational(rhs), Some(Ordering::Less | Ordering::Equal))
    }
    pub fn rel_ge(&self, rhs: &Self) -> bool {
        matches!(
            self.relational(rhs),
            Some(Ordering::Greater | Ordering::Equal)
        )
    }
    pub fn add_number(&self, rhs: f64) -> Result<Self, MarkerLineError> {
        self.add_value(&OptionValue::Number(rhs))
    }
    /// General addition also supports the captured decoration-height operand.
    /// Host object conversion errors remain in the original source chain.
    pub fn add_value(&self, rhs: &OptionValue) -> Result<Self, MarkerLineError> {
        Self::try_from(self.0.js_add(rhs)?)
    }
    pub fn subtract_number(&self, rhs: f64) -> Self {
        Self::number(self.to_number() - rhs)
    }
    pub fn increment(&self) -> Self {
        Self::number(self.to_number() + 1.0)
    }
    pub fn map_key(&self) -> MarkerLineKey {
        MarkerLineKey(match &self.0 {
            OptionValue::Number(value) => Key::Number(if value.is_nan() {
                f64::NAN.to_bits()
            } else if *value == 0.0 {
                0
            } else {
                value.to_bits()
            }),
            OptionValue::String(value) => Key::String(value.clone()),
            _ => unreachable!("validated Number/String marker line"),
        })
    }
}
impl TryFrom<OptionValue> for MarkerLine {
    type Error = MarkerLineError;
    fn try_from(value: OptionValue) -> Result<Self, Self::Error> {
        match value {
            OptionValue::Number(_) | OptionValue::String(_) => Ok(Self(value)),
            _ => Err(MarkerLineError::NotNumberOrString),
        }
    }
}
impl From<f64> for MarkerLine {
    fn from(value: f64) -> Self {
        Self::number(value)
    }
}
impl From<Vec<u16>> for MarkerLine {
    fn from(value: Vec<u16>) -> Self {
        Self::string(value)
    }
}
impl From<&str> for MarkerLine {
    fn from(value: &str) -> Self {
        Self::string(value.encode_utf16().collect())
    }
}

// These are JavaScript Number conversions, not exact storage of arbitrary Rust
// integers. Native buffer coordinates are bounded by u32 and remain exact.
macro_rules! integer_adapter {
    ($($kind:ty),+ $(,)?) => { $(impl From<$kind> for MarkerLine {
        fn from(value: $kind) -> Self { Self::number(value as f64) }
    })+ };
}
integer_adapter!(i32, i64, u32, usize);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    Number(u64),
    String(Vec<u16>),
}

/// JavaScript Map SameValueZero, not strict equality or insertion ordering.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MarkerLineKey(Key);
impl MarkerLineKey {
    pub fn to_line(&self) -> MarkerLine {
        match &self.0 {
            Key::Number(bits) => MarkerLine::number(f64::from_bits(*bits)),
            Key::String(value) => MarkerLine::string(value.clone()),
        }
    }
}
