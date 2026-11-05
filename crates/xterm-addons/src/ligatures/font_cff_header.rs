// CFF header contracts follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::font_bytes::{FontError, Reader};

/// Only the constructor header phase is validated here. CFF index, dictionary
/// and charstring interpretation remain separate, unfinished port work.
pub(super) fn validate(bytes: &[u8]) -> Result<(), FontError> {
    let reader = Reader::new(bytes);
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
