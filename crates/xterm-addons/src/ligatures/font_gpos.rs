// Constructor parsing follows opentype.js, see licenses/opentype-js-MIT.txt.
use super::{
    font_bytes::{DeferredTable, Reader},
    FontError, FontLoadError, FontWarning,
};
use std::borrow::Cow;
#[path = "font_gpos_layout.rs"]
mod layout;

type Result<T, E> = std::result::Result<T, FontLoadError<E>>;

pub fn validate<E>(
    table: &DeferredTable<'_>,
    bytes: &[u8],
    refreshed: &mut Option<Vec<u8>>,
    warning: &mut impl FnMut(FontWarning) -> std::result::Result<Option<Vec<u8>>, E>,
) -> Result<(), E> {
    let inflated = match table.load_from(refreshed.as_deref().unwrap_or(bytes))? {
        Cow::Owned(bytes) => Some(bytes),
        Cow::Borrowed(_) => None,
    };
    let start = if inflated.is_some() {
        0
    } else {
        table.offset()? as u64
    };
    let mut context = Context {
        bytes,
        refreshed,
        inflated,
        warning,
    };
    let major = context.word(start)?;
    let minor = context.word(start + 2)?;
    let version = u32::from(major) * 10 + u32::from(minor);
    if !matches!(version, 10 | 11) {
        return Err(FontError::GposVersion { major, minor }.into());
    }
    let scripts = context.scripts(start)?;
    let features = context.features(start)?;
    let lookups = context.lookups(start)?;
    if version == 11 {
        if let Some(at) = context.pointer32(start, 10)? {
            let major = context.word(at)?;
            let minor = context.word(at + 2)?;
            if major != 1 || minor != 0 {
                return Err(
                    FontError::Invalid("GPOS/GSUB feature variations table unknown.").into(),
                );
            }
            let count = context.long(at + 4)?;
            // Source retains these records but does not follow either offset.
            context.span(at + 8, u64::from(count) * 8)?;
        }
    }
    layout::initialize(scripts, features, lookups).map_err(FontLoadError::Font)
}

struct Context<'a, 'b, F> {
    bytes: &'a [u8],
    refreshed: &'b mut Option<Vec<u8>>,
    inflated: Option<Vec<u8>>,
    warning: &'b mut F,
}
impl<E, F: FnMut(FontWarning) -> std::result::Result<Option<Vec<u8>>, E>> Context<'_, '_, F> {
    fn reader(&self) -> Reader<'_> {
        Reader::new(
            self.inflated
                .as_deref()
                .unwrap_or_else(|| self.refreshed.as_deref().unwrap_or(self.bytes)),
        )
    }
    // Logical offsets can exceed usize on wasm32 without ever being read, for
    // example an ignored type9 subtable. Convert only at an actual DataView read.
    fn word(&self, at: u64) -> std::result::Result<u16, FontError> {
        self.reader()
            .u16(usize::try_from(at).map_err(|_| FontError::Bounds)?)
    }
    fn long(&self, at: u64) -> std::result::Result<u32, FontError> {
        self.reader()
            .u32(usize::try_from(at).map_err(|_| FontError::Bounds)?)
    }
    fn span(&self, at: u64, count: u64) -> std::result::Result<(), FontError> {
        if count != 0 {
            self.reader().bytes(
                usize::try_from(at).map_err(|_| FontError::Bounds)?,
                usize::try_from(count).map_err(|_| FontError::Bounds)?,
            )?;
        }
        Ok(())
    }
    fn pointer(&self, base: u64, field: u64) -> std::result::Result<Option<u64>, FontError> {
        let offset = self.word(base + field)?;
        Ok((offset != 0).then_some(base + u64::from(offset)))
    }
    fn pointer32(&self, base: u64, field: u64) -> std::result::Result<Option<u64>, FontError> {
        let offset = self.long(base + field)?;
        Ok((offset != 0).then_some(base + u64::from(offset)))
    }
    fn coverage(&self, at: u64) -> std::result::Result<(), FontError> {
        let format = self.word(at)?;
        let count = self.word(at + 2)?;
        let width = match format {
            1 => 2,
            2 => 6,
            _ => {
                return Err(FontError::CoverageFormat {
                    offset: usize::try_from(at).map_err(|_| FontError::Bounds)?,
                })
            }
        };
        self.span(at + 4, u64::from(count) * width)
    }
    fn class(&mut self, at: u64) -> Result<(), E> {
        let format = self.word(at)?;
        match format {
            1 => {
                self.word(at + 2)?;
                self.span(at + 6, u64::from(self.word(at + 4)?) * 2)?;
            }
            2 => self.span(at + 4, u64::from(self.word(at + 2)?) * 6)?,
            _ => {
                let next = (self.warning)(FontWarning::InvalidClassDefinition {
                    offset: usize::try_from(at).map_err(|_| FontError::Bounds)?,
                    format,
                })
                .map_err(FontLoadError::Warning)?;
                if next.is_some() {
                    *self.refreshed = next;
                }
            }
        }
        Ok(())
    }
    fn values(&self, at: u64, format: u16, count: u64) -> std::result::Result<u64, FontError> {
        // Only low eight bits cause reads. Device offsets are read as shorts,
        // never followed by this source parser.
        let size = u64::from((format & 255).count_ones()) * 2;
        self.span(at, size * count)?;
        Ok(at + size * count)
    }
    fn subtable(&mut self, at: u64, kind: u16) -> Result<(), E> {
        // Source types3..9 return an error object without reading the payload.
        // The constructor still succeeds, since the ligature engine ignores it.
        if kind >= 3 {
            return Ok(());
        }
        let format = self.word(at)?;
        if !matches!(format, 1 | 2) {
            return Err(FontError::GposFormat {
                offset: usize::try_from(at).map_err(|_| FontError::Bounds)?,
                kind,
            }
            .into());
        }
        if let Some(coverage) = self.pointer(at, 2)? {
            self.coverage(coverage)?;
        }
        if kind == 1 {
            let value_format = self.word(at + 4)?;
            if format == 1 {
                self.values(at + 6, value_format, 1)?;
            } else {
                self.values(at + 8, value_format, u64::from(self.word(at + 6)?))?;
            }
            return Ok(());
        }
        let first = self.word(at + 4)?;
        let second = self.word(at + 6)?;
        if format == 1 {
            let count = self.word(at + 8)?;
            for index in 0..count {
                if let Some(pair) = self.pointer(at, 10 + u64::from(index) * 2)? {
                    let count = self.word(pair)?;
                    let width = 2 + u64::from(
                        ((first & 255).count_ones() + (second & 255).count_ones()) * 2,
                    );
                    self.span(pair + 2, u64::from(count) * width)?;
                }
            }
        } else {
            if let Some(class) = self.pointer(at, 8)? {
                self.class(class)?;
            }
            if let Some(class) = self.pointer(at, 10)? {
                self.class(class)?;
            }
            let rows = self.word(at + 12)?;
            let columns = self.word(at + 14)?;
            let count = u64::from(rows) * u64::from(columns);
            let width = u64::from(((first & 255).count_ones() + (second & 255).count_ones()) * 2);
            // Zero-width records require no payload, even for 65535² entries.
            self.span(at + 16, count * width)?;
        }
        Ok(())
    }
}
