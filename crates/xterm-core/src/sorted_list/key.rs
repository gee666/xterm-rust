//! Primitive key operators are context-specific, not one numeric projection.
use super::SortedListError;
use crate::buffer::marker_line::MarkerLine;

#[derive(Clone, Debug, PartialEq)]
pub enum SortedKey {
    Undefined,
    Value(MarkerLine),
}
impl SortedKey {
    pub fn number(value: f64) -> Self {
        Self::Value(MarkerLine::number(value))
    }
    pub fn string(value: Vec<u16>) -> Self {
        Self::Value(MarkerLine::string(value))
    }
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }
    pub fn numeric_difference(&self, rhs: &Self) -> f64 {
        match (self, rhs) {
            (Self::Value(left), Self::Value(right)) => left.to_number() - right.to_number(),
            _ => f64::NAN,
        }
    }
    pub fn strict_eq(&self, rhs: &Self) -> bool {
        self == rhs
    }
    pub fn rel_lt(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Value(left), Self::Value(right)) => left.rel_lt(right),
            _ => false,
        }
    }
    pub fn rel_gt(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Value(left), Self::Value(right)) => left.rel_gt(right),
            _ => false,
        }
    }
    pub fn rel_le(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Value(left), Self::Value(right)) => left.rel_le(right),
            _ => false,
        }
    }
}
impl From<f64> for SortedKey {
    fn from(value: f64) -> Self {
        Self::number(value)
    }
}
impl From<MarkerLine> for SortedKey {
    fn from(value: MarkerLine) -> Self {
        Self::Value(value)
    }
}
impl From<&str> for SortedKey {
    fn from(value: &str) -> Self {
        Self::string(value.encode_utf16().collect())
    }
}

/// `None` means the actual JS undefined argument, including exhausted-array reads.
pub trait KeyReader<T> {
    fn read_key(&self, value: Option<&T>) -> Result<SortedKey, SortedListError>;
    fn read_undefined(&self, represented: Option<&T>) -> Result<SortedKey, SortedListError> {
        self.read_key(represented)
    }
}
impl<T, F: Fn(&T) -> f64> KeyReader<T> for F {
    fn read_key(&self, value: Option<&T>) -> Result<SortedKey, SortedListError> {
        Ok(value.map_or(SortedKey::Undefined, |value| SortedKey::number(self(value))))
    }
}
/// Explicit wrapper avoids overlapping a nonthrowing native f64 closure with
/// a source getter whose undefined or ordinary reads can throw typed failures.
pub struct TryKey<F>(pub F);
impl<F> TryKey<F> {
    pub fn new(reader: F) -> Self {
        Self(reader)
    }
}
impl<T, F: Fn(Option<&T>) -> Result<SortedKey, SortedListError>> KeyReader<T> for TryKey<F> {
    fn read_key(&self, value: Option<&T>) -> Result<SortedKey, SortedListError> {
        (self.0)(value)
    }
    fn read_undefined(&self, _: Option<&T>) -> Result<SortedKey, SortedListError> {
        (self.0)(None)
    }
}
