// Initial CFF INDEX reads follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::super::{
    font_bytes::{FontError, Reader},
    font_deflate,
};
use std::borrow::Cow;

#[path = "font_cff2_dict.rs"]
mod dict;

struct Data<'a> {
    reader: Reader<'a>,
    compressed: Option<(&'a [u8], usize)>,
}
impl<'a> Data<'a> {
    fn check(&self, start: i64, length: usize) -> Result<usize, FontError> {
        let start = usize::try_from(start).map_err(|_| FontError::Bounds)?;
        if let Some((_, limit)) = self.compressed {
            if start.checked_add(length).is_none_or(|end| end > limit) {
                return Err(FontError::Bounds);
            }
        } else {
            self.reader.bytes(start, length)?;
        }
        Ok(start)
    }
    fn bytes(&self, start: i64, length: usize) -> Result<Cow<'a, [u8]>, FontError> {
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
    fn count(&self, start: i64, cff2: bool) -> Result<i64, FontError> {
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
    let mut position = i64::try_from(start).map_err(|_| FontError::Bounds)?;
    let major = data.bytes(position, 1)?[0];
    if major < 2 {
        // CFF1 ignores header.size and always starts its Name INDEX at byte4.
        position += 4;
        for _ in 0..3 {
            position = index(&data, position, false)?;
        }
    } else {
        // CFF2 reads its global subroutines before interpreting the top dict.
        position += i64::from(data.bytes(position + 2, 1)?[0]);
        position += data.count(
            i64::try_from(start).map_err(|_| FontError::Bounds)? + 3,
            false,
        )?;
    }
    index(&data, position, major > 1)?;
    if major > 1 {
        // CFF2 copies its bounded topDictLength bytes only after validating the
        // global INDEX. The dictionary is a separate DataView, so operands may
        // not consume following subroutine or whole-font bytes.
        let start = i64::try_from(start).map_err(|_| FontError::Bounds)?;
        let offset = start + i64::from(data.bytes(start + 2, 1)?[0]);
        let length =
            usize::try_from(data.count(start + 3, false)?).map_err(|_| FontError::Bounds)?;
        dict::validate(&data.bytes(offset, length)?)?;
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
fn index(data: &Data<'_>, start: i64, cff2: bool) -> Result<i64, FontError> {
    let count_length = if cff2 { 4 } else { 2 };
    let count = data.count(start, cff2)?;
    if count == 0 {
        return Ok(start + count_length);
    }
    let width = usize::from(data.bytes(start + count_length, 1)?[0]);
    let object_offset = start + (count + 1) * width as i64 + count_length;
    if width == 0 {
        // Every offset is zero. Source creates count empty arrays without any
        // data reads; neither the empty ranges nor their position is validated.
        return Ok(object_offset);
    }
    let length = usize::try_from((count + 1) * width as i64).map_err(|_| FontError::Bounds)?;
    data.check(start + count_length + 1, length)?;
    let mut previous = None;
    for first in (0..count + 1).step_by(16) {
        let length =
            usize::try_from((count + 1 - first).min(16)).map_err(|_| FontError::Bounds)? * width;
        let bytes = data.bytes(start + count_length + 1 + first * width as i64, length)?;
        for bytes in bytes.chunks_exact(width) {
            let next = offset(bytes);
            if let Some(previous) = previous.filter(|previous| *previous < next) {
                // getBytes does not read reversed or empty ranges. Forward
                // ranges are checked against the whole source DataView.
                data.check(
                    object_offset + previous,
                    usize::try_from(next - previous).map_err(|_| FontError::Bounds)?,
                )?;
            }
            previous = Some(next);
        }
    }
    // count is nonzero and every iteration consumes at least one offset.
    Ok(object_offset + previous.ok_or(FontError::Bounds)?)
}
