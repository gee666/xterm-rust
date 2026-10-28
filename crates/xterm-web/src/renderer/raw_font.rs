// Copyright (c) 2023 The xterm.js authors. All rights reserved. MIT license.
//! Raw font keys stay uncoerced until the source's individual canvas setter.
use super::width_cache::{Font, FontVariant};
use xterm_core::options::{OptionError, OptionValue, OptionsService};

#[derive(Clone, Debug, PartialEq)]
pub struct RawFont {
    pub family: OptionValue,
    pub size: OptionValue,
    pub weight: OptionValue,
    pub weight_bold: OptionValue,
}
impl Default for RawFont {
    fn default() -> Self {
        Self {
            family: "".into(),
            size: 0.0.into(),
            weight: "normal".into(),
            weight_bold: "bold".into(),
        }
    }
}
impl RawFont {
    /// All four argument reads precede setFont and any per-variant conversion.
    pub fn read_with(
        mut read: impl FnMut(&str) -> Result<OptionValue, OptionError>,
    ) -> Result<Self, OptionError> {
        Ok(Self {
            family: read("fontFamily")?,
            size: read("fontSize")?,
            weight: read("fontWeight")?,
            weight_bold: read("fontWeightBold")?,
        })
    }
    pub fn from_options(options: &OptionsService) -> Result<Self, OptionError> {
        Self::read_with(|key| options.get(key))
    }
}

pub trait FontDescriptor: Default + PartialEq {
    fn variant_units(&self, variant: FontVariant) -> Result<Vec<u16>, OptionError>;
}
impl FontDescriptor for RawFont {
    fn variant_units(&self, variant: FontVariant) -> Result<Vec<u16>, OptionError> {
        let mut units = Vec::new();
        if variant.is_italic() {
            units.extend("italic".encode_utf16());
        }
        units.push(u16::from(b' '));
        let weight = if variant.is_bold() {
            &self.weight_bold
        } else {
            &self.weight
        };
        units.extend(weight.to_utf16()?);
        units.push(u16::from(b' '));
        units.extend(self.size.to_utf16()?);
        units.extend("px ".encode_utf16());
        units.extend(self.family.to_utf16()?);
        Ok(trim_ecma(&units).to_vec())
    }
}
impl FontDescriptor for Font {
    fn variant_units(&self, variant: FontVariant) -> Result<Vec<u16>, OptionError> {
        RawFont {
            family: self.family.as_str().into(),
            size: self.size.into(),
            weight: self.weight.as_str().into(),
            weight_bold: self.weight_bold.as_str().into(),
        }
        .variant_units(variant)
    }
}

/// TextMetricsMeasureStrategy converts size before reading family. Unlike the
/// width-cache canvas setter, this source string has no final trim operation.
pub fn text_metrics_font(
    mut read: impl FnMut(&str) -> Result<OptionValue, OptionError>,
) -> Result<Vec<u16>, OptionError> {
    let mut units = font_size(&read("fontSize")?)?;
    units.push(u16::from(b' '));
    units.extend(read("fontFamily")?.to_utf16()?);
    Ok(units)
}
pub fn font_size(value: &OptionValue) -> Result<Vec<u16>, OptionError> {
    let mut units = value.to_utf16()?;
    units.extend("px".encode_utf16());
    Ok(units)
}

// ECMAScript WhiteSpace/LineTerminator data differs from Rust trim at FEFF/NEL.
fn whitespace(unit: u16) -> bool {
    matches!(unit,0x9..=0xd|0x20|0xa0|0x1680|0x2000..=0x200a|0x2028|0x2029|0x202f|0x205f|0x3000|0xfeff)
}
fn trim_ecma(units: &[u16]) -> &[u16] {
    let start = units
        .iter()
        .position(|unit| !whitespace(*unit))
        .unwrap_or(units.len());
    let end = units
        .iter()
        .rposition(|unit| !whitespace(*unit))
        .map_or(start, |index| index + 1);
    &units[start..end]
}
#[cfg(test)]
#[path = "raw_font_tests.rs"]
mod tests;
