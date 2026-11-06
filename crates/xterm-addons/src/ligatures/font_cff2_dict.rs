// CFF dictionary reads follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::{address::IndexOffset, FontError};

const ESCAPED_OPERATOR: u16 = 1200;
const KEY_COUNT: usize = ESCAPED_OPERATOR as usize + 256;
const FD_ARRAY: u16 = 1236;
const MISSING_FD_ARRAY: &str = "This is a CFF2 font, but FDArray information is missing";

pub(super) fn validate(bytes: &[u8]) -> Result<IndexOffset, FontError> {
    dictionary(bytes)?
        .filter(IndexOffset::truthy)
        .ok_or(FontError::Invalid(MISSING_FD_ARRAY))
}

/// Operand reads finish before entriesToObject can report a duplicate key.
/// Return FDArray operands only after these earlier dictionary phases succeed.
fn dictionary(bytes: &[u8]) -> Result<Option<IndexOffset>, FontError> {
    let mut position = 0;
    let mut operands = Vec::new();
    // Each operand needs at least one byte. This bound also covers blend reuse.
    operands.try_reserve_exact(bytes.len())?;
    let mut keys = [None; KEY_COUNT];
    let mut duplicate = None;
    let mut negative_blend = false;
    let mut fd_array = None;
    while position < bytes.len() {
        let byte = read(bytes, &mut position)?;
        match byte {
            23 => {
                let count = operands.pop().unwrap_or(f64::NAN);
                // Source loops forever once a negative count empties the stack.
                // Reject that input instead of reproducing an unbounded loop.
                if count < 0.0 {
                    negative_blend = true;
                    operands.clear();
                } else {
                    while operands.len() as f64 > count {
                        operands.pop();
                    }
                }
            }
            0..=27 => {
                let key = if byte == 12 {
                    ESCAPED_OPERATOR + u16::from(read(bytes, &mut position)?)
                } else {
                    u16::from(byte)
                };
                let previous = &mut keys[usize::from(key)];
                if *previous == Some(true) && duplicate.is_none() {
                    duplicate = Some(key);
                }
                // isNaN([]) is false, isNaN([n]) follows n, and arrays with
                // multiple numeric elements stringify with commas, yielding NaN.
                *previous = Some(match operands.as_slice() {
                    [] => true,
                    [value] => !value.is_nan(),
                    _ => false,
                });
                if key == FD_ARRAY {
                    // entriesToObject unwraps one operand to a number, but keeps
                    // zero or multiple operands as arrays. Even [] is truthy.
                    fd_array = Some(IndexOffset::operands(&operands)?);
                }
                operands.clear();
            }
            28 => {
                let bytes = take(bytes, &mut position, 2)?;
                operands.push(f64::from(u16::from_be_bytes([bytes[0], bytes[1]])));
            }
            29 => {
                let bytes = take(bytes, &mut position, 4)?;
                operands.push(f64::from(i32::from_be_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                ])));
            }
            30 => operands.push(real(bytes, &mut position)?),
            32..=246 => operands.push(f64::from(byte) - 139.0),
            247..=254 => {
                let next = f64::from(read(bytes, &mut position)?);
                let value = if byte <= 250 {
                    f64::from(byte - 247) * 256.0 + next + 108.0
                } else {
                    -f64::from(byte - 251) * 256.0 - next - 108.0
                };
                operands.push(value);
            }
            31 | 255 => return Err(FontError::CffOperand(byte)),
        }
    }
    if negative_blend {
        return Err(FontError::Invalid("CFF2 blend count cannot be negative"));
    }
    duplicate.map_or(Ok(fd_array), |key| Err(FontError::CffDuplicate(key)))
}
fn read(bytes: &[u8], position: &mut usize) -> Result<u8, FontError> {
    let byte = *bytes.get(*position).ok_or(FontError::Bounds)?;
    *position += 1;
    Ok(byte)
}
fn take<'a>(bytes: &'a [u8], position: &mut usize, length: usize) -> Result<&'a [u8], FontError> {
    let value = bytes
        .get(*position..*position + length)
        .ok_or(FontError::Bounds)?;
    *position += length;
    Ok(value)
}
fn real(bytes: &[u8], position: &mut usize) -> Result<f64, FontError> {
    const NIBBLES: [&str; 15] = [
        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ".", "E", "E-", "null", "-",
    ];
    let mut text = String::new();
    loop {
        let byte = read(bytes, position)?;
        for nibble in [byte >> 4, byte & 15] {
            if nibble == 15 {
                return float_prefix(&text);
            }
            let part = NIBBLES[usize::from(nibble)];
            text.try_reserve(part.len())?;
            text.push_str(part);
        }
    }
}
fn float_prefix(text: &str) -> Result<f64, FontError> {
    // JS parseFloat accepts a numeric prefix, including incomplete exponents.
    // The CFF nibble alphabet contains no whitespace, plus sign or Infinity.
    let bytes = text.as_bytes();
    let mut end = usize::from(bytes.first() == Some(&b'-'));
    let mut digits = 0;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        digits += 1;
        end += 1;
    }
    if bytes.get(end) == Some(&b'.') {
        end += 1;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            digits += 1;
            end += 1;
        }
    }
    if digits == 0 {
        return Ok(f64::NAN);
    }
    let mut exponent = end;
    if bytes.get(end) == Some(&b'E') {
        exponent += 1;
        if bytes.get(exponent) == Some(&b'-') {
            exponent += 1;
        }
        let start = exponent;
        while bytes.get(exponent).is_some_and(u8::is_ascii_digit) {
            exponent += 1;
        }
        if exponent > start {
            end = exponent;
        }
    }
    text[..end]
        .parse()
        .map_err(|_| FontError::Invalid("Invalid CFF real numeric prefix"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // These existing assertions test the lexical/entries phase. The constructor
    // presence guard is a later phase, covered separately below.
    fn validate(bytes: &[u8]) -> Result<(), FontError> {
        dictionary(bytes).map(|_| ())
    }

    #[test]
    fn fd_array_presence_uses_js_truthiness_after_dictionary_errors() {
        for bytes in [&[][..], &[139, 12, 36], &[30, 0xff, 12, 36]] {
            assert_eq!(
                super::validate(bytes).map(|_| ()),
                Err(FontError::Invalid(MISSING_FD_ARRAY))
            );
        }
        for bytes in [&[12, 36][..], &[139, 140, 12, 36], &[138, 12, 36]] {
            assert_eq!(super::validate(bytes).map(|_| ()), Ok(()));
        }
        assert_eq!(
            super::validate(&[139, 12, 36, 0, 0]).map(|_| ()),
            Err(FontError::CffDuplicate(0))
        );
        assert_eq!(
            super::validate(&[139, 12, 36, 0, 0, 31]).map(|_| ()),
            Err(FontError::CffOperand(31))
        );
        // A NaN previous value permits a replacement, including replacement by0.
        assert_eq!(
            super::validate(&[30, 0xff, 12, 36, 140, 12, 36]).map(|_| ()),
            Ok(())
        );
        assert_eq!(
            super::validate(&[30, 0xff, 12, 36, 139, 12, 36]).map(|_| ()),
            Err(FontError::Invalid(MISSING_FD_ARRAY))
        );
    }

    #[test]
    fn all_single_byte_operands_and_operators() {
        for byte in 0..=255 {
            let expected = match byte {
                12 | 28..=30 | 247..=254 => Err(FontError::Bounds),
                31 | 255 => Err(FontError::CffOperand(byte)),
                _ => Ok(()),
            };
            assert_eq!(validate(&[byte]), expected, "byte {byte}");
        }
    }
    #[test]
    fn escaped_operator_and_real_terminators_are_not_operands() {
        for byte in 0..=255 {
            assert_eq!(validate(&[12, byte]), Ok(()));
        }
        for byte in [0x0f, 0xf0, 0xff, 0xdf, 0xfd] {
            assert_eq!(validate(&[30, byte]), Ok(()));
        }
        assert_eq!(
            validate(&[30, 0x12, 0x3f, 255]),
            Err(FontError::CffOperand(255))
        );
        assert_eq!(validate(&[28, 31, 255, 29, 31, 255, 31, 255]), Ok(()));
    }
    #[test]
    fn duplicate_coercion_and_deferred_lexical_errors() {
        assert_eq!(validate(&[0, 0]), Err(FontError::CffDuplicate(0)));
        assert_eq!(validate(&[139, 140, 0, 0]), Ok(()));
        assert_eq!(validate(&[30, 0xff, 0, 0]), Ok(()));
        assert_eq!(validate(&[0, 0, 31]), Err(FontError::CffOperand(31)));
        assert_eq!(validate(&[0, 0, 28]), Err(FontError::Bounds));
        assert_eq!(
            validate(&[139, 140, 140, 23, 0, 0]),
            Err(FontError::CffDuplicate(0))
        );
        assert_eq!(
            validate(&[138, 23]),
            Err(FontError::Invalid("CFF2 blend count cannot be negative"))
        );
    }
    #[test]
    fn real_numeric_prefixes() {
        for text in ["", "null", ".", "-", "E1", "-null", "..1"] {
            assert!(float_prefix(text).unwrap().is_nan(), "{text}");
        }
        for (text, value) in [
            ("1null", 1.0),
            ("1E", 1.0),
            ("1E-", 1.0),
            (".1E2", 10.0),
            ("-1E-2-", -0.01),
        ] {
            assert_eq!(float_prefix(text), Ok(value));
        }
        assert_eq!(float_prefix("1E9999"), Ok(f64::INFINITY));
    }
}
