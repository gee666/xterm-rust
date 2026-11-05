// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Container/cmap behavior also follows opentype.js, see licenses/opentype-js-MIT.txt.
use super::{font_cmap::Cmap, font_warning::FontLoadError, FontWarning};
use std::{borrow::Cow, fmt};
#[path = "font_hint_bytes.rs"]
mod hints;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FontError {
    Bounds,
    ArrayLength,
    MissingTable(&'static str),
    Signature([u8; 4]),
    UnsupportedFormat(&'static str, u16),
    KernVersion(u16),
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

pub struct DeferredTable<'a> {
    record: Reader<'a>,
    woff: bool,
}
impl DeferredTable<'_> {
    pub fn offset(&self) -> Result<usize, FontError> {
        self.record.offset32(if self.woff { 4 } else { 8 })
    }

    pub fn load_from<'b>(&self, bytes: &'b [u8]) -> Result<Cow<'b, [u8]>, FontError> {
        // The source copies directory descriptors before callbacks, but retains
        // the live DataView for payload reads. Never reread a mutated directory.
        table_data(Reader::new(bytes), self.record, self.woff, true)
    }
}

pub struct FontTables<'a> {
    pub cmap: Cmap,
    pub gsub: Option<DeferredTable<'a>>,
    pub gdef: Option<DeferredTable<'a>>,
    pub kern: Option<DeferredTable<'a>>,
    pub meta: Option<DeferredTable<'a>>,
    pub fvar: Option<DeferredTable<'a>>,
    pub avar: Option<DeferredTable<'a>>,
    pub glyph_count: Option<u16>,
}
impl<'a> FontTables<'a> {
    pub fn parse_with_refresh<E>(
        bytes: &'a [u8],
        warning: &mut impl FnMut(FontWarning) -> Result<Option<Vec<u8>>, E>,
    ) -> Result<(Self, Option<Vec<u8>>), FontLoadError<E>> {
        let reader = Reader::new(bytes);
        let mut refreshed = None;
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
            return Err(FontError::Signature(signature).into());
        }
        let count = usize::from(reader.u16(if woff { 12 } else { 4 })?);
        let (start, width) = if woff { (44, 20) } else { (12, 16) };
        reader.bytes(start, count.checked_mul(width).ok_or(FontError::Bounds)?)?;
        let mut cmap = None;
        let mut gsub = None;
        let mut gdef = None;
        let mut kern = None;
        let mut meta = None;
        let mut fvar = None;
        let mut avar = None;
        let mut name = None;
        let mut tags = None;
        let mut glyph_count = None;
        let (mut glyf, mut loca, mut cff) = (false, false, false);
        let mut hmtx = None;
        // Missing hhea leaves the source metric count undefined. Its numeric
        // comparison then reads no metrics, the same boundary as zero here.
        let mut metric_count = 0;
        for index in 0..count {
            let record = reader.at(start + index * width)?;
            let tag = record.bytes(0, 4)?;
            match tag {
                b"cvt " | b"fpgm" | b"prep" => {
                    hints::validate(reader, record, woff, tag == b"cvt ")?;
                    continue;
                }
                b"glyf" => glyf = true,
                b"loca" => loca = true,
                b"CFF " | b"CFF2" => cff = true,
                b"name" | b"GSUB" | b"GDEF" | b"hmtx" | b"kern" | b"meta" | b"fvar" | b"avar" => {
                    // These tables are deferred by the source constructor. Select
                    // the final descriptor without touching earlier data, even
                    // when its offset or compressed payload is invalid.
                    match tag {
                        b"name" => name = Some(record),
                        b"GSUB" => gsub = Some(record),
                        b"GDEF" => gdef = Some(record),
                        b"kern" => kern = Some(record),
                        b"meta" => meta = Some(record),
                        b"fvar" => fvar = Some(record),
                        b"avar" => avar = Some(record),
                        _ => hmtx = Some(record),
                    }
                    continue;
                }
                _ => {}
            }
            let metadata = matches!(
                tag,
                b"head"
                    | b"hhea"
                    | b"maxp"
                    | b"OS/2"
                    | b"post"
                    | b"ltag"
                    | b"SVG "
                    | b"CPAL"
                    | b"COLR"
            );
            if tag != b"cmap" && !metadata {
                continue;
            }
            let offset = record.offset32(if woff { 4 } else { 8 })?;
            let payload = Reader::new(refreshed.as_deref().unwrap_or(bytes));
            let table = table_data(payload, record, woff, metadata)?;
            match tag {
                b"cmap" => cmap = Some(Cmap::parse(&table)?),
                b"CPAL" | b"COLR" => {
                    let diagnostic = super::font_color::warning(tag, &table)?;
                    let borrowed = matches!(table, Cow::Borrowed(_));
                    if let Some(diagnostic) = diagnostic {
                        let next = warning(diagnostic).map_err(FontLoadError::Warning)?;
                        if let Some(next) = next {
                            if borrowed {
                                // Header version is already parsed; only later reads see writes.
                                super::font_color::finish(
                                    tag,
                                    &table_data(Reader::new(&next), record, woff, true)?,
                                )?;
                            } else {
                                super::font_color::finish(tag, &table)?;
                            }
                            refreshed = Some(next);
                            continue;
                        }
                    }
                    super::font_color::finish(tag, &table)?;
                }
                b"SVG " => super::font_svg::validate(&table)?,
                b"hhea" => {
                    super::font_metadata::immediate(tag, &table)?;
                    metric_count = Reader::new(&table).u16(34)?;
                }
                b"maxp" => {
                    super::font_metadata::immediate(tag, &table)?;
                    glyph_count = Some(Reader::new(&table).u16(4)?);
                }
                b"ltag" => tags = Some(super::font_metadata::ltag(Reader::new(&table))?),
                b"post" if matches!(&table, Cow::Borrowed(_)) => {
                    // Signed Pascal lengths can read backwards into preceding
                    // tables, since the source retains the full font DataView.
                    super::font_metadata::post(payload, offset)?;
                }
                _ => super::font_metadata::immediate(tag, &table)?,
            }
        }
        let payload = Reader::new(refreshed.as_deref().unwrap_or(bytes));
        let name = table_data(
            payload,
            name.ok_or(FontError::MissingTable("name"))?,
            woff,
            true,
        )?;
        super::font_metadata::name(&name, tags)?;
        if !(glyf && loca || cff) {
            return Err(FontError::Invalid(
                "Font doesn\'t contain TrueType, CFF or CFF2 outlines.",
            )
            .into());
        }
        let hmtx = hmtx.ok_or(FontError::MissingTable("hmtx"))?;
        if glyf && loca {
            validate_hmtx(payload, hmtx, woff, glyph_count.unwrap_or(0), metric_count)?;
        }
        Ok((
            Self {
                cmap: cmap.ok_or(FontError::MissingTable("cmap"))?,
                gdef: gdef.map(|record| DeferredTable { record, woff }),
                gsub: gsub.map(|record| DeferredTable { record, woff }),
                kern: kern.map(|record| DeferredTable { record, woff }),
                meta: meta.map(|record| DeferredTable { record, woff }),
                fvar: fvar.map(|record| DeferredTable { record, woff }),
                avar: avar.map(|record| DeferredTable { record, woff }),
                glyph_count: if glyf && loca {
                    Some(glyph_count.unwrap_or(0))
                } else {
                    None
                },
            },
            refreshed,
        ))
    }
}

const HORIZONTAL_METRIC_SIZE: usize = 4;
fn validate_hmtx(
    reader: Reader<'_>,
    record: Reader<'_>,
    woff: bool,
    glyph_count: u16,
    metric_count: u16,
) -> Result<(), FontError> {
    let required = usize::from(glyph_count.min(metric_count)) * HORIZONTAL_METRIC_SIZE;
    if woff && record.offset32(8)? < record.offset32(12)? {
        // Source decompression precedes the glyph loop, including a loop that
        // reads no metric bytes. Do not turn those failures into success.
        let bytes = table_data(reader, record, woff, true)?;
        Reader::new(&bytes).bytes(0, required)?;
    } else if required != 0 {
        // Uncompressed offsets are never dereferenced by a zero-read loop.
        // Directory lengths do not constrain the source's original DataView.
        reader.bytes(record.offset32(if woff { 4 } else { 8 })?, required)?;
    }
    Ok(())
}

fn table_data<'a>(
    reader: Reader<'a>,
    record: Reader<'_>,
    woff: bool,
    metadata: bool,
) -> Result<Cow<'a, [u8]>, FontError> {
    let offset = record.offset32(if woff { 4 } else { 8 })?;
    let length = record.offset32(12)?;
    if woff {
        let compressed = record.offset32(8)?;
        if compressed < length {
            let input = reader.bytes(offset, compressed)?;
            // opentype skips the two-byte zlib wrapper and ignores its Adler32
            // trailer. Buffer aliases can change those bytes before this load.
            let deflate = input.get(2..).ok_or(FontError::Bounds)?;
            let inflated = miniz_oxide::inflate::decompress_to_vec_with_limit(deflate, length)
                .map_err(|_| FontError::Compression)?;
            if inflated.len() != length {
                return Err(FontError::Compression);
            }
            return Ok(Cow::Owned(inflated));
        }
    }
    // Uncompressed metadata uses the original DataView, not directory lengths.
    // Later records may satisfy reads beyond the nominal end of a table.
    Ok(Cow::Borrowed(reader.bytes(
        offset,
        if metadata {
            reader.bytes.len().saturating_sub(offset)
        } else {
            length
        },
    )?))
}
