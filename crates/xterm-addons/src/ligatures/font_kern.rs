// Constructor reads follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::{font_bytes::Reader, FontError, FontWarning};

const KERN_PAIR_SIZE: usize = 6;

pub enum Header {
    Windows,
    Mac { multiple_subtables: bool },
}
impl Header {
    pub fn read(bytes: &[u8]) -> Result<Self, FontError> {
        let table = Reader::new(bytes);
        match table.u16(0)? {
            0 => Ok(Self::Windows),
            1 => Ok(Self::Mac {
                multiple_subtables: table.u32(4)? > 1,
            }),
            version => Err(FontError::KernVersion(version)),
        }
    }

    pub fn warning(&self) -> Option<FontWarning> {
        match self {
            Self::Mac {
                multiple_subtables: true,
            } => Some(FontWarning::MultipleKernSubtables),
            _ => None,
        }
    }

    // Keep the parsed header across the callback. The source resumes at coverage
    // even if the callback overwrites the version or subtable count in memory.
    pub fn finish(self, bytes: &[u8]) -> Result<(), FontError> {
        let table = Reader::new(bytes);
        let (count_offset, pairs_offset) = match self {
            Self::Windows => {
                if table.u16(4)? != 0 {
                    return Err(FontError::Invalid("Unsupported kern sub-table version."));
                }
                (10, 18)
            }
            Self::Mac { .. } => {
                if table.u16(12)? & 0xff != 0 {
                    return Ok(());
                }
                (16, 24)
            }
        };
        let pairs = usize::from(table.u16(count_offset)?);
        if pairs != 0 {
            table.bytes(pairs_offset, pairs * KERN_PAIR_SIZE)?;
        }
        Ok(())
    }
}
