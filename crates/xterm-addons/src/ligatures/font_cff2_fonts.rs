// Eager CFF2 font/private DICT reads follow opentype.js.
// See licenses/opentype-js-MIT.txt.
use super::{dict, Data, FontError};

pub(super) fn validate(data: &Data<'_>, base: f64, bytes: &[u8]) -> Result<(), FontError> {
    let font = dict::parse(
        &mut |at| bytes.get(at).copied().ok_or(FontError::Bounds),
        bytes.len() as f64,
        dict::Kind::Font,
    )?;
    let Some((size, offset)) = font.private else {
        return Ok(());
    };
    if size == 0.0 || offset == 0.0 {
        return Ok(());
    }
    let start = offset + base;
    if size == f64::INFINITY && start.is_nan() {
        // Every address becomes ToIndex(NaN)=0 forever. Preserve immediate
        // operand errors, but do not reproduce a nonterminating source loop.
        let byte = data.bytes(start, 1)?[0];
        return Err(if matches!(byte, 31 | 255) {
            FontError::CffOperand(byte)
        } else {
            FontError::Invalid("CFF2 private dictionary does not terminate")
        });
    }
    let private = dict::parse(
        &mut |at| Ok(data.bytes(start + at as f64, 1)?[0]),
        size,
        dict::Kind::Private,
    )?;
    if let Some(subrs) = private.subrs.filter(super::address::IndexOffset::truthy) {
        // Source adds the relative private offset, then the Subrs operand, then
        // the table base. Array operands make the last two additions strings.
        subrs.subrs(data, offset, base)?;
    }
    Ok(())
}
