// Constructor reads follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::font_bytes::{FontError, Reader};

const KERN_PAIR_SIZE: usize = 6;

// Ligature shaping consumes GSUB, not kerning pairs. Still validate every byte
// the source constructor reads. Its skip operations intentionally do not check
// bounds, table lengths and later subtables do not constrain the first one.
pub fn validate(bytes: &[u8]) -> Result<(), FontError> {
    let table = Reader::new(bytes);
    let (count_offset, pairs_offset) = match table.u16(0)? {
        0 => {
            if table.u16(4)? != 0 {
                return Err(FontError::Invalid("Unsupported kern sub-table version."));
            }
            (10, 18)
        }
        1 => {
            table.u32(4)?;
            if table.u16(12)? & 0xff != 0 {
                return Ok(());
            }
            (16, 24)
        }
        version => return Err(FontError::KernVersion(version)),
    };
    let pairs = usize::from(table.u16(count_offset)?);
    if pairs != 0 {
        table.bytes(pairs_offset, pairs * KERN_PAIR_SIZE)?;
    }
    Ok(())
}
