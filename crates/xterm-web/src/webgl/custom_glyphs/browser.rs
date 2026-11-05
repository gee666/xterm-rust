// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
use super::{
    draw_custom_glyph, patterns::set_pattern, CustomGlyphDrawParams, DrawCommand as C, GlyphCanvas,
};
use crate::js_error::to_js_error;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, Path2d};
use xterm_core::log_service::{LogLevel, LogService};

struct BrowserCanvas<'a> {
    context: &'a CanvasRenderingContext2d,
    log: &'a LogService,
}

/// Rasterize through Rust-owned definitions and path arithmetic into real Canvas2D.
/// Context state and exceptions follow the source; callers own outer save/restore.
pub fn try_draw_custom_glyph(
    context: &CanvasRenderingContext2d,
    chars: &[u16],
    params: CustomGlyphDrawParams<'_>,
    log: &LogService,
) -> Result<bool, JsValue> {
    draw_custom_glyph(&mut BrowserCanvas { context, log }, chars, params)
}

impl GlyphCanvas for BrowserCanvas<'_> {
    type Error = JsValue;
    fn draw(&mut self, command: C) -> Result<(), JsValue> {
        let ctx = self.context;
        match command {
            C::Save => ctx.save(),
            C::Restore => ctx.restore(),
            C::Begin => ctx.begin_path(),
            C::Close => ctx.close_path(),
            C::Move([x, y]) => ctx.move_to(x, y),
            C::Line([x, y]) => ctx.line_to(x, y),
            C::Quadratic([x, y, a, b]) => ctx.quadratic_curve_to(x, y, a, b),
            C::Cubic([x, y, a, b, c, d]) => ctx.bezier_curve_to(x, y, a, b, c, d),
            C::Arc([x, y, r, start, end]) => ctx.arc(x, y, r, start, end)?,
            C::Ellipse([x, y, rx, ry, phi, start, end], reverse) => {
                ctx.ellipse_with_anticlockwise(x, y, rx, ry, phi, start, end, reverse)?
            }
            C::Fill => ctx.fill(),
            C::Stroke => ctx.stroke(),
            C::Rect([x, y, w, h]) => ctx.rect(x, y, w, h),
            C::FillRect([x, y, w, h]) => ctx.fill_rect(x, y, w, h),
            C::Clip => ctx.clip(),
            C::ClipRect([x, y, w, h]) => {
                let path = Path2d::new()?;
                path.rect(x, y, w, h);
                ctx.clip_with_path_2d(&path);
            }
            C::StrokeFromFill => {
                js_sys::Reflect::set(ctx.as_ref(), &"strokeStyle".into(), &ctx.fill_style())?;
            }
            C::Background(color) => {
                ctx.set_fill_style_str(&color);
                ctx.set_stroke_style_str(&color);
            }
            C::LineWidth(width) => ctx.set_line_width(width),
            C::SquareCap => ctx.set_line_cap("square"),
            C::Pattern(id, pattern, variant) => set_pattern(ctx, id, pattern, variant)?,
            C::LogError(message) => self
                .log
                .log(
                    LogLevel::Error,
                    &message.encode_utf16().collect::<Vec<_>>(),
                    Vec::new(),
                )
                .map_err(|error| to_js_error(&error))?,
        }
        Ok(())
    }
}
