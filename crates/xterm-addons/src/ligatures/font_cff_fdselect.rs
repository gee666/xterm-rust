// Eager CFF2 FDSelect reads follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::super::super::font_error::FdSelectError;
use super::{address::IndexOffset, Data, FontError};

pub(super) fn validate(
    data: &Data<'_>,
    base: usize,
    offset: &IndexOffset,
    glyphs: Option<u16>,
    fonts: u32,
) -> Result<(), FontError> {
    let mut relative = 0;
    let mut read = |width| {
        let value = offset.read(data, base, relative, width)?;
        relative = relative.checked_add(width).ok_or(FontError::Bounds)?;
        Ok::<_, FontError>(value)
    };
    let format = u8::try_from(read(1)?).map_err(|_| FontError::Bounds)?;
    let index = |value| {
        if value >= fonts {
            Err(FontError::CffFdSelect(FdSelectError::Index {
                index: value,
                count: fonts,
            }))
        } else {
            Ok(())
        }
    };
    if format == 0 {
        // An absent maxp leaves numGlyphs undefined. The source loop reads no
        // entries, not an inferred count from the later charstrings INDEX.
        for _ in 0..glyphs.unwrap_or(0) {
            index(read(1)?)?;
        }
        return Ok(());
    }
    if !matches!(format, 3 | 4) {
        return Err(FontError::CffFdSelect(FdSelectError::Format(format)));
    }
    let width = if format == 4 { 4 } else { 2 };
    let ranges = read(width)?;
    let first = read(width)?;
    if first != 0 {
        return Err(FontError::CffFdSelect(FdSelectError::Initial {
            format,
            glyph: first,
        }));
    }
    let mut next = None;
    for _ in 0..ranges {
        let fd = read(if format == 4 { 2 } else { 1 })?;
        let end = read(width)?;
        // Both values are read before either is checked. In particular a
        // truncated endpoint wins over an already-invalid FD index.
        index(fd)?;
        if glyphs.is_some_and(|count| end > u32::from(count)) {
            return Err(FontError::CffFdSelect(FdSelectError::Range {
                version: 2,
                glyph: end,
            }));
        }
        // The ligature constructor never uses the expanded glyph-to-FD array.
        // Do not allocate repeated entries or reject descending/equal ranges.
        next = Some(end);
    }
    if next != glyphs.map(u32::from) {
        return Err(FontError::CffFdSelect(FdSelectError::Sentinel(next)));
    }
    Ok(())
}
