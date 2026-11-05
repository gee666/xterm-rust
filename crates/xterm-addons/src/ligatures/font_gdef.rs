// Constructor reads follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::{
    font_bytes::{DeferredTable, Reader},
    FontError, FontLoadError, FontWarning,
};
use std::borrow::Cow;

/// GDEF is not used for ligature lookup, but its constructor reads and warnings
/// still determine whether loading succeeds. Keep the original DataView offsets
/// for diagnostics and refresh only uncompressed payloads after callbacks.
pub fn validate<E>(
    table: &DeferredTable<'_>,
    bytes: &[u8],
    refreshed: &mut Option<Vec<u8>>,
    warning: &mut impl FnMut(FontWarning) -> Result<Option<Vec<u8>>, E>,
) -> Result<(), FontLoadError<E>> {
    let inflated = match table.load_from(refreshed.as_deref().unwrap_or(bytes))? {
        Cow::Owned(data) => Some(data),
        Cow::Borrowed(_) => None,
    };
    let start = if inflated.is_some() {
        0
    } else {
        table.offset()?
    };
    let mut context = Context {
        bytes,
        refreshed,
        inflated,
        warning,
    };
    let major = context.reader().u16(start)?;
    let minor = context.reader().u16(start + 2)?;
    // parseVersion(1) computes major + minor / 10, including unusual 0.12/0.13.
    let version = u32::from(major) * 10 + u32::from(minor);
    if !matches!(version, 10 | 12 | 13) {
        return Err(FontError::Invalid("Unsupported GDEF table version.").into());
    }
    for (position, kind) in [
        (4, Kind::Class),
        (6, Kind::Attach),
        (8, Kind::Caret),
        (10, Kind::Class),
    ] {
        context.pointer(start, position, kind)?;
    }
    if version >= 12 {
        context.pointer(start, 12, Kind::MarkSets)?;
    }
    // Version1.3's variation-store offset and CaretValue3's device offset are
    // deliberately unread by this source version, even when truncated/invalid.
    Ok(())
}

#[derive(Clone, Copy)]
enum Kind {
    Class,
    Coverage,
    Attach,
    Points,
    Caret,
    LigGlyph,
    CaretValue,
    MarkSets,
}
struct Context<'a, 'b, F> {
    bytes: &'a [u8],
    refreshed: &'b mut Option<Vec<u8>>,
    inflated: Option<Vec<u8>>,
    warning: &'b mut F,
}
impl<E, F: FnMut(FontWarning) -> Result<Option<Vec<u8>>, E>> Context<'_, '_, F> {
    fn reader(&self) -> Reader<'_> {
        Reader::new(
            self.inflated
                .as_deref()
                .unwrap_or_else(|| self.refreshed.as_deref().unwrap_or(self.bytes)),
        )
    }
    fn pointer(&mut self, base: usize, at: usize, kind: Kind) -> Result<(), FontLoadError<E>> {
        let offset = usize::from(self.reader().u16(base + at)?);
        if offset != 0 {
            self.read(base + offset, kind)?;
        }
        Ok(())
    }
    fn pointers(&mut self, base: usize, at: usize, kind: Kind) -> Result<(), FontLoadError<E>> {
        let count = usize::from(self.reader().u16(base + at)?);
        // Read each pointer immediately before visiting its child. A child may
        // warn and mutate the next pointer through the original buffer alias.
        for index in 0..count {
            self.pointer(base, at + 2 + index * 2, kind)?;
        }
        Ok(())
    }
    fn read(&mut self, base: usize, kind: Kind) -> Result<(), FontLoadError<E>> {
        match kind {
            Kind::Class => {
                let format = self.reader().u16(base)?;
                match format {
                    1 => {
                        self.reader().u16(base + 2)?;
                        self.words(base, 4, 2)?;
                    }
                    2 => self.words(base, 2, 6)?,
                    _ => {
                        let current = (self.warning)(FontWarning::InvalidClassDefinition {
                            offset: base,
                            format,
                        })
                        .map_err(FontLoadError::Warning)?;
                        if current.is_some() {
                            *self.refreshed = current;
                        }
                    }
                }
            }
            Kind::Coverage => {
                let format = self.reader().u16(base)?;
                let count = usize::from(self.reader().u16(base + 2)?);
                let size = match format {
                    1 => 2,
                    2 => 6,
                    _ => return Err(FontError::CoverageFormat { offset: base }.into()),
                };
                self.reader().bytes(base + 4, count * size)?;
            }
            Kind::Attach | Kind::Caret => {
                self.pointer(base, 0, Kind::Coverage)?;
                self.pointers(
                    base,
                    2,
                    if matches!(kind, Kind::Attach) {
                        Kind::Points
                    } else {
                        Kind::LigGlyph
                    },
                )?;
            }
            Kind::Points => self.words(base, 0, 2)?,
            Kind::LigGlyph => self.pointers(base, 0, Kind::CaretValue)?,
            Kind::CaretValue => {
                if !matches!(self.reader().u16(base)?, 1..=3) {
                    return Err(FontError::Invalid("Unsupported CaretValue table version.").into());
                }
                self.reader().i16(base + 2)?;
            }
            Kind::MarkSets => {
                self.reader().u16(base)?;
                // This source uses 16-bit pointers despite the OpenType spec's
                // Offset32 records. Compatibility follows the actual parser.
                self.pointers(base, 2, Kind::Coverage)?;
            }
        }
        Ok(())
    }
    fn words(&self, base: usize, at: usize, width: usize) -> Result<(), FontError> {
        let count = usize::from(self.reader().u16(base + at)?);
        self.reader().bytes(base + at + 2, count * width)?;
        Ok(())
    }
}
