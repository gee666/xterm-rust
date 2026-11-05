// Licensed under MIT. Fallible live-field access shared by keyboard encoders.

use std::borrow::Cow;
use std::convert::Infallible;

use super::keyboard::KeyboardEvent;
use super::win32_input_mode::lookup::{native_lookup, Table, Value};

#[path = "keyboard_map_inherited.rs"]
pub(crate) mod inherited;

#[path = "keyboard_lookup.rs"]
pub mod lookup;

/// Read every field at its source expression. Implementations may observe a
/// mutable browser event or throw; encoders must not eagerly snapshot or cache
/// these calls across source reads. Strings retain JavaScript UTF-16 units.
///
/// Numeric key codes remain f64 so synthetic events can retain fractional,
/// negative, infinite and NaN values instead of an early WebIDL conversion.
pub trait KeyboardEventSource {
    type Error;

    /// Ordinary Win32 object lookup, retaining raw values until interpolation.
    /// A browser implementation must keep each table alive across events.
    fn win32_lookup(&self, table: Table, key: &[u16]) -> Result<Value<Self::Error>, Self::Error> {
        Ok(native_lookup(table, key))
    }

    /// Ordinary Kitty lookup. Only the runtime knows its prototype and receiver.
    fn kitty_lookup(
        &self,
        table: super::kitty_keyboard::lookup::Table,
        key: &[u16],
    ) -> Result<Value<Self::Error>, Self::Error> {
        Ok(super::kitty_keyboard::lookup::native_lookup(table, key))
    }

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

#[cfg(test)]
mod tests {
    use super::{code_point_at_zero, number_is_truthy};

    #[test]
    fn shared_keyboard_helpers_retain_js_values() {
        // Retained from the source-path oracle runner when it began linking
        // the real core crate. Keep private-helper checks inside their crate.
        assert!(!number_is_truthy(0.0));
        assert_eq!(code_point_at_zero(&[0x41]), Some(0x41));
        assert!(!number_is_truthy(-0.0));
        assert!(!number_is_truthy(f64::NAN));
        assert!(number_is_truthy(f64::INFINITY));
        assert_eq!(code_point_at_zero(&[]), None);
        assert_eq!(code_point_at_zero(&[0xd800]), Some(0xd800));
        assert_eq!(code_point_at_zero(&[0xdc00]), Some(0xdc00));
        assert_eq!(code_point_at_zero(&[0xd83d, 0xde00]), Some(0x1f600));
    }
}
