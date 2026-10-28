// Copyright (c) 2022 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from browser/Types.ts and ThemeService.ts.
use crate::contrast_cache::ColorContrastCache;
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};
use xterm_core::color::Color;

/// Shared color-object identity, including mutations retained by restore slots.
#[derive(Debug)]
struct SharedColor {
    value: Color,
    raw_css: Option<(String, Vec<u16>)>,
}
#[derive(Clone, Debug)]
pub struct ColorRef(Rc<RefCell<SharedColor>>);
impl From<Color> for ColorRef {
    fn from(color: Color) -> Self {
        Self(Rc::new(RefCell::new(SharedColor {
            value: color,
            raw_css: None,
        })))
    }
}
impl From<xterm_core::color::css::Utf16Color> for ColorRef {
    fn from(color: xterm_core::color::css::Utf16Color) -> Self {
        let result = Self::from(Color {
            css: String::new(),
            rgba: color.rgba,
        });
        result.set_css_units(color.css);
        result
    }
}
impl ColorRef {
    pub fn borrow(&self) -> Ref<'_, Color> {
        Ref::map(self.0.borrow(), |color| &color.value)
    }
    pub fn borrow_mut(&self) -> RefMut<'_, Color> {
        RefMut::map(self.0.borrow_mut(), |color| &mut color.value)
    }
    /// Actual CSS code units. For an ill-formed JS string, borrow().css is only
    /// an escaped native inspection view; browser setters must use this method.
    pub fn css_units(&self) -> Vec<u16> {
        let color = self.0.borrow();
        if let Some((projection, units)) = &color.raw_css {
            if *projection == color.value.css {
                return units.clone();
            }
        }
        color.value.css.encode_utf16().collect()
    }
    pub fn set_css_units(&self, units: Vec<u16>) {
        let mut color = self.0.borrow_mut();
        match String::from_utf16(&units) {
            Ok(text) => {
                color.value.css = text;
                color.raw_css = None;
            }
            Err(_) => {
                use std::fmt::Write;
                let mut projection = String::new();
                for unit in &units {
                    // String's formatter has no I/O error path.
                    write!(&mut projection, "\\u{unit:04x}")
                        .expect("formatting into String cannot fail");
                }
                color.value.css = projection.clone();
                color.raw_css = Some((projection, units));
            }
        }
    }
    pub fn same_object(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl PartialEq for ColorRef {
    fn eq(&self, other: &Self) -> bool {
        self.borrow().rgba == other.borrow().rgba && self.css_units() == other.css_units()
    }
}
impl Eq for ColorRef {}

/// Color objects and caches are shared, like the upstream `IColorSet` members.
#[derive(Clone, Debug)]
pub struct ColorSet {
    pub foreground: ColorRef,
    pub background: ColorRef,
    pub cursor: ColorRef,
    pub cursor_accent: ColorRef,
    pub selection_foreground: Option<ColorRef>,
    pub selection_background_transparent: ColorRef,
    pub selection_background_opaque: ColorRef,
    pub selection_inactive_background_transparent: ColorRef,
    pub selection_inactive_background_opaque: ColorRef,
    pub scrollbar_slider_background: ColorRef,
    pub scrollbar_slider_hover_background: ColorRef,
    pub scrollbar_slider_active_background: ColorRef,
    pub overview_ruler_border: ColorRef,
    pub ansi: Vec<ColorRef>,
    pub contrast_cache: Rc<RefCell<ColorContrastCache>>,
    pub half_contrast_cache: Rc<RefCell<ColorContrastCache>>,
}

pub type SharedColors = Rc<RefCell<ColorSet>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSlot {
    Ansi(u8),
    Foreground,
    Background,
    Cursor,
}

#[derive(Clone)]
pub(super) struct RestoreColors {
    foreground: ColorRef,
    background: ColorRef,
    cursor: ColorRef,
    ansi: Vec<ColorRef>,
}
impl RestoreColors {
    pub fn capture(colors: &ColorSet) -> Self {
        Self {
            foreground: colors.foreground.clone(),
            background: colors.background.clone(),
            cursor: colors.cursor.clone(),
            ansi: colors.ansi.clone(),
        }
    }
    pub fn restore(&self, colors: &mut ColorSet, slot: Option<ColorSlot>) {
        match slot {
            None => {
                for (index, color) in self.ansi.iter().enumerate() {
                    if index < colors.ansi.len() {
                        colors.ansi[index] = color.clone();
                    } else {
                        colors.ansi.push(color.clone());
                    }
                }
            }
            Some(ColorSlot::Foreground) => colors.foreground = self.foreground.clone(),
            Some(ColorSlot::Background) => colors.background = self.background.clone(),
            Some(ColorSlot::Cursor) => colors.cursor = self.cursor.clone(),
            Some(ColorSlot::Ansi(index)) => {
                if let Some(color) = self.ansi.get(usize::from(index)) {
                    let index = usize::from(index);
                    // Public mutation can shorten ansi; source assignment extends it.
                    if colors.ansi.len() <= index {
                        colors.ansi.resize(index + 1, Color::null().into());
                    }
                    colors.ansi[index] = color.clone();
                }
            }
        }
    }
}
