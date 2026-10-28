// Copyright (c) 2018, 2023 The xterm.js authors. All rights reserved. MIT license.
use super::row_types::{RenderCell, RowDecoration, RowError, RowOptions, RowSpan};
use crate::theme::{ColorRef, ColorSet};
use xterm_core::{
    buffer::constants::attributes as mode,
    color::{channels, value, Color},
};

pub(super) fn palette<E>(colors: &ColorSet, index: i32) -> Result<Color, RowError<E>> {
    usize::try_from(index)
        .ok()
        .and_then(|i| colors.ansi.get(i))
        .map(|color| color.borrow().clone())
        .ok_or(RowError::Palette(index))
}
pub(super) fn apply<E>(
    span: &mut RowSpan,
    cell: &RenderCell,
    options: &RowOptions,
    colors: &ColorSet,
    decorations: &[RowDecoration],
    selected: bool,
) -> Result<(), RowError<E>> {
    let attrs = &cell.attrs;
    let (mut fg, mut fg_mode, mut bg, mut bg_mode) = (
        attrs.get_fg_color(),
        attrs.get_fg_color_mode(),
        attrs.get_bg_color(),
        attrs.get_bg_color_mode(),
    );
    let inverse = attrs.is_inverse() != 0;
    if inverse {
        std::mem::swap(&mut fg, &mut bg);
        std::mem::swap(&mut fg_mode, &mut bg_mode);
    }
    let (mut bg_override, mut fg_override) = (None, None);
    let mut top = false;
    for decoration in decorations {
        if !decoration.top && top {
            continue;
        }
        if let Some(color) = &decoration.background {
            bg_mode = mode::CM_RGB;
            bg = (color.rgba >> 8) as i32;
            bg_override = Some(color.clone());
        }
        if let Some(color) = &decoration.foreground {
            fg_mode = mode::CM_RGB;
            fg = (color.rgba >> 8) as i32;
            fg_override = Some(color.clone());
        }
        top = decoration.top;
    }
    if !top && selected {
        let background = if options.focused {
            &colors.selection_background_opaque
        } else {
            &colors.selection_inactive_background_opaque
        };
        let color = background.borrow().clone();
        bg = (color.rgba >> 8) as i32;
        bg_mode = mode::CM_RGB;
        bg_override = Some(color);
        top = true;
        if let Some(color) = &colors.selection_foreground {
            let color = color.borrow().clone();
            fg_mode = mode::CM_RGB;
            fg = (color.rgba >> 8) as i32;
            fg_override = Some(color);
        }
    }
    if top {
        span.classes.push("xterm-decoration-top".into());
    }
    let resolved_bg = match bg_mode {
        mode::CM_P16 | mode::CM_P256 => {
            span.classes.push(format!("xterm-bg-{bg}"));
            usize::try_from(bg)
                .ok()
                .and_then(|i| colors.ansi.get(i))
                .map(|c| c.borrow().clone())
        }
        mode::CM_RGB => {
            span.style("background-color", format!("#{bg:06x}"));
            Some(rgb_color(bg))
        }
        _ => Some(if inverse {
            span.classes.push("xterm-bg-257".into());
            colors.foreground.borrow().clone()
        } else {
            colors.background.borrow().clone()
        }),
    };
    if bg_override.is_none() && attrs.is_dim() != 0 {
        bg_override = Some(value::multiply_opacity(
            resolved_bg.as_ref().ok_or(RowError::Palette(bg))?,
            0.5,
        ));
    }
    // Source tests use a short palette and ratio1. Accessing missing colors is
    // harmless until a dim/contrast operation actually reads their channels.
    let needs_contrast = options.minimum_contrast_ratio != 1.0 && !background_glyph(cell.code);
    match fg_mode {
        mode::CM_P16 | mode::CM_P256 => {
            if attrs.is_bold() != 0 && fg < 8 && options.bold_bright {
                fg += 8;
            }
            if !needs_contrast
                || !apply_contrast(
                    span,
                    cell,
                    options,
                    colors,
                    ContrastInput {
                        bg: resolved_bg.as_ref().ok_or(RowError::Palette(bg))?,
                        fg: &palette::<E>(colors, fg)?,
                        bg_override: bg_override.as_ref(),
                        fg_override: None,
                    },
                )
            {
                span.classes.push(format!("xterm-fg-{fg}"));
            }
        }
        mode::CM_RGB => {
            if !needs_contrast
                || !apply_contrast(
                    span,
                    cell,
                    options,
                    colors,
                    ContrastInput {
                        bg: resolved_bg.as_ref().ok_or(RowError::Palette(bg))?,
                        fg: &rgb_color(fg),
                        bg_override: bg_override.as_ref(),
                        fg_override: fg_override.as_ref(),
                    },
                )
            {
                span.style("color", format!("#{fg:06x}"));
            }
        }
        _ => {
            if (!needs_contrast
                || !apply_contrast(
                    span,
                    cell,
                    options,
                    colors,
                    ContrastInput {
                        bg: resolved_bg.as_ref().ok_or(RowError::Palette(bg))?,
                        fg: &colors.foreground.borrow(),
                        bg_override: bg_override.as_ref(),
                        fg_override: fg_override.as_ref(),
                    },
                ))
                && inverse
            {
                span.classes.push("xterm-fg-257".into());
            }
        }
    }
    Ok(())
}
fn rgb_color(color: i32) -> Color {
    channels::to_color(
        i64::from((color >> 16) & 255),
        i64::from((color >> 8) & 255),
        i64::from(color & 255),
        None,
    )
}
fn background_glyph(code: Option<u32>) -> bool {
    code.is_some_and(super::utils::treat_glyph_as_background_color)
}
struct ContrastInput<'a> {
    bg: &'a Color,
    fg: &'a Color,
    bg_override: Option<&'a Color>,
    fg_override: Option<&'a Color>,
}
fn apply_contrast(
    span: &mut RowSpan,
    cell: &RenderCell,
    options: &RowOptions,
    colors: &ColorSet,
    input: ContrastInput<'_>,
) -> bool {
    if options.minimum_contrast_ratio == 1.0 || background_glyph(cell.code) {
        return false;
    }
    let cache = if cell.attrs.is_dim() != 0 {
        &colors.half_contrast_cache
    } else {
        &colors.contrast_cache
    };
    let mut adjusted = None;
    if input.bg_override.is_none() && input.fg_override.is_none() {
        adjusted = cache
            .borrow()
            .get_color(input.bg.rgba, input.fg.rgba)
            .cloned();
    }
    let result = match adjusted {
        Some(color) => color,
        None => {
            let ratio =
                options.minimum_contrast_ratio / if cell.attrs.is_dim() != 0 { 2.0 } else { 1.0 };
            let bg = input.bg_override.unwrap_or(input.bg);
            let fg = input.fg_override.unwrap_or(input.fg);
            let result = value::ensure_contrast_ratio(bg, fg, ratio).map(ColorRef::from);
            cache
                .borrow_mut()
                .set_color(bg.rgba, fg.rgba, result.clone());
            result
        }
    };
    if let Some(color) = result {
        span.style("color", color.borrow().css.clone());
        true
    } else {
        false
    }
}
