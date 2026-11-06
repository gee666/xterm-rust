// CFF INDEX reads follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::super::{
    font_bytes::{FontError, Reader},
    font_deflate,
};
use std::borrow::Cow;

#[path = "font_cff_address.rs"]
mod address;
#[path = "font_cff2_dict.rs"]
mod dict;
#[cfg(test)]
#[path = "font_cff_index_tests.rs"]
mod tests;

struct Data<'a> {
    reader: Reader<'a>,
    compressed: Option<(&'a [u8], usize)>,
}
impl<'a> Data<'a> {
    fn check(&self, start: f64, length: usize) -> Result<usize, FontError> {
        let start = address::to_index(start)?;
        if let Some((_, limit)) = self.compressed {
            if start.checked_add(length).is_none_or(|end| end > limit) {
                return Err(FontError::Bounds);
            }
        } else {
            self.reader.bytes(start, length)?;
        }
        Ok(start)
    }
    fn bytes(&self, start: f64, length: usize) -> Result<Cow<'a, [u8]>, FontError> {
        let start = self.check(start, length)?;
        if let Ok(bytes) = self.reader.bytes(start, length) {
            return Ok(Cow::Borrowed(bytes));
        }
        // Callers read at most16 INDEX offsets at once, never an untrusted whole
        // table. Replay the validated stream only for reads beyond the prefix.
        let (input, limit) = self.compressed.ok_or(FontError::Bounds)?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(length)?;
        bytes.resize(length, 0);
        font_deflate::range(input, limit, start, &mut bytes)?;
        Ok(Cow::Owned(bytes))
    }
    fn count(&self, start: f64, cff2: bool) -> Result<i64, FontError> {
        let bytes = self.bytes(start, if cff2 { 4 } else { 2 })?;
        Ok(bytes
            .iter()
            .fold(0_i64, |value, byte| (value << 8) + i64::from(*byte)))
    }
}
pub(super) fn validate(reader: Reader<'_>, start: usize) -> Result<(), FontError> {
    validate_data(
        Data {
            reader,
            compressed: None,
        },
        start,
    )
}
pub(super) fn compressed(input: &[u8], limit: usize, prefix: &[u8]) -> Result<(), FontError> {
    validate_data(
        Data {
            reader: Reader::new(prefix),
            compressed: Some((input, limit)),
        },
        0,
    )
}
fn validate_data(data: Data<'_>, start: usize) -> Result<(), FontError> {
    // The directory offset is u32, so it is exactly representable as a JS number.
    let base = start as f64;
    let mut position = base;
    let major = data.bytes(position, 1)?[0];
    if major < 2 {
        // CFF1 ignores header.size and always starts its Name INDEX at byte4.
        position += 4.0;
        for _ in 0..3 {
            position = index(&data, position, false)?;
        }
    } else {
        // CFF2 reads its global subroutines before interpreting the top dict.
        position += f64::from(data.bytes(position + 2.0, 1)?[0]);
        position += data.count(base + 3.0, false)? as f64;
    }
    index(&data, position, major > 1)?;
    if major > 1 {
        // The top dictionary has its own DataView. Operands may not consume
        // following subroutine or whole-font bytes.
        let offset = base + f64::from(data.bytes(base + 2.0, 1)?[0]);
        let length =
            usize::try_from(data.count(base + 3.0, false)?).map_err(|_| FontError::Bounds)?;
        let fd_array = dict::validate(&data.bytes(offset, length)?)?;
        fd_array.validate(&data, start)?;
    }
    Ok(())
}
fn offset(bytes: &[u8]) -> i64 {
    // getOffset uses a signed JavaScript << before adding each byte, even for
    // offSize values outside the format's usual1..4 range. Do not reject them.
    i64::from(bytes.iter().fold(0_i32, |value, byte| {
        value.wrapping_shl(8).wrapping_add(i32::from(*byte))
    }))
}
fn index(data: &Data<'_>, start: f64, cff2: bool) -> Result<f64, FontError> {
    let count_length = if cff2 { 4.0 } else { 2.0 };
    let count = data.count(start, cff2)?;
    if count == 0 {
        return Ok(start + count_length);
    }
    let width = usize::from(data.bytes(start + count_length, 1)?[0]);
    let object_offset = start + ((count + 1) * width as i64) as f64 + count_length;
    if width == 0 {
        // Every offset is zero. Source creates count empty arrays without any
        // data reads; neither the empty ranges nor their position is validated.
        return Ok(object_offset);
    }
    let length = usize::try_from((count + 1) * width as i64).map_err(|_| FontError::Bounds)?;
    data.check(start + count_length + 1.0, length)?;
    let mut previous = None;
    for first in (0..count + 1).step_by(16) {
        let length =
            usize::try_from((count + 1 - first).min(16)).map_err(|_| FontError::Bounds)? * width;
        let bytes = data.bytes(
            start + count_length + 1.0 + (first * width as i64) as f64,
            length,
        )?;
        for bytes in bytes.chunks_exact(width) {
            let next = offset(bytes);
            if let Some(previous) = previous.filter(|previous| *previous < next) {
                // Empty/reversed ranges perform no reads. Fractional starts
                // survive arithmetic until DataView truncates each byte index.
                let first = object_offset + previous as f64;
                data.check(first, 1)?;
                data.check(first + (next - previous - 1) as f64, 1)?;
            }
            previous = Some(next);
        }
    }
    // count is nonzero and every iteration consumes at least one offset.
    Ok(object_offset + previous.ok_or(FontError::Bounds)? as f64)
}
