//! Live option application deliberately commits each field before the next getter.
use super::{resolve, ColorRef, RestoreColors, Theme, ThemeOptionError, ThemeService};
use xterm_core::options::OptionValue;
impl ThemeService {
    pub fn set_theme_value(&self, value: &OptionValue) -> Result<(), ThemeOptionError> {
        if let OptionValue::Object(object) = value {
            if let Some(theme) = object.payload::<Theme>() {
                return self.set_theme(theme).map_err(Into::into);
            }
        }
        if matches!(value, OptionValue::Undefined) {
            return self.set_theme(&Theme::default()).map_err(Into::into);
        }
        let foreground = self.read_color(value, "foreground", &self.defaults.foreground)?;
        self.colors.borrow_mut().foreground = foreground;
        let background = self.read_color(value, "background", &self.defaults.background)?;
        self.colors.borrow_mut().background = background;
        let background = self.colors.borrow().background.clone();
        let cursor = self.read_color(value, "cursor", &self.defaults.cursor)?;
        self.colors.borrow_mut().cursor = resolve::blend(&background, &cursor);
        let background = self.colors.borrow().background.clone();
        let accent = self.read_color(value, "cursorAccent", &self.defaults.background)?;
        self.colors.borrow_mut().cursor_accent = resolve::blend(&background, &accent);
        let selection = self.read_color(value, "selectionBackground", &self.defaults.selection)?;
        self.colors.borrow_mut().selection_background_transparent = selection;
        let (background, selection) = {
            let colors = self.colors.borrow();
            (
                colors.background.clone(),
                colors.selection_background_transparent.clone(),
            )
        };
        self.colors.borrow_mut().selection_background_opaque =
            resolve::blend(&background, &selection);
        let inactive_value = value.get_property("selectionInactiveBackground")?;
        let fallback = self
            .colors
            .borrow()
            .selection_background_transparent
            .clone();
        self.colors
            .borrow_mut()
            .selection_inactive_background_transparent =
            self.parse_value(&inactive_value, &fallback);
        let (background, selection) = {
            let colors = self.colors.borrow();
            (
                colors.background.clone(),
                colors.selection_inactive_background_transparent.clone(),
            )
        };
        self.colors
            .borrow_mut()
            .selection_inactive_background_opaque = resolve::blend(&background, &selection);
        let first = value.get_property("selectionForeground")?;
        let selection_foreground = if first.is_truthy() {
            match value.get_property("selectionForeground")? {
                OptionValue::String(units) => (self.parser)(&units).ok(),
                _ => None,
            }
        } else {
            None
        };
        self.colors.borrow_mut().selection_foreground = selection_foreground;
        let selection = self
            .colors
            .borrow()
            .selection_background_transparent
            .clone();
        self.colors.borrow_mut().selection_background_transparent =
            resolve::selection_transparent(selection);
        let selection = self
            .colors
            .borrow()
            .selection_inactive_background_transparent
            .clone();
        self.colors
            .borrow_mut()
            .selection_inactive_background_transparent = resolve::selection_transparent(selection);
        for (key, factor) in [
            ("scrollbarSliderBackground", 0.2),
            ("scrollbarSliderHoverBackground", 0.4),
            ("scrollbarSliderActiveBackground", 0.5),
        ] {
            let input = value.get_property(key)?;
            let foreground = self.colors.borrow().foreground.clone();
            let color = self.parse_value(&input, &resolve::opacity(&foreground, factor));
            let mut colors = self.colors.borrow_mut();
            match key {
                "scrollbarSliderBackground" => colors.scrollbar_slider_background = color,
                "scrollbarSliderHoverBackground" => {
                    colors.scrollbar_slider_hover_background = color
                }
                _ => colors.scrollbar_slider_active_background = color,
            }
        }
        let border = self.read_color(value, "overviewRulerBorder", &self.defaults.foreground)?;
        self.colors.borrow_mut().overview_ruler_border = border;
        self.colors.borrow_mut().ansi = self.defaults.ansi.clone();
        for (index, name) in super::option::ANSI_NAMES.iter().enumerate() {
            let color = self.read_color(value, name, &self.defaults.ansi[index])?;
            self.assign_ansi(index, color)?;
        }
        if value.get_property("extendedAnsi")?.is_truthy() {
            let length = value
                .get_property("extendedAnsi")?
                .get_property("length")?
                .to_number()?;
            let available = self.colors.borrow().ansi.len().saturating_sub(16) as f64;
            let count = if length.is_nan() || length <= 0.0 {
                0
            } else {
                length.min(available).ceil() as usize
            };
            for index in 0..count {
                let input = value
                    .get_property("extendedAnsi")?
                    .get_property(&index.to_string())?;
                let fallback = self
                    .defaults
                    .ansi
                    .get(index + 16)
                    .ok_or(ThemeOptionError::InvalidAnsiState)?;
                let color = self.parse_value(&input, fallback);
                self.assign_ansi(index + 16, color)?;
            }
        }
        {
            let colors = self.colors.borrow();
            colors.contrast_cache.borrow_mut().clear();
            colors.half_contrast_cache.borrow_mut().clear();
            *self.restore.borrow_mut() = RestoreColors::capture(&colors);
        }
        self.changes.fire(&self.colors)?;
        Ok(())
    }
    fn assign_ansi(&self, index: usize, color: ColorRef) -> Result<(), ThemeOptionError> {
        let mut colors = self.colors.borrow_mut();
        let slot = colors
            .ansi
            .get_mut(index)
            .ok_or(ThemeOptionError::InvalidAnsiState)?;
        *slot = color;
        Ok(())
    }
    fn read_color(
        &self,
        value: &OptionValue,
        key: &str,
        fallback: &ColorRef,
    ) -> Result<ColorRef, ThemeOptionError> {
        Ok(self.parse_value(&value.get_property(key)?, fallback))
    }
    fn parse_value(&self, value: &OptionValue, fallback: &ColorRef) -> ColorRef {
        match value {
            OptionValue::String(units) => {
                resolve::parse(Some(units), fallback, self.parser.as_ref())
            }
            _ => fallback.clone(),
        }
    }
}
