//! CSS text at a JS boundary is UTF-16, not necessarily a Unicode scalar string.
use std::{fmt, ops::Deref};
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CssText(Vec<u16>);
impl CssText {
    pub fn units(&self) -> &[u16] {
        &self.0
    }
    pub fn into_units(self) -> Vec<u16> {
        self.0
    }
    pub fn to_well_formed(&self) -> Result<String, std::string::FromUtf16Error> {
        String::from_utf16(&self.0)
    }
}
impl From<Vec<u16>> for CssText {
    fn from(value: Vec<u16>) -> Self {
        Self(value)
    }
}
impl From<&str> for CssText {
    fn from(value: &str) -> Self {
        Self(value.encode_utf16().collect())
    }
}
impl From<String> for CssText {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}
impl Deref for CssText {
    type Target = [u16];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
/// Display is an inspection view only. Browser setters use units() directly.
/// Invalid code units are escaped visibly instead of being replaced or dropped.
impl fmt::Display for CssText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use fmt::Write;
        for unit in char::decode_utf16(self.0.iter().copied()) {
            match unit {
                Ok(character) => f.write_char(character)?,
                Err(error) => write!(f, "\\u{:04x}", error.unpaired_surrogate())?,
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn units_survive_nul_markers_and_lone_surrogates() {
        let units = vec![0, 0x1f, 0xd800, 97, 0xdc00, 0xd83d, 0xde00];
        let text = CssText::from(units.clone());
        assert_eq!(text.units(), units);
        assert!(text.to_well_formed().is_err());
        assert_eq!(text.to_string(), "\0\u{1f}\\ud800a\\udc00😀");
        assert_eq!(text.into_units(), units);
    }
}
