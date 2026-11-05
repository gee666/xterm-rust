// SVG constructor reads follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::font_bytes::{FontError, Reader};
use std::collections::HashSet;

/// Validate construction without retaining SVG documents unused by ligatures.
/// The source creates document views even for an empty/reversed glyph range.
pub fn validate(bytes: &[u8]) -> Result<(), FontError> {
    let table = Reader::new(bytes);
    if table.u16(0)? != 0 {
        return Ok(());
    }
    let list_offset = table.offset32(2)?;
    let count = usize::from(table.u16(list_offset)?);
    let mut documents = HashSet::new();
    for index in 0..count {
        let record = list_offset
            .checked_add(2 + index * 12)
            .ok_or(FontError::Bounds)?;
        table.u16(record)?;
        table.u16(record + 2)?;
        let document = list_offset
            .checked_add(table.offset32(record + 4)?)
            .ok_or(FontError::Bounds)?;
        let length = table.offset32(record + 8)?;
        // The source caches by absolute offset alone. A repeated offset still
        // reads its length field but does not construct or validate a new view.
        if !documents.contains(&document) {
            table.bytes(document, length)?;
            documents.try_reserve(1)?;
            documents.insert(document);
        }
    }
    Ok(())
}
