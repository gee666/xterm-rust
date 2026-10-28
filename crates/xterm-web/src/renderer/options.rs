use super::width_cache::Font;
use xterm_core::options::{OptionError, OptionsService};
#[derive(Clone, Debug)]
pub(super) struct RenderOptions {
    pub font: Font,
    pub letter_spacing: f64,
    pub line_height: f64,
    pub cursor_width: f64,
    pub cursor_style: String,
    pub cursor_inactive_style: String,
    pub cursor_blink: bool,
    pub bold_bright: bool,
    pub minimum_contrast_ratio: f64,
    pub blink_interval: f64,
}
pub fn read_font(options: &OptionsService) -> Result<Font, OptionError> {
    let text = |key| -> Result<String, OptionError> {
        Ok(String::from_utf16_lossy(&options.get(key)?.to_utf16()?))
    };
    Ok(Font {
        family: text("fontFamily")?,
        size: options.get("fontSize")?.to_number()?,
        weight: text("fontWeight")?,
        weight_bold: text("fontWeightBold")?,
    })
}
impl RenderOptions {
    pub fn read(options: &OptionsService) -> Result<Self, OptionError> {
        let text = |key| -> Result<String, OptionError> {
            Ok(String::from_utf16_lossy(&options.get(key)?.to_utf16()?))
        };
        let number = |key| options.get(key)?.to_number();
        Ok(Self {
            font: read_font(options)?,
            letter_spacing: number("letterSpacing")?,
            line_height: number("lineHeight")?,
            cursor_width: number("cursorWidth")?,
            cursor_style: text("cursorStyle")?,
            cursor_inactive_style: text("cursorInactiveStyle")?,
            cursor_blink: options.get("cursorBlink")?.is_truthy(),
            bold_bright: options.get("drawBoldTextInBrightColors")?.is_truthy(),
            minimum_contrast_ratio: number("minimumContrastRatio")?,
            blink_interval: number("blinkIntervalDuration")?,
        })
    }
}
