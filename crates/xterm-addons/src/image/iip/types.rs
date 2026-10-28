// Copyright (c) 2023 The xterm.js authors. MIT license.
use super::geometry::CellSize;
use crate::image::base64::InvalidSettings;
use crate::image::header::{HeaderFields, HeaderRangeError, HeaderValue};
use crate::image::metrics::ImageMetrics;
use std::collections::TryReserveError;
use std::fmt;

pub const DECODER_KEEP_BYTES: usize = 4_194_304;
pub const DECODER_INITIAL_BYTES: usize = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IipOptions {
    pub pixel_limit: f64,
    pub size_limit: usize,
}
impl Default for IipOptions {
    fn default() -> Self {
        Self {
            pixel_limit: 16_777_216.0,
            size_limit: 33_554_432,
        }
    }
}

/// The host must actually decode and insert the image before completing this
/// request. Owned bytes survive decoder release and any later terminal writes.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayRequest {
    pub bytes: Vec<u8>,
    pub source: ImageMetrics,
    pub target: CellSize,
}
#[derive(Clone, Debug)]
#[must_use = "protocol effects must be applied to the real terminal and image host"]
pub enum IipEffect {
    Report(Vec<u16>),
    Warning(String),
    HostWarning {
        message: String,
        cause: std::rc::Rc<dyn std::error::Error>,
    },
    Display(DisplayRequest),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryPhase {
    CellReport,
    Resize,
}
#[derive(Debug)]
pub(super) enum PendingKind {
    Report,
    Resize(ImageMetrics),
}
#[derive(Debug)]
pub struct PendingEnd {
    pub(super) owner: std::rc::Rc<()>,
    pub(super) kind: PendingKind,
}
impl PendingEnd {
    pub fn phase(&self) -> GeometryPhase {
        match self.kind {
            PendingKind::Report => GeometryPhase::CellReport,
            PendingKind::Resize(_) => GeometryPhase::Resize,
        }
    }
}
#[derive(Debug)]
#[must_use = "complete the requested host geometry phase without holding controller borrows"]
pub enum EndPreparation {
    Complete(Vec<IipEffect>),
    Geometry(PendingEnd),
}

#[derive(Debug)]
pub enum IipError {
    Settings(InvalidSettings),
    Range(HeaderRangeError),
    Allocation(TryReserveError),
    HeaderField(&'static str),
    ForeignEndToken,
}
impl fmt::Display for IipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => error.fmt(f),
            Self::Range(error) => error.fmt(f),
            Self::Allocation(error) => error.fmt(f),
            Self::HeaderField(field) => write!(f, "invalid IIP {field} field type"),
            Self::ForeignEndToken => f.write_str("IIP end token belongs to another handler"),
        }
    }
}
impl std::error::Error for IipError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Settings(e) => Some(e),
            Self::Range(e) => Some(e),
            Self::Allocation(e) => Some(e),
            Self::HeaderField(_) | Self::ForeignEndToken => None,
        }
    }
}
impl From<InvalidSettings> for IipError {
    fn from(value: InvalidSettings) -> Self {
        Self::Settings(value)
    }
}
impl From<HeaderRangeError> for IipError {
    fn from(value: HeaderRangeError) -> Self {
        Self::Range(value)
    }
}
impl From<TryReserveError> for IipError {
    fn from(value: TryReserveError) -> Self {
        Self::Allocation(value)
    }
}

pub(super) fn default_header() -> HeaderFields {
    let mut header = HeaderFields::default();
    for (key, value) in [
        ("type", HeaderValue::Number(0.0)),
        (
            "name",
            HeaderValue::Text("Unnamed file".encode_utf16().collect()),
        ),
        ("size", HeaderValue::Number(0.0)),
        ("width", HeaderValue::Text("auto".encode_utf16().collect())),
        ("height", HeaderValue::Text("auto".encode_utf16().collect())),
        ("preserveAspectRatio", HeaderValue::Number(1.0)),
        ("inline", HeaderValue::Number(0.0)),
    ] {
        header.set(key.encode_utf16().collect(), value);
    }
    header
}
pub(super) fn truthy(value: &HeaderValue) -> bool {
    match value {
        HeaderValue::Number(v) => number_truthy(*v),
        HeaderValue::Text(v) => !v.is_empty(),
        HeaderValue::Null | HeaderValue::Undefined => false,
        HeaderValue::Boolean(v) => *v,
        HeaderValue::Codepoints(_) | HeaderValue::DecoderObject => true,
    }
}
pub(super) fn number_truthy(value: f64) -> bool {
    value != 0.0 && !value.is_nan()
}
pub(super) fn merge_header(fields: &HeaderFields) -> HeaderFields {
    let mut header = default_header();
    for (key, value) in fields.entries() {
        header.set(key.to_vec(), value.clone());
    }
    header
}
