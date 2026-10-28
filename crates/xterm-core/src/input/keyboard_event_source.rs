// Licensed under MIT. Fallible live-field access shared by keyboard encoders.

use std::borrow::Cow;
use std::convert::Infallible;

use super::keyboard::KeyboardEvent;

/// Read every field at its source expression. Implementations may observe a
/// mutable browser event or throw; encoders must not eagerly snapshot or cache
/// these calls across source reads. Strings retain JavaScript UTF-16 units.
///
/// Numeric key codes remain f64 so synthetic events can retain fractional,
/// negative, infinite and NaN values instead of an early WebIDL conversion.
pub trait KeyboardEventSource {
    type Error;

    fn key_code(&self) -> Result<f64, Self::Error>;
    fn key(&self) -> Result<Cow<'_, [u16]>, Self::Error>;
    fn code(&self) -> Result<Cow<'_, [u16]>, Self::Error>;
    fn shift_key(&self) -> Result<bool, Self::Error>;
    fn alt_key(&self) -> Result<bool, Self::Error>;
    fn ctrl_key(&self) -> Result<bool, Self::Error>;
    fn meta_key(&self) -> Result<bool, Self::Error>;
}

impl KeyboardEventSource for KeyboardEvent<'_> {
    type Error = Infallible;

    fn key_code(&self) -> Result<f64, Infallible> {
        Ok(f64::from(self.key_code))
    }
    fn key(&self) -> Result<Cow<'_, [u16]>, Infallible> {
        Ok(Cow::Borrowed(self.key))
    }
    fn code(&self) -> Result<Cow<'_, [u16]>, Infallible> {
        Ok(Cow::Borrowed(self.code))
    }
    fn shift_key(&self) -> Result<bool, Infallible> {
        Ok(self.shift_key)
    }
    fn alt_key(&self) -> Result<bool, Infallible> {
        Ok(self.alt_key)
    }
    fn ctrl_key(&self) -> Result<bool, Infallible> {
        Ok(self.ctrl_key)
    }
    fn meta_key(&self) -> Result<bool, Infallible> {
        Ok(self.meta_key)
    }
}

/// JavaScript numeric truthiness, including both signed zeroes and NaN.
pub fn number_is_truthy(number: f64) -> bool {
    number != 0.0 && !number.is_nan()
}

/// String.fromCharCode uses ToUint16 rather than Unicode scalar validation.
pub fn to_uint16(number: f64) -> u16 {
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    number.trunc().rem_euclid(65536.0) as u16
}

pub(crate) fn code_point_at_zero(units: &[u16]) -> Option<u32> {
    let first = *units.first()?;
    if (0xd800..=0xdbff).contains(&first) {
        if let Some(second) = units
            .get(1)
            .filter(|unit| (0xdc00..=0xdfff).contains(*unit))
        {
            return Some(
                ((u32::from(first) - 0xd800) << 10) + u32::from(*second) - 0xdc00 + 0x10000,
            );
        }
    }
    Some(u32::from(first))
}
