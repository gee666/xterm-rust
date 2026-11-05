// CFF header contracts follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::{
    font_bytes::{FontError, Reader},
    font_deflate,
};

/// Only the constructor header phase is validated here. CFF index, dictionary
/// and charstring interpretation remain separate, unfinished port work.
pub(super) fn validate(
    reader: Reader<'_>,
    record: Reader<'_>,
    woff: bool,
) -> Result<(), FontError> {
    let offset = record.offset32(if woff { 4 } else { 8 })?;
    if woff && record.offset32(8)? < record.offset32(12)? {
        let input = offset.checked_add(2).ok_or(FontError::Bounds)?;
        let length = record
            .offset32(8)?
            .checked_sub(2)
            .ok_or(FontError::Bounds)?;
        // Source allocates a zeroed table, ignores output overflow and validates
        // the whole compressed stream before inspecting its first header byte.
        // Keep only the five possible header bytes; do not allocate a font-sized
        // buffer for metadata whose remaining interpretation is not ported yet.
        let mut prefix = [0; 5];
        let size = record.offset32(12)?.min(prefix.len());
        font_deflate::prefix(reader.bytes(input, length)?, &mut prefix[..size])?;
        header(Reader::new(&prefix[..size]))
    } else {
        header(reader.at(offset)?)
    }
}
fn header(reader: Reader<'_>) -> Result<(), FontError> {
    let major = reader.bytes(0, 1)?[0];
    let minor = reader.bytes(1, 1)?[0];
    if major > 2 {
        return Err(FontError::CffVersion { major, minor });
    }
    // Size is read even though the CFF1 parser uses a fixed four-byte header.
    // Unsupported versions throw before this read, not after a full-header check.
    reader.bytes(2, 1)?;
    if major < 2 {
        reader.bytes(3, 1)?;
    } else {
        reader.u16(3)?;
    }
    Ok(())
}
