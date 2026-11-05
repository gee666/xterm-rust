// Copyright (c) The xterm.js authors. All rights reserved. MIT license.
// Constructor ordering follows opentype.js; see licenses/opentype-js-MIT.txt.
use super::font_bytes::{table_data, FontError, Reader};

pub(super) fn validate(
    reader: Reader<'_>,
    glyf: Reader<'_>,
    loca: Reader<'_>,
    woff: bool,
    glyph_count: Option<u16>,
    short_loca: bool,
) -> Result<(), FontError> {
    // Missing maxp leaves numGlyphs undefined, so even the sentinel is unread.
    // Zero glyphs, however, still reads one sentinel offset. Offset values do
    // not trigger glyph body reads: opentype's glyph paths remain lazy here.
    let required = glyph_count.map_or(0, |count| {
        (usize::from(count) + 1) * if short_loca { 2 } else { 4 }
    });
    read_constructor_table(reader, loca, woff, required)?;
    read_constructor_table(reader, glyf, woff, 0)
}

pub(super) fn validate_metrics(
    reader: Reader<'_>,
    record: Reader<'_>,
    woff: bool,
    glyph_count: u16,
    metric_count: u16,
) -> Result<(), FontError> {
    const HORIZONTAL_METRIC_SIZE: usize = 4;
    let required = usize::from(glyph_count.min(metric_count)) * HORIZONTAL_METRIC_SIZE;
    read_constructor_table(reader, record, woff, required)
}

fn read_constructor_table(
    reader: Reader<'_>,
    record: Reader<'_>,
    woff: bool,
    required: usize,
) -> Result<(), FontError> {
    if woff && record.offset32(8)? < record.offset32(12)? {
        let bytes = table_data(reader, record, woff, true)?;
        Reader::new(&bytes).bytes(0, required)?;
    } else if required != 0 {
        // The original uncompressed DataView ignores nominal table lengths.
        // Without reads its offset may be outside the input without throwing.
        reader.bytes(record.offset32(if woff { 4 } else { 8 })?, required)?;
    }
    Ok(())
}
