// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Constructor read phases follow opentype.js, licenses/opentype-js-MIT.txt.
use super::{font_bytes::Reader, FontError, FontWarning};

pub(super) fn warning(tag: &[u8], bytes: &[u8]) -> Result<Option<FontWarning>, FontError> {
    Ok(
        (Reader::new(bytes).u16(0)? != 0).then_some(if tag == b"CPAL" {
            FontWarning::UnsupportedCpalVersion
        } else {
            FontWarning::UnsupportedColrVersion
        }),
    )
}

pub(super) fn finish(tag: &[u8], bytes: &[u8]) -> Result<(), FontError> {
    let reader = Reader::new(bytes);
    if tag == b"CPAL" {
        reader.i16(2)?;
        let palettes = reader.i16(4)?;
        let colors = reader.i16(6)?;
        let offset = reader.offset32(8)?;
        // CPAL uses signed counts despite its nominal OpenType unsigned fields.
        // Array creation precedes element reads, and palette reads precede the
        // color array's creation. Preserve that error order without allocating.
        let palettes = usize::try_from(palettes).map_err(|_| FontError::ArrayLength)?;
        extent(reader, 12, palettes, 2)?;
        let colors = usize::try_from(colors).map_err(|_| FontError::ArrayLength)?;
        extent(reader, offset, colors, 4)
    } else {
        let bases = usize::from(reader.u16(2)?);
        let base_offset = reader.offset32(4)?;
        let layer_offset = reader.offset32(8)?;
        let layers = usize::from(reader.u16(12)?);
        extent(reader, base_offset, bases, 6)?;
        extent(reader, layer_offset, layers, 4)
    }
}

fn extent(reader: Reader<'_>, offset: usize, count: usize, width: usize) -> Result<(), FontError> {
    // A zero-length source loop never dereferences its offset, even beyond EOF.
    if count != 0 {
        reader.bytes(offset, count * width)?;
    }
    Ok(())
}
