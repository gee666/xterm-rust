// Copyright (c) 2022 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from browser/services/ThemeService.ts.
use super::{palette::default_ansi_colors, ColorRef, ColorSet, Theme};
use crate::contrast_cache::ColorContrastCache;
use std::{cell::RefCell, rc::Rc};
use xterm_core::color::{channels, css::UnsupportedCss, value, Color};

type Parser = dyn Fn(&str) -> Result<Color, UnsupportedCss>;
type Caches = (
    Rc<RefCell<ColorContrastCache>>,
    Rc<RefCell<ColorContrastCache>>,
);

pub(super) struct Defaults {
    foreground: ColorRef,
    background: ColorRef,
    cursor: ColorRef,
    selection: ColorRef,
    ansi: Vec<ColorRef>,
}
impl Defaults {
    pub fn new() -> Self {
        Self {
            foreground: channels::to_color(255, 255, 255, None).into(),
            background: channels::to_color(0, 0, 0, None).into(),
            cursor: channels::to_color(255, 255, 255, None).into(),
            selection: Color {
                css: "rgba(255, 255, 255, 0.3)".into(),
                rgba: 0xffffff4d,
            }
            .into(),
            ansi: default_ansi_colors().map(ColorRef::from).into(),
        }
    }
}
fn parse(input: Option<&str>, fallback: &ColorRef, parser: &Parser) -> ColorRef {
    input
        .and_then(|input| parser(input).ok())
        .map(ColorRef::from)
        .unwrap_or_else(|| fallback.clone())
}
fn opacity(color: &ColorRef, factor: f64) -> ColorRef {
    value::opacity(&color.borrow(), factor).into()
}
fn blend(bg: &ColorRef, fg: &ColorRef) -> ColorRef {
    value::blend(&bg.borrow(), &fg.borrow()).into()
}
fn selection_transparent(color: ColorRef) -> ColorRef {
    if value::is_opaque(&color.borrow()) {
        opacity(&color, 0.3)
    } else {
        color
    }
}

pub(super) fn colors(
    theme: &Theme,
    defaults: &Defaults,
    parser: &Parser,
    caches: Option<Caches>,
) -> ColorSet {
    let foreground = parse(theme.foreground.as_deref(), &defaults.foreground, parser);
    let background = parse(theme.background.as_deref(), &defaults.background, parser);
    let cursor = blend(
        &background,
        &parse(theme.cursor.as_deref(), &defaults.cursor, parser),
    );
    let cursor_accent = blend(
        &background,
        &parse(theme.cursor_accent.as_deref(), &defaults.background, parser),
    );
    let selection = parse(
        theme.selection_background.as_deref(),
        &defaults.selection,
        parser,
    );
    let inactive_selection = parse(
        theme.selection_inactive_background.as_deref(),
        &selection,
        parser,
    );
    // The opaque selection is resolved before the special 0.3 transparent-copy
    // adjustment. Reordering these operations changes opaque custom selections.
    let selection_background_opaque = blend(&background, &selection);
    let selection_inactive_background_opaque = blend(&background, &inactive_selection);
    let selection_foreground = theme
        .selection_foreground
        .as_deref()
        .filter(|s| !s.is_empty())
        .and_then(|input| parser(input).ok())
        .map(ColorRef::from);
    let scrollbar_slider_background = parse(
        theme.scrollbar_slider_background.as_deref(),
        &opacity(&foreground, 0.2),
        parser,
    );
    let scrollbar_slider_hover_background = parse(
        theme.scrollbar_slider_hover_background.as_deref(),
        &opacity(&foreground, 0.4),
        parser,
    );
    let scrollbar_slider_active_background = parse(
        theme.scrollbar_slider_active_background.as_deref(),
        &opacity(&foreground, 0.5),
        parser,
    );
    let overview_ruler_border = parse(
        theme.overview_ruler_border.as_deref(),
        &defaults.foreground,
        parser,
    );
    let mut ansi = defaults.ansi.clone();
    for (index, input) in theme.ansi.iter().enumerate() {
        ansi[index] = parse(input.as_deref(), &defaults.ansi[index], parser);
    }
    for (index, input) in theme.extended_ansi.iter().take(240).enumerate() {
        ansi[index + 16] = parse(input.as_deref(), &defaults.ansi[index + 16], parser);
    }
    let (contrast_cache, half_contrast_cache) = caches.unwrap_or_default();
    ColorSet {
        foreground,
        background,
        cursor,
        cursor_accent,
        selection_foreground,
        selection_background_transparent: selection_transparent(selection),
        selection_background_opaque,
        selection_inactive_background_transparent: selection_transparent(inactive_selection),
        selection_inactive_background_opaque,
        scrollbar_slider_background,
        scrollbar_slider_hover_background,
        scrollbar_slider_active_background,
        overview_ruler_border,
        ansi,
        contrast_cache,
        half_contrast_cache,
    }
}
