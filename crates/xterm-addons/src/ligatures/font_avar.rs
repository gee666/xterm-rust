// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Constructor read order follows opentype.js; see licenses/opentype-js-MIT.txt.
use super::{
    font_bytes::{FontError, Reader},
    FontWarning,
};

pub struct Header {
    major: u16,
    minor: u16,
}
impl Header {
    pub fn read(bytes: &[u8]) -> Result<Self, FontError> {
        let reader = Reader::new(bytes);
        Ok(Self {
            major: reader.u16(0)?,
            minor: reader.u16(2)?,
        })
    }

    pub fn warning(&self) -> Option<FontWarning> {
        (self.major != 1).then_some(FontWarning::AvarVersion {
            major: self.major,
            minor: self.minor,
        })
    }

    pub fn finish(&self, bytes: &[u8], axes: Option<u16>) -> Result<(), FontError> {
        let reader = Reader::new(bytes);
        // Reserved bytes are skipped by the source, not validated.
        let count = reader.u16(6)?;
        let axes = axes.ok_or(FontError::Type(
            "Cannot read properties of undefined (reading 'axes')",
        ))?;
        if count != axes {
            return Err(FontError::Invalid(
                "avar axis count must correspond to fvar axis count",
            ));
        }
        let mut at = 8;
        for _ in 0..count {
            let positions = usize::from(reader.u16(at)?);
            at += 2;
            // Coordinates are F2Dot14 reads with no ordering/range validation.
            let size = positions * 4;
            reader.bytes(at, size)?;
            at += size;
        }
        Ok(())
    }
}
