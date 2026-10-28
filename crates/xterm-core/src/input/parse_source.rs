//! Retained parse input for bounded decoding and observable diagnostics. Host
//! sources read their live data only when requested; they must not eagerly copy
//! a mutable byte view before logging or an asynchronous parser pause.
use crate::log_service::LogValue;
use std::{borrow::Cow, error::Error, fmt, ops::Range, rc::Rc};

#[derive(Clone, Debug)]
pub struct ParseSourceError(Rc<dyn Error>);
impl ParseSourceError {
    pub fn new(error: impl Error + 'static) -> Self {
        Self(Rc::new(error))
    }
}
impl fmt::Display for ParseSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.0.as_ref(), f)
    }
}
impl Error for ParseSourceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_ref())
    }
}

pub enum ParseChunk<'a> {
    Text(Cow<'a, [u16]>),
    Bytes(Cow<'a, [u8]>),
}

/// Lengths and ranges are input units: UTF-16 for text and bytes for byte input.
/// The source's kind stays constant. Release a returned chunk after decoding,
/// before invoking parser/user callbacks. Retain the Rc source across pauses.
pub trait ParseInputSource {
    fn len(&self) -> Result<usize, ParseSourceError>;
    fn is_empty(&self) -> Result<bool, ParseSourceError> {
        self.len().map(|length| length == 0)
    }
    /// Positive substring/subarray bounds clamp as JavaScript does. A host view
    /// performs its read now, not at write enqueue or before a prior callback.
    fn read_range(&self, range: Range<usize>) -> Result<ParseChunk<'_>, ParseSourceError>;
    /// Small writes pass the original view to the decoder without calling its
    /// subarray method. A detached zero-length byte view must remain empty here.
    fn read_all(&self) -> Result<ParseChunk<'_>, ParseSourceError>;
    /// Full input before the Debug call, without interpreting UTF-8 bytes.
    fn debug_units(&self) -> Result<Vec<u16>, ParseSourceError>;
    /// Text returns an eager code-unit array. A browser byte source returns its
    /// original JS Uint8Array as an opaque value, not a copied numeric array.
    fn trace_value(&self) -> Result<LogValue, ParseSourceError>;
}

/// Native owned input. Slice-based terminal methods can copy into this owner
/// because async parsing may outlive the call; callers can avoid that copy by
/// moving their Vec into this enum and passing an Rc source directly.
#[derive(Clone, Debug)]
pub enum OwnedParseInput {
    Text(Vec<u16>),
    Bytes(Vec<u8>),
}
impl ParseInputSource for Vec<u16> {
    fn len(&self) -> Result<usize, ParseSourceError> {
        Ok(Vec::len(self))
    }
    fn read_range(&self, range: Range<usize>) -> Result<ParseChunk<'_>, ParseSourceError> {
        let (start, end) = (
            range.start.min(Vec::len(self)),
            range.end.min(Vec::len(self)),
        );
        Ok(ParseChunk::Text(Cow::Borrowed(
            &self[start.min(end)..start.max(end)],
        )))
    }
    fn read_all(&self) -> Result<ParseChunk<'_>, ParseSourceError> {
        Ok(ParseChunk::Text(Cow::Borrowed(self)))
    }
    fn debug_units(&self) -> Result<Vec<u16>, ParseSourceError> {
        Ok(self.clone())
    }
    fn trace_value(&self) -> Result<LogValue, ParseSourceError> {
        Ok(LogValue::CodeUnits(self.clone()))
    }
}
impl ParseInputSource for Vec<u8> {
    fn len(&self) -> Result<usize, ParseSourceError> {
        Ok(Vec::len(self))
    }
    fn read_range(&self, range: Range<usize>) -> Result<ParseChunk<'_>, ParseSourceError> {
        let start = range.start.min(Vec::len(self));
        let end = range.end.min(Vec::len(self)).max(start);
        Ok(ParseChunk::Bytes(Cow::Borrowed(&self[start..end])))
    }
    fn read_all(&self) -> Result<ParseChunk<'_>, ParseSourceError> {
        Ok(ParseChunk::Bytes(Cow::Borrowed(self)))
    }
    fn debug_units(&self) -> Result<Vec<u16>, ParseSourceError> {
        Ok(self.iter().copied().map(u16::from).collect())
    }
    fn trace_value(&self) -> Result<LogValue, ParseSourceError> {
        Ok(LogValue::Bytes(self.clone()))
    }
}
impl ParseInputSource for OwnedParseInput {
    fn len(&self) -> Result<usize, ParseSourceError> {
        match self {
            Self::Text(data) => ParseInputSource::len(data),
            Self::Bytes(data) => ParseInputSource::len(data),
        }
    }
    fn read_range(&self, range: Range<usize>) -> Result<ParseChunk<'_>, ParseSourceError> {
        match self {
            Self::Text(data) => data.read_range(range),
            Self::Bytes(data) => data.read_range(range),
        }
    }
    fn read_all(&self) -> Result<ParseChunk<'_>, ParseSourceError> {
        match self {
            Self::Text(data) => data.read_all(),
            Self::Bytes(data) => data.read_all(),
        }
    }
    fn debug_units(&self) -> Result<Vec<u16>, ParseSourceError> {
        match self {
            Self::Text(data) => data.debug_units(),
            Self::Bytes(data) => data.debug_units(),
        }
    }
    fn trace_value(&self) -> Result<LogValue, ParseSourceError> {
        match self {
            Self::Text(data) => data.trace_value(),
            Self::Bytes(data) => data.trace_value(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_ranges_use_utf16_substring_clamping_and_swap() {
        let input = OwnedParseInput::Text(vec![0, 0xd800, 0xd83d, 0xde00]);
        let ParseChunk::Text(chunk) = input.read_range(Range { start: 3, end: 1 }).unwrap() else {
            panic!("wrong input kind")
        };
        assert_eq!(&*chunk, &[0xd800, 0xd83d]);
        let ParseChunk::Text(chunk) = input.read_range(2..usize::MAX).unwrap() else {
            panic!("wrong input kind")
        };
        assert_eq!(&*chunk, &[0xd83d, 0xde00]);
        assert_eq!(input.debug_units().unwrap(), [0, 0xd800, 0xd83d, 0xde00]);
        assert_eq!(
            input.trace_value().unwrap(),
            LogValue::CodeUnits(vec![0, 0xd800, 0xd83d, 0xde00])
        );
    }
    #[test]
    fn bytes_keep_all_values_and_subarray_does_not_swap_bounds() {
        let bytes: Vec<_> = (0..=255_u8).collect();
        let input = OwnedParseInput::Bytes(bytes.clone());
        assert_eq!(input.len().unwrap(), 256);
        assert_eq!(
            input.debug_units().unwrap(),
            (0..=255_u16).collect::<Vec<_>>()
        );
        assert_eq!(input.trace_value().unwrap(), LogValue::Bytes(bytes));
        let ParseChunk::Bytes(chunk) = input.read_range(Range { start: 10, end: 3 }).unwrap()
        else {
            panic!("wrong input kind")
        };
        assert!(chunk.is_empty());
        let ParseChunk::Bytes(chunk) = input.read_range(255..usize::MAX).unwrap() else {
            panic!("wrong input kind")
        };
        assert_eq!(&*chunk, &[255]);
        assert!(OwnedParseInput::Text(vec![]).is_empty().unwrap());
        assert!(OwnedParseInput::Bytes(vec![]).is_empty().unwrap());
    }
    #[test]
    fn host_errors_keep_the_original_typed_cause() {
        let error = ParseSourceError::new(std::io::Error::other("view getter failed"));
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .unwrap()
                .to_string(),
            "view getter failed"
        );
    }
}
