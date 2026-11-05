//! Constructor diagnostics are synchronous so hosts can preserve callback order.
use super::FontError;
use std::{borrow::Cow, fmt};

/// A diagnostic produced at the same read phase as the source constructor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontWarning {
    /// The Mac kern constructor reads only the first subtable.
    MultipleKernSubtables,
    /// The parser warns and retains an otherwise unsupported ClassDef value.
    InvalidClassDefinition { offset: usize, format: u16 },
    /// An avar table was supplied without the required parsed fvar axes.
    AvarMissingFvar,
    /// Unsupported major versions still parse after the diagnostic.
    AvarVersion { major: u16, minor: u16 },
}
impl FontWarning {
    pub fn message(self) -> Cow<'static, str> {
        match self {
            Self::MultipleKernSubtables => "Only the first kern subtable is supported.".into(),
            Self::InvalidClassDefinition { offset, format } => format!(
                "0x{offset:x}: This font file uses an invalid ClassDef format of {format}. It might be corrupted and should be reacquired if it doesn't display as intended."
            ).into(),
            Self::AvarMissingFvar => "This font provides an avar table, but no fvar table, which is required for variable fonts.".into(),
            Self::AvarVersion { major, minor } => format!("Unsupported avar table version {major}.{minor}").into(),
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
