// Constructor reads follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::super::font_deflate as deflate;
use super::{FontError, Reader};

// Hint values never affect ligature lookup, so validate constructor reads
// without allocating a discarded list proportional to an untrusted length.
pub(super) fn validate(
    reader: Reader<'_>,
    record: Reader<'_>,
    woff: bool,
    words: bool,
) -> Result<(), FontError> {
    let length = record.offset32(12)?;
    let compressed = woff && record.offset32(8)? < length;
    if compressed {
        let offset = record
            .offset32(4)?
            .checked_add(2)
            .ok_or(FontError::Bounds)?;
        let size = record
            .offset32(8)?
            .checked_sub(2)
            .ok_or(FontError::Bounds)?;
        // Source skips the zlib header and ignores its checksum and output
        // length. Its fixed typed array zero-fills short output and discards
        // overflow. Only decompressor failure can reject the discarded hints.
        deflate::validate(reader.bytes(offset, size)?)?;
    }
    // parseShortList allocates Array(length / 2) after decompression. Odd
    // directory lengths throw before any DataView reads, even at bad offsets.
    if words && length % 2 != 0 {
        return Err(FontError::ArrayLength);
    }
    if !compressed && length != 0 {
        reader.bytes(record.offset32(if woff { 4 } else { 8 })?, length)?;
    }
    // Zero-element uncompressed lists never dereference their table offset.
    Ok(())
}
