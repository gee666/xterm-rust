// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.

use super::{CoercionError, CoercionHint, NativeProperties, OptionObject};
use std::any::Any;
use std::rc::Rc;

/// Values accepted at the JS options boundary. UTF-16 strings retain lone
/// surrogates. Numbers retain NaN, infinities, and signed zero. Native objects
/// use reference identity; opaque host values use their host's strict equality.
#[derive(Clone, Debug)]
pub enum OptionValue {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(Vec<u16>),
    Object(Rc<OptionObject>),
}

impl PartialEq for OptionValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Undefined, Self::Undefined) | (Self::Null, Self::Null) => true,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Object(a), Self::Object(b)) => {
                a.has_custom_equality() == b.has_custom_equality()
                    && a.equals(b).unwrap_or_else(|| Rc::ptr_eq(a, b))
            }
            _ => false,
        }
    }
}

impl From<&str> for OptionValue {
    fn from(value: &str) -> Self {
        Self::String(value.encode_utf16().collect())
    }
}
impl From<f64> for OptionValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}
impl From<bool> for OptionValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl OptionValue {
    pub fn object<T: Any>(value: T) -> Self {
        Self::Object(OptionObject::plain(value))
    }

    /// A structured options bag. Missing properties return JS undefined. For
    /// dynamic objects, use OptionObject::with_properties instead.
    pub fn property_map<K: Into<String>>(entries: impl IntoIterator<Item = (K, Self)>) -> Self {
        let properties = NativeProperties::new(entries);
        Self::Object(OptionObject::with_property_access(
            properties.clone(),
            |_| Ok(Self::from("[object Object]")),
            properties,
        ))
    }

    pub fn empty_object() -> Self {
        Self::property_map(std::iter::empty::<(&str, Self)>())
    }

    pub fn set_property(&self, key: &str, value: Self) -> Result<(), super::OptionError> {
        match self {
            Self::Object(object) => object.set_property(key, value),
            _ => Err(CoercionError::new("property assignment requires an object").into()),
        }
    }
    pub fn delete_property(&self, key: &str) -> Result<bool, super::OptionError> {
        match self {
            Self::Object(object) => object.delete_property(key),
            _ => Err(CoercionError::new("property deletion requires an object").into()),
        }
    }

    /// Read nested options. JS boxes primitive values; strings additionally
    /// expose UTF-16 length/index properties. Host object lookup, including
    /// getters and prototypes, is supplied by OptionObject::with_properties.
    pub fn get_property(&self, key: &str) -> Result<Self, super::OptionError> {
        match self {
            Self::Object(value) => value.get_property(key),
            Self::Null | Self::Undefined => {
                let base = if matches!(self, Self::Null) {
                    "null"
                } else {
                    "undefined"
                };
                Err(CoercionError::new(&format!(
                    "Cannot read properties of {base} (reading '{key}')"
                ))
                .into())
            }
            Self::String(units) if key == "length" => Ok(Self::Number(units.len() as f64)),
            Self::String(units) => Ok(key
                .parse::<usize>()
                .ok()
                .filter(|index| index.to_string() == key)
                .and_then(|index| units.get(index))
                .map(|unit| Self::String(vec![*unit]))
                .unwrap_or(Self::Undefined)),
            _ => Ok(Self::Undefined),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Undefined | Self::Null => false,
            Self::Bool(value) => *value,
            Self::Number(value) => *value != 0.0 && !value.is_nan(),
            Self::String(value) => !value.is_empty(),
            Self::Object(value) => value.truthy(),
        }
    }

    /// JS addition preserves numeric addition versus UTF-16 concatenation and
    /// invokes object default-hint conversion at each call, not once per option.
    pub fn js_add(&self, rhs: &Self) -> Result<Self, super::OptionError> {
        if let Self::Object(left) = self {
            if let Some(result) = left.add_left(rhs) {
                return result;
            }
        }
        if let Self::Object(right) = rhs {
            if let Some(result) = right.add_right(self) {
                return result;
            }
        }
        let primitive = |value: &Self| match value {
            Self::Object(value) => value.primitive(CoercionHint::Number),
            _ => Ok(value.clone()),
        };
        let left = primitive(self)?;
        let right = primitive(rhs)?;
        if matches!(left, Self::String(_)) || matches!(right, Self::String(_)) {
            let mut value = left.text_utf16()?;
            value.extend(right.text_utf16()?);
            Ok(Self::String(value))
        } else {
            Ok(Self::Number(left.number()? + right.number()?))
        }
    }

    pub(crate) fn less_than(&self, rhs: f64) -> Result<bool, super::OptionError> {
        match self {
            Self::Object(value) => value.less_than(rhs),
            _ => Ok(self.number()? < rhs),
        }
    }
    pub(crate) fn less_equal(&self, rhs: f64) -> Result<bool, super::OptionError> {
        match self {
            Self::Object(value) => value.less_equal(rhs),
            _ => Ok(self.number()? <= rhs),
        }
    }
    pub(crate) fn multiply(&self, rhs: f64) -> Result<f64, super::OptionError> {
        match self {
            Self::Object(value) => value.multiply(rhs),
            _ => Ok(self.number()? * rhs),
        }
    }

    /// ECMAScript ToNumber at a host boundary, including fallible Number-hint
    /// object conversion. Strings use JS whitespace and numeric grammar.
    pub fn to_number(&self) -> Result<f64, super::OptionError> {
        self.number()
    }

    /// String concatenation uses ToPrimitive with the default hint. This is
    /// distinct from explicit String-hint conversion for Date/custom objects.
    pub fn concat_utf16(&self) -> Result<Vec<u16>, super::OptionError> {
        match self {
            Self::Object(value) => value.concat_utf16(),
            _ => self.text_utf16(),
        }
    }

    /// ECMAScript string conversion with String hint and lossless UTF-16.
    pub fn to_utf16(&self) -> Result<Vec<u16>, super::OptionError> {
        self.text_utf16()
    }

    pub(crate) fn number(&self) -> Result<f64, super::OptionError> {
        Ok(match self {
            Self::Undefined => f64::NAN,
            Self::Null => 0.0,
            Self::Bool(value) => u8::from(*value).into(),
            Self::Number(value) => *value,
            Self::String(value) => parse_number(value),
            Self::Object(value) => value.primitive(CoercionHint::Number)?.number()?,
        })
    }

    pub(crate) fn text_utf16(&self) -> Result<Vec<u16>, super::OptionError> {
        let text = match self {
            Self::Undefined => "undefined".into(),
            Self::Null => "null".into(),
            Self::Bool(value) => value.to_string(),
            Self::Number(value) => ryu_js::Buffer::new().format(*value).to_owned(),
            Self::String(value) => return Ok(value.clone()),
            Self::Object(value) => return value.primitive(CoercionHint::String)?.text_utf16(),
        };
        Ok(text.encode_utf16().collect())
    }
}

fn parse_number(units: &[u16]) -> f64 {
    let Ok(text) = String::from_utf16(units) else {
        return f64::NAN;
    };
    // ECMAScript whitespace includes BOM, but excludes NEL, which Rust includes.
    let text = text.trim_matches(|c: char| {
        matches!(c,
        '\u{9}'..='\u{d}' | ' ' | '\u{a0}' | '\u{1680}' |
        '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' |
        '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
    });
    if text.is_empty() {
        return 0.0;
    }
    match text {
        "Infinity" | "+Infinity" => return f64::INFINITY,
        "-Infinity" => return f64::NEG_INFINITY,
        _ => {}
    }
    for (prefixes, radix) in [(["0x", "0X"], 16), (["0o", "0O"], 8), (["0b", "0B"], 2)] {
        if let Some(digits) = prefixes.iter().find_map(|prefix| text.strip_prefix(prefix)) {
            if digits.is_empty() {
                return f64::NAN;
            }
            return radix_number(digits, radix);
        }
    }
    // Rust accepts inf/NaN spellings outside the JS decimal grammar.
    if text
        .chars()
        .any(|c| !matches!(c, '0'..='9' | '+' | '-' | '.' | 'e' | 'E'))
    {
        return f64::NAN;
    }
    text.parse().unwrap_or(f64::NAN)
}

// Repeated floating-point multiply/add double-rounds long radix strings. Keep
// the first 53 significant bits and guard/sticky bits, then round once, ties-even.
fn radix_number(digits: &str, radix: u32) -> f64 {
    let bits_per_digit = radix.ilog2();
    let mut count = 0_u32;
    let mut mantissa = 0_u64;
    let mut guard = false;
    let mut sticky = false;
    for digit in digits.chars() {
        let Some(digit) = digit.to_digit(radix) else {
            return f64::NAN;
        };
        for shift in (0..bits_per_digit).rev() {
            let bit = digit & (1 << shift) != 0;
            if count == 0 && !bit {
                continue;
            }
            match count {
                0..=52 => mantissa = (mantissa << 1) | u64::from(bit),
                53 => guard = bit,
                _ => sticky |= bit,
            }
            count = count.saturating_add(1);
        }
    }
    if count > 1024 {
        return f64::INFINITY;
    }
    if count <= 53 {
        return mantissa as f64;
    }
    if guard && (sticky || mantissa & 1 != 0) {
        mantissa += 1;
    }
    // count is in 54..=1024, so this conversion cannot truncate.
    let exponent = i32::try_from(count - 53).expect("bounded exponent");
    mantissa as f64 * 2_f64.powi(exponent)
}
