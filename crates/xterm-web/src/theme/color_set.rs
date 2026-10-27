// Copyright (c) 2022 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from browser/Types.ts and ThemeService.ts.
use crate::contrast_cache::ColorContrastCache;
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};
use xterm_core::color::Color;

/// Shared color-object identity, including mutations retained by restore slots.
#[derive(Clone, Debug)]
pub struct ColorRef(Rc<RefCell<Color>>);
impl From<Color> for ColorRef {
    fn from(color: Color) -> Self {
        Self(Rc::new(RefCell::new(color)))
    }
}
impl ColorRef {
    pub fn borrow(&self) -> Ref<'_, Color> {
        self.0.borrow()
    }
    pub fn borrow_mut(&self) -> RefMut<'_, Color> {
        self.0.borrow_mut()
    }
    pub fn same_object(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl PartialEq for ColorRef {
    fn eq(&self, other: &Self) -> bool {
        *self.borrow() == *other.borrow()
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
