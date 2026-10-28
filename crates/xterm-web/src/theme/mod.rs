// Copyright (c) 2022 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from browser/services/ThemeService.ts.
mod color_set;
mod css_text;
mod live;
pub use css_text::CssText;
pub use xterm_core::color::palette;
mod option;
mod resolve;
pub use option::ThemeOptionError;

use color_set::RestoreColors;
pub use color_set::{ColorRef, ColorSet, ColorSlot, SharedColors};
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    color::{css, Color},
    event::{EmitError, Emitter, Event},
    lifecycle::Disposable,
};

/// The browser binding maps the sixteen named ANSI properties to `ansi` in
/// black/red/green/yellow/blue/magenta/cyan/white then bright order.
#[derive(Clone, Debug, Default)]
pub struct Theme {
    pub foreground: Option<CssText>,
    pub background: Option<CssText>,
    pub cursor: Option<CssText>,
    pub cursor_accent: Option<CssText>,
    pub selection_foreground: Option<CssText>,
    pub selection_background: Option<CssText>,
    pub selection_inactive_background: Option<CssText>,
    pub scrollbar_slider_background: Option<CssText>,
    pub scrollbar_slider_hover_background: Option<CssText>,
    pub scrollbar_slider_active_background: Option<CssText>,
    pub overview_ruler_border: Option<CssText>,
    pub ansi: [Option<CssText>; 16],
    pub extended_ansi: Vec<Option<CssText>>,
}

pub type ColorParser = Rc<dyn Fn(&str) -> Result<Color, css::UnsupportedCss>>;
pub type Utf16ColorParser = Rc<dyn Fn(&[u16]) -> Result<ColorRef, css::UnsupportedCss>>;

/// Color state independent of DOM lifetime. The host must forward typed theme
/// option changes and minimum-contrast changes through the corresponding methods.
/// Events carry the same live color-set identity, with no borrow across callbacks.
pub struct ThemeService {
    colors: SharedColors,
    restore: RefCell<RestoreColors>,
    defaults: resolve::Defaults,
    parser: Utf16ColorParser,
    changes: Emitter<SharedColors>,
}
impl ThemeService {
    pub fn new(theme: &Theme, parser: ColorParser) -> Self {
        Self::new_utf16(
            theme,
            Rc::new(move |units| match String::from_utf16(units) {
                Ok(text) => parser(&text).map(ColorRef::from),
                Err(_) => css::to_color_utf16(units).map(ColorRef::from),
            }),
        )
    }
    pub fn new_utf16(theme: &Theme, parser: Utf16ColorParser) -> Self {
        let defaults = resolve::Defaults::new();
        let colors = resolve::colors(theme, &defaults, parser.as_ref(), None);
        let restore = RestoreColors::capture(&colors);
        Self {
            colors: Rc::new(RefCell::new(colors)),
            restore: RefCell::new(restore),
            defaults,
            parser,
            changes: Emitter::new(),
        }
    }
    pub fn parse_color(&self, input: &str) -> Result<Color, css::UnsupportedCss> {
        let parsed = (self.parser)(&input.encode_utf16().collect::<Vec<_>>())?;
        let color = parsed.borrow().clone();
        Ok(color)
    }
    pub fn parse_css_units(&self, input: &[u16]) -> Result<ColorRef, css::UnsupportedCss> {
        (self.parser)(input)
    }
    pub fn colors(&self) -> SharedColors {
        Rc::clone(&self.colors)
    }
    pub fn on_change_colors(&self) -> Event<SharedColors> {
        self.changes.event()
    }
    pub fn set_theme(&self, theme: &Theme) -> Result<(), EmitError> {
        let caches = {
            let colors = self.colors.borrow();
            (
                Rc::clone(&colors.contrast_cache),
                Rc::clone(&colors.half_contrast_cache),
            )
        };
        let colors = resolve::colors(theme, &self.defaults, self.parser.as_ref(), Some(caches));
        colors.contrast_cache.borrow_mut().clear();
        colors.half_contrast_cache.borrow_mut().clear();
        *self.restore.borrow_mut() = RestoreColors::capture(&colors);
        *self.colors.borrow_mut() = colors;
        self.changes.fire(&self.colors)
    }
    /// Upstream clears only the full-contrast cache for this option event.
    pub fn minimum_contrast_ratio_changed(&self) {
        self.colors.borrow().contrast_cache.borrow_mut().clear();
    }
    pub fn restore_color(&self, slot: Option<ColorSlot>) -> Result<(), EmitError> {
        self.restore
            .borrow()
            .restore(&mut self.colors.borrow_mut(), slot);
        self.changes.fire(&self.colors)
    }
    /// The callback receives the live set, not a snapshot. Release any mutable
    /// borrow before reentering the service, as required by Rust borrowing rules.
    pub fn modify_colors(&self, callback: impl FnOnce(SharedColors)) -> Result<(), EmitError> {
        callback(self.colors());
        self.changes.fire(&self.colors)
    }
}
impl Default for ThemeService {
    fn default() -> Self {
        Self::new(&Theme::default(), Rc::new(css::to_color))
    }
}
impl Disposable for ThemeService {
    fn dispose(&self) {
        self.changes.dispose();
    }
}
impl Drop for ThemeService {
    fn drop(&mut self) {
        self.dispose();
    }
}
