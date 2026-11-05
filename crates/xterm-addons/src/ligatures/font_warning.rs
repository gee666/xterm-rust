//! Constructor diagnostics are synchronous so hosts can preserve callback order.
use super::FontError;
use std::fmt;

/// A diagnostic produced at the same read phase as the source constructor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontWarning {
    /// The Mac kern constructor reads only the first subtable.
    MultipleKernSubtables,
}
impl FontWarning {
    pub fn message(self) -> &'static str {
        match self {
            Self::MultipleKernSubtables => "Only the first kern subtable is supported.",
        }
    }
}

/// Distinguishes malformed font data from a host diagnostic callback failure.
#[derive(Debug, Eq, PartialEq)]
pub enum FontLoadError<E> {
    Font(FontError),
    Warning(E),
}
impl<E> From<FontError> for FontLoadError<E> {
    fn from(error: FontError) -> Self {
        Self::Font(error)
    }
}
impl<E: fmt::Display> fmt::Display for FontLoadError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Font(error) => error.fmt(f),
            Self::Warning(error) => error.fmt(f),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for FontLoadError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Font(error) => error,
            Self::Warning(error) => error,
        })
    }
}
