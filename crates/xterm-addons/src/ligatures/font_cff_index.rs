// Initial CFF INDEX reads follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::super::font_bytes::{FontError, Reader};

pub(super) fn validate(reader: Reader<'_>, start: usize) -> Result<(), FontError> {
    let major = reader.bytes(start, 1)?[0];
    let mut position = i64::try_from(start).map_err(|_| FontError::Bounds)?;
    if major < 2 {
        // CFF1 ignores header.size and always starts its Name INDEX at byte4.
        position += 4;
        for _ in 0..3 {
            position = index(reader, position, false)?;
        }
    } else {
        // CFF2 reads its global subroutines before interpreting the top dict.
        position += i64::from(reader.bytes(start + 2, 1)?[0]);
        position += i64::from(reader.u16(start + 3)?);
    }
    index(reader, position, major > 1)?;
    Ok(())
}

fn at(value: i64) -> Result<usize, FontError> {
    usize::try_from(value).map_err(|_| FontError::Bounds)
}

fn offset(bytes: &[u8]) -> i64 {
    // getOffset uses a signed JavaScript << before adding each byte, even for
    // offSize values outside the format's usual1..4 range. Do not reject them.
    let value = bytes.iter().fold(0_i32, |value, byte| {
        value.wrapping_shl(8).wrapping_add(i32::from(*byte))
    });
    i64::from(value)
}

fn index(reader: Reader<'_>, start: i64, cff2: bool) -> Result<i64, FontError> {
    let count_length = if cff2 { 4 } else { 2 };
    let count = if cff2 {
        i64::from(reader.u32(at(start)?)?)
    } else {
        i64::from(reader.u16(at(start)?)?)
    };
    if count == 0 {
        return Ok(start + count_length);
    }
    let width = usize::from(reader.bytes(at(start + count_length)?, 1)?[0]);
    let object_offset = start + (count + 1) * width as i64 + count_length;
    if width == 0 {
        // Every offset is zero. Source creates count empty arrays without any
        // data reads; neither the empty ranges nor their position is validated.
        return Ok(object_offset);
    }
    let length = usize::try_from((count + 1) * width as i64).map_err(|_| FontError::Bounds)?;
    let offsets = reader.bytes(at(start + count_length + 1)?, length)?;
    let mut previous = offset(&offsets[..width]);
    for bytes in offsets[width..].chunks_exact(width) {
        let next = offset(bytes);
        if previous < next {
            // getBytes does not read reversed or empty ranges, but every byte
            // in a forward range is bounds checked against the full DataView.
            reader.bytes(at(object_offset + previous)?, at(next - previous)?)?;
        }
        previous = next;
    }
    Ok(object_offset + previous)
}
