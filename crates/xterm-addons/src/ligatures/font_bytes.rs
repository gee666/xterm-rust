// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Container/cmap behavior also follows opentype.js, see licenses/opentype-js-MIT.txt.
use std::{borrow::Cow, fmt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FontError {
    Bounds,
    MissingTable(&'static str),
    Signature([u8; 4]),
    UnsupportedFormat(&'static str, u16),
    Invalid(&'static str),
    Capacity,
    Compression,
    Borrowed,
    CacheEmptyKey,
}
impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bounds => f.write_str("Font table offset is outside the input"),
            Self::MissingTable(table) => write!(f, "Font is missing required {table} table"),
            Self::Signature(signature) => write!(f, "Unsupported font signature {signature:?}"),
            Self::UnsupportedFormat(table, value) => {
                write!(f, "Unsupported {table} format {value}")
            }
            Self::Invalid(reason) => f.write_str(reason),
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

#[derive(Clone, Copy)]
pub struct Reader<'a> {
    bytes: &'a [u8],
}
impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
    pub fn bytes(&self, offset: usize, length: usize) -> Result<&'a [u8], FontError> {
        let end = offset.checked_add(length).ok_or(FontError::Bounds)?;
        self.bytes.get(offset..end).ok_or(FontError::Bounds)
    }
    pub fn at(&self, offset: usize) -> Result<Self, FontError> {
        Ok(Self::new(
            self.bytes.get(offset..).ok_or(FontError::Bounds)?,
        ))
    }
    pub fn u16(&self, offset: usize) -> Result<u16, FontError> {
        let bytes = self.bytes(offset, 2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }
    pub fn i16(&self, offset: usize) -> Result<i16, FontError> {
        let bytes = self.bytes(offset, 2)?;
        Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
    }
    pub fn u32(&self, offset: usize) -> Result<u32, FontError> {
        let bytes = self.bytes(offset, 4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
    pub fn offset32(&self, offset: usize) -> Result<usize, FontError> {
        usize::try_from(self.u32(offset)?).map_err(|_| FontError::Bounds)
    }
    pub fn child16(&self, offset: usize) -> Result<Self, FontError> {
        self.at(usize::from(self.u16(offset)?))
    }
    pub fn words(&self, offset: usize, count: usize) -> Result<Vec<u16>, FontError> {
        let bytes = self.bytes(offset, count.checked_mul(2).ok_or(FontError::Bounds)?)?;
        let mut values = Vec::new();
        values.try_reserve_exact(count)?;
        values.extend(
            bytes
                .chunks_exact(2)
                .map(|pair| u16::from_be_bytes([pair[0], pair[1]])),
        );
        Ok(values)
    }
}

pub struct FontTables<'a> {
    pub cmap: Cow<'a, [u8]>,
    pub gsub: Option<Cow<'a, [u8]>>,
}
impl<'a> FontTables<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, FontError> {
        let reader = Reader::new(bytes);
        let signature_bytes = reader.bytes(0, 4)?;
        let signature = [
            signature_bytes[0],
            signature_bytes[1],
            signature_bytes[2],
            signature_bytes[3],
        ];
        let woff = signature == *b"wOFF";
        let flavor = if woff {
            reader.bytes(4, 4)?
        } else {
            signature_bytes
        };
        if !matches!(flavor, [0, 1, 0, 0] | [b'O', b'T', b'T', b'O'])
            && (woff || !matches!(flavor, [b't', b'r', b'u', b'e'] | [b't', b'y', b'p', b'1']))
        {
            return Err(FontError::Signature(signature));
        }
        let count = usize::from(reader.u16(if woff { 12 } else { 4 })?);
        let (start, width) = if woff { (44, 20) } else { (12, 16) };
        reader.bytes(start, count.checked_mul(width).ok_or(FontError::Bounds)?)?;
        let mut cmap = None;
        let mut gsub = None;
        for index in 0..count {
            let record = reader.at(start + index * width)?;
            let tag = record.bytes(0, 4)?;
            if tag != b"cmap" && tag != b"GSUB" {
                continue;
            }
            let offset = record.offset32(if woff { 4 } else { 8 })?;
            let length = record.offset32(12)?;
            let table = if woff {
                let compressed = record.offset32(8)?;
                if compressed < length {
                    let input = reader.bytes(offset, compressed)?;
                    let inflated =
                        miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(input, length)
                            .map_err(|_| FontError::Compression)?;
                    if inflated.len() != length {
                        return Err(FontError::Compression);
                    }
                    Cow::Owned(inflated)
                } else {
                    Cow::Borrowed(reader.bytes(offset, length)?)
                }
            } else {
                Cow::Borrowed(reader.bytes(offset, length)?)
            };
            if tag == b"cmap" {
                cmap = Some(table);
            } else {
                gsub = Some(table);
            }
        }
        Ok(Self {
            cmap: cmap.ok_or(FontError::MissingTable("cmap"))?,
            gsub,
        })
    }
}
