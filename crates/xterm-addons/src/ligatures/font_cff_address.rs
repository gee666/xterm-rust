// CFF addresses preserve JS addition before DataView applies ToIndex.
// See licenses/opentype-js-MIT.txt.
use super::{Data, FontError};

#[derive(Debug, PartialEq)]
pub(super) enum IndexOffset {
    Number(f64),
    Array(String),
}
impl IndexOffset {
    pub(super) fn operands(values: &[f64]) -> Result<Self, FontError> {
        if let [value] = values {
            return Ok(Self::Number(*value));
        }
        let mut text = String::new();
        for (i, value) in values.iter().enumerate() {
            let mut buffer = ryu_js::Buffer::new();
            let value = buffer.format(*value);
            text.try_reserve(value.len() + usize::from(i > 0))?;
            if i > 0 {
                text.push(',');
            }
            text.push_str(value);
        }
        Ok(Self::Array(text))
    }
    pub(super) fn truthy(&self) -> bool {
        match self {
            Self::Number(value) => *value != 0.0 && !value.is_nan(),
            Self::Array(_) => true,
        }
    }
    pub(super) fn validate(&self, data: &Data<'_>, start: usize) -> Result<(), FontError> {
        match self {
            Self::Number(value) => super::index(data, start as f64 + value, true).map(|_| ()),
            Self::Array(value) => text_index(data, format!("{start}{value}")),
        }
    }
}
pub(super) fn to_index(value: f64) -> Result<usize, FontError> {
    let integer = if value.is_nan() { 0.0 } else { value.trunc() };
    // DataView ToIndex rejects infinities, negative integers and values beyond
    // MAX_SAFE_INTEGER. The usize bound also keeps this safe on wasm32.
    if !integer.is_finite()
        || !(0.0..=9_007_199_254_740_991.0).contains(&integer)
        || integer > usize::MAX as f64
    {
        return Err(FontError::Bounds);
    }
    Ok(integer as usize)
}
fn number(text: &str) -> f64 {
    text.parse().unwrap_or(f64::NAN)
}
fn text_index(data: &Data<'_>, start: String) -> Result<(), FontError> {
    let count = data.count(number(&start), true)?;
    if count == 0 {
        return Ok(());
    }
    let width = usize::from(data.bytes(number(&format!("{start}4")), 1)?[0]);
    if width == 0 {
        return Ok(());
    }
    if number(&start).is_nan() {
        // A multi-operand array retains commas through every +=. Every read is
        // ToIndex(NaN)=0, all offsets are identical and all object ranges empty.
        return Ok(());
    }
    let object = format!("{start}{}4", (count + 1) * width as i64);
    let mut position = format!("{start}41");
    let mut offsets = Vec::new();
    for _ in 0..=count {
        let mut offset = 0_i32;
        for i in 0..width {
            let byte = data.bytes(number(&format!("{position}{i}")), 1)?[0];
            offset = offset.wrapping_shl(8).wrapping_add(i32::from(byte));
        }
        // Numeric string positions grow by concatenation. A bounded DataView
        // normally rejects them within a few reads, not after count allocations.
        offsets.try_reserve(1)?;
        offsets.push(offset);
        position.try_reserve(3)?;
        position.push_str(&width.to_string());
    }
    // Source reads every INDEX offset before inspecting any object range.
    for pair in offsets.windows(2) {
        text_range(
            data,
            format!("{object}{}", pair[0]),
            format!("{object}{}", pair[1]),
        )?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_shapes_keep_numeric_and_string_addition_distinct() {
        assert_eq!(
            IndexOffset::operands(&[-0.0]).unwrap(),
            IndexOffset::Number(-0.0)
        );
        assert_eq!(
            IndexOffset::operands(&[]).unwrap(),
            IndexOffset::Array(String::new())
        );
        assert_eq!(
            IndexOffset::operands(&[-0.0, f64::NAN]).unwrap(),
            IndexOffset::Array("0,NaN".into())
        );
        let data = Data {
            reader: super::super::Reader::new(&[1, 0, 0, 0]),
            compressed: None,
        };
        // The source attempts millions of identical empty-object allocations.
        // Only finite Rust traversal, not resource-error equivalence, is tested.
        assert_eq!(
            IndexOffset::operands(&[1.0, 2.0])
                .unwrap()
                .validate(&data, 0),
            Ok(())
        );
    }
    #[test]
    fn string_ranges_read_only_forward_lexical_positions_and_reject_nontermination() {
        let data = Data {
            reader: super::super::Reader::new(&[0, 0]),
            compressed: None,
        };
        assert_eq!(text_range(&data, "2".into(), "1".into()), Ok(()));
        assert_eq!(text_range(&data, "2".into(), "2".into()), Ok(()));
        assert_eq!(
            text_range(&data, "0".into(), "2".into()),
            Err(FontError::Bounds)
        );
        assert_eq!(
            text_range(&data, "x".into(), "y".into()),
            Err(FontError::Invalid(
                "CFF2 string INDEX range does not terminate"
            ))
        );
    }
}
fn text_range(data: &Data<'_>, mut first: String, last: String) -> Result<(), FontError> {
    while first < last {
        let value = number(&first);
        if value.is_nan() {
            // Source appends "1" forever while repeatedly reading byte0 here.
            // Do not hang on an untrusted malformed font.
            return Err(FontError::Invalid(
                "CFF2 string INDEX range does not terminate",
            ));
        }
        data.bytes(value, 1)?;
        first.try_reserve(1)?;
        first.push('1');
    }
    Ok(())
}
