// CFF dictionary operand reads follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::super::font_bytes::FontError;

/// The source completes all operand reads before entriesToObject interprets
/// values or detects duplicate keys. Keep that lexical phase separate so an
/// earlier duplicate cannot mask a later truncated or invalid operand.
pub(super) fn validate(bytes: &[u8]) -> Result<(), FontError> {
    let mut position = 0;
    while position < bytes.len() {
        let byte = read(bytes, &mut position)?;
        match byte {
            12 => {
                read(bytes, &mut position)?;
            }
            0..=11 | 13..=27 | 32..=246 => {}
            28 => {
                take(bytes, &mut position, 2)?;
            }
            29 => {
                take(bytes, &mut position, 4)?;
            }
            30 => loop {
                let byte = read(bytes, &mut position)?;
                if byte >> 4 == 15 || byte & 15 == 15 {
                    break;
                }
            },
            247..=254 => {
                read(bytes, &mut position)?;
            }
            31 | 255 => return Err(FontError::CffOperand(byte)),
        }
    }
    Ok(())
}
fn read(bytes: &[u8], position: &mut usize) -> Result<u8, FontError> {
    let byte = *bytes.get(*position).ok_or(FontError::Bounds)?;
    *position += 1;
    Ok(byte)
}
fn take(bytes: &[u8], position: &mut usize, length: usize) -> Result<(), FontError> {
    // Operand sizes are protocol constants and position never exceeds len.
    bytes
        .get(*position..*position + length)
        .ok_or(FontError::Bounds)?;
    *position += length;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
