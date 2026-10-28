use super::Theme;
use std::fmt;
use xterm_core::options::{OptionError, OptionValue};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeOptionError(pub OptionError);
impl fmt::Display for ThemeOptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for ThemeOptionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}
impl From<OptionError> for ThemeOptionError {
    fn from(error: OptionError) -> Self {
        Self(error)
    }
}
impl Theme {
    /// Reads live host properties in source order, preserving getter failures.
    /// The binding retains the original option object; this is its parsed view.
    pub fn from_option_value(value: &OptionValue) -> Result<Self, ThemeOptionError> {
        if let OptionValue::Object(object) = value {
            if let Some(theme) = object.payload::<Theme>() {
                return Ok(theme.clone());
            }
        }
        if matches!(value, OptionValue::Undefined) {
            return Ok(Self::default());
        }
        let mut theme = Self {
            foreground: css(value, "foreground")?,
            background: css(value, "background")?,
            cursor: css(value, "cursor")?,
            cursor_accent: css(value, "cursorAccent")?,
            selection_background: css(value, "selectionBackground")?,
            selection_inactive_background: css(value, "selectionInactiveBackground")?,
            selection_foreground: css(value, "selectionForeground")?,
            scrollbar_slider_background: css(value, "scrollbarSliderBackground")?,
            scrollbar_slider_hover_background: css(value, "scrollbarSliderHoverBackground")?,
            scrollbar_slider_active_background: css(value, "scrollbarSliderActiveBackground")?,
            overview_ruler_border: css(value, "overviewRulerBorder")?,
            ..Self::default()
        };
        const NAMES: [&str; 16] = [
            "black",
            "red",
            "green",
            "yellow",
            "blue",
            "magenta",
            "cyan",
            "white",
            "brightBlack",
            "brightRed",
            "brightGreen",
            "brightYellow",
            "brightBlue",
            "brightMagenta",
            "brightCyan",
            "brightWhite",
        ];
        for (index, name) in NAMES.iter().enumerate() {
            theme.ansi[index] = css(value, name)?;
        }
        let extended = value.get_property("extendedAnsi")?;
        if extended.is_truthy() {
            let length = extended.get_property("length")?.to_number()?;
            // Source loops with i<count, so a fractional array-like length rounds
            // up, whereas NaN or a negative length executes no iterations.
            let count = if length.is_nan() || length <= 0.0 {
                0
            } else {
                length.min(240.0).ceil() as usize
            };
            for index in 0..count {
                theme
                    .extended_ansi
                    .push(css(&extended, &index.to_string())?);
            }
        }
        Ok(theme)
    }
}
fn css(value: &OptionValue, key: &str) -> Result<Option<String>, OptionError> {
    Ok(match value.get_property(key)? {
        OptionValue::String(units) => Some(String::from_utf16_lossy(&units)),
        _ => None,
    })
}
