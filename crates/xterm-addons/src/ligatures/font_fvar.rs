// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Read order follows opentype.js; see licenses/opentype-js-MIT.txt.
use super::font_bytes::{FontError, Reader};

const FVAR_VERSION: u32 = 0x0001_0000;

pub fn validate(bytes: &[u8]) -> Result<u16, FontError> {
    let reader = Reader::new(bytes);
    if reader.u32(0)? != FVAR_VERSION {
        return Err(FontError::Invalid("Unsupported fvar table version."));
    }
    let offset = usize::from(reader.u16(4)?);
    // countSizePairs and flags are skipped, not read or validated by the source.
    let axes = reader.u16(8)?;
    let axis_count = usize::from(axes);
    let axis_size = usize::from(reader.u16(10)?);
    let instance_count = usize::from(reader.u16(12)?);
    let instance_size = usize::from(reader.u16(14)?);
    for index in 0..axis_count {
        let at = offset + index * axis_size;
        reader.bytes(at, 16)?;
        reader.u16(at + 18)?;
    }
    let instance_start = offset + axis_count * axis_size;
    for index in 0..instance_count {
        let at = instance_start
            .checked_add(index * instance_size)
            .ok_or(FontError::Bounds)?;
        reader.u16(at)?;
        let coordinate_bytes = axis_count * 4;
        // With no axes, skipped flags need not exist. A zero-length read at
        // at+4 would reject inputs whose only actual read is the name ID.
        if coordinate_bytes != 0 {
            let coordinates = at.checked_add(4).ok_or(FontError::Bounds)?;
            reader.bytes(coordinates, coordinate_bytes)?;
        }
        if instance_size != 4 + coordinate_bytes {
            let postscript = at
                .checked_add(4 + coordinate_bytes)
                .ok_or(FontError::Bounds)?;
            reader.u16(postscript)?;
        }
    }
    Ok(axes)
}
