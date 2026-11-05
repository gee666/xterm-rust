// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FontError {
    Bounds,
    ArrayLength,
    MissingTable(&'static str),
    Signature([u8; 4]),
    UnsupportedFormat(&'static str, u16),
    KernVersion(u16),
    GposVersion { major: u16, minor: u16 },
    GposLookupType(u16),
    GposFormat { offset: usize, kind: u16 },
    CffVersion { major: u8, minor: u8 },
    CoverageFormat { offset: usize },
    Invalid(&'static str),
    Type(&'static str),
    Capacity,
    Compression,
    Borrowed,
    CacheEmptyKey,
}
impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bounds => f.write_str("Font table offset is outside the input"),
            Self::ArrayLength => f.write_str("Invalid array length"),
            Self::MissingTable(table) => write!(f, "Font is missing required {table} table"),
            Self::Signature(signature) => write!(f, "Unsupported font signature {signature:?}"),
            Self::UnsupportedFormat(table, value) => {
                write!(f, "Unsupported {table} format {value}")
            }
            Self::KernVersion(version) => {
                write!(f, "Unsupported kern table version ({version}).")
            }
            Self::CffVersion { major, minor } => {
                write!(f, "Unsupported CFF table version {major}.{minor}")
            }
            Self::GposVersion { major, minor } => write!(
                f,
                "Unsupported GPOS table version {}",
                f64::from(*major) + f64::from(*minor) / 10.0
            ),
            Self::GposLookupType(kind) => write!(f, "GPOS/GSUB lookup type {kind} unknown."),
            Self::GposFormat { offset, kind } => write!(
                f,
                "0x{offset:x}: GPOS lookup type {kind} format must be 1 or 2."
            ),
            Self::CoverageFormat { offset } => {
                write!(f, "0x{offset:x}: Coverage format must be 1 or 2.")
            }
            Self::Invalid(reason) | Self::Type(reason) => f.write_str(reason),
            Self::Capacity => f.write_str("Cannot allocate font table storage"),
            Self::Compression => f.write_str("Invalid compressed WOFF table"),
            Self::Borrowed => f.write_str("A mutable font result is already borrowed"),
            Self::CacheEmptyKey => {
                f.write_str("sizeCalculation return invalid (expect positive integer)")
            }
        }
    }
}
impl std::error::Error for FontError {}
impl From<std::collections::TryReserveError> for FontError {
    fn from(_: std::collections::TryReserveError) -> Self {
        Self::Capacity
    }
}
