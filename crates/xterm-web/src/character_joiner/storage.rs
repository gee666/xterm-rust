//! Typed and browser ranges share traversal, but only browser values can throw
//! during conversion. Keeping them raw preserves host mutation and identity.
use super::{merge_ranges, string_ranges_to_cell_ranges, JoinRange, JoinerError, LineRef};
use std::convert::Infallible;

/// Range operations have two error phases. Handler and merge failures are
/// logged and later handlers continue; conversion and append failures escape.
/// Implementations must release line borrows before entering host callbacks.
pub trait RangeStorage: Sized {
    type HandlerError;
    type ConversionError;
    fn empty() -> Self;
    fn missing_handler(text: &[u16]) -> Result<Self, Self::HandlerError>;
    fn merge(&mut self, other: Self) -> Result<(), Self::HandlerError>;
    fn convert(&mut self, line: &LineRef, start: usize) -> Result<(), Self::ConversionError>;
    fn append(&mut self, other: Self) -> Result<(), Self::ConversionError>;
}
impl RangeStorage for Vec<JoinRange> {
    type HandlerError = JoinerError;
    type ConversionError = Infallible;
    fn empty() -> Self {
        Vec::new()
    }
    fn missing_handler(_: &[u16]) -> Result<Self, JoinerError> {
        Err(JoinerError(
            "Character joiner was removed before the next segment".into(),
        ))
    }
    fn merge(&mut self, other: Self) -> Result<(), JoinerError> {
        for range in other {
            merge_ranges(self, range);
        }
        Ok(())
    }
    fn convert(&mut self, line: &LineRef, start: usize) -> Result<(), Infallible> {
        string_ranges_to_cell_ranges(self, &line.borrow(), start);
        Ok(())
    }
    fn append(&mut self, other: Self) -> Result<(), Infallible> {
        self.extend(other);
        Ok(())
    }
}
