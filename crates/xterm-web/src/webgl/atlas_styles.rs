// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Canvas decorations use the source's paths and saved-state boundaries.
use super::{atlas_context::Context, atlas_rasterizer::Rasterizer};
use crate::{
    js_error::to_js_error, renderer::utils::compute_next_variant_offset, theme::ColorRef,
    utf16::to_js_string,
};
use wasm_bindgen::JsValue;
use xterm_core::buffer::{constants::underline_style, AttributeData};

pub(super) struct Underline<'a> {
    pub chars: &'a [u16],
    pub padding: f64,
    pub width: f64,
    pub restrict: bool,
    pub custom: bool,
    pub background: &'a ColorRef,
}
impl Rasterizer {
    pub(super) fn underline(
        &self,
        ctx: &Context<'_>,
        underline: Underline<'_>,
        threshold: &mut bool,
    ) -> Result<(), JsValue> {
        let Underline {
            chars,
            padding,
            width,
            restrict,
            custom,
            background,
        } = underline;
        let d = self.config.dimensions;
        ctx.op("save", &[])?;
        let line_width = (self.config.options.font_size * d.device_pixel_ratio / 15.0)
            .floor()
            .max(1.0);
        let y_offset = if line_width % 2.0 == 1.0 { 0.5 } else { 0.0 };
        ctx.set("lineWidth", &line_width.into())?;
        if self.attrs.borrow().is_underline_color_default() {
            ctx.set("strokeStyle", &ctx.get("fillStyle")?)?;
        } else if self.attrs.borrow().is_underline_color_rgb() {
            *threshold = false;
            let [r, g, b] =
                AttributeData::to_color_rgb(self.attrs.borrow().get_underline_color() as u32);
            ctx.set("strokeStyle", &format!("rgb({r},{g},{b})").into())?;
        } else {
            *threshold = false;
            let mut fg = self.attrs.borrow().get_underline_color();
            if self.config.options.draw_bold_text_in_bright_colors
                && self.attrs.borrow().is_bold() != 0
                && fg < 8
            {
                fg += 8;
            }
            let color = self.colors.palette(fg).map_err(|e| to_js_error(&e))?;
            ctx.string("strokeStyle", &color.css_units())?;
        }
        ctx.set("fillStyle", &ctx.get("strokeStyle")?)?;
        ctx.op("beginPath", &[])?;
        let top = (padding + d.device_char_height).ceil()
            - y_offset
            - if restrict { line_width * 2.0 } else { 0.0 };
        let bottom = top + line_width * 2.0;
        let mut next = f64::from(self.attrs.borrow().get_underline_variant_offset());
        let mut y_top = 0.0;
        let mut y_bottom = 0.0;
        let mut i = 0.0;
        while i < width {
            ctx.op("save", &[])?;
            y_top = top;
            y_bottom = bottom;
            let left = padding + i * d.device_cell_width;
            let right = padding + (i + 1.0) * d.device_cell_width;
            let style = self.attrs.borrow().extended.underline_style();
            let mut filled = false;
            match style {
                underline_style::DOUBLE => {
                    line(ctx, left, top, right, top)?;
                    line(ctx, left, bottom, right, bottom)?;
                }
                underline_style::CURLY => {
                    y_top = d.device_char_height + 1.0;
                    y_bottom = y_top + 3.0 * d.device_pixel_ratio;
                    ctx.clip_rect([left, y_top, d.device_cell_width, y_bottom - y_top])?;
                    let sx = d.device_cell_width / 6.0;
                    let sy = (y_bottom - y_top) / 3.0;
                    for polygon in [
                        &[0.0, 2.0, 1.0, 3.0, 2.4, 3.0, 0.0, 0.6][..],
                        &[5.5, 0.0, 2.5, 3.0, 1.1, 3.0, 4.1, 0.0],
                        &[4.0, 0.0, 6.0, 2.0, 6.0, 0.6, 5.4, 0.0],
                    ] {
                        ctx.op("beginPath", &[])?;
                        for (j, p) in polygon.chunks_exact(2).enumerate() {
                            ctx.op(
                                if j == 0 { "moveTo" } else { "lineTo" },
                                &[left + p[0] * sx, y_bottom - p[1] * sy],
                            )?;
                        }
                        ctx.op("closePath", &[])?;
                        ctx.op("fill", &[])?;
                    }
                    filled = true;
                }
                underline_style::DOTTED => {
                    let offset = if next == 0.0 {
                        0.0
                    } else if next >= line_width {
                        line_width * 2.0 - next
                    } else {
                        line_width - next
                    };
                    let dash = js_sys::Array::new();
                    dash.push(&line_width.round().into());
                    dash.push(&line_width.round().into());
                    ctx.call("setLineDash", &[dash.into()])?;
                    if next >= line_width || offset == 0.0 {
                        line(ctx, left + offset, top, right, top)?;
                    } else {
                        line(ctx, left, top, left + offset, top)?;
                        line(ctx, left + offset + line_width, top, right, top)?;
                    }
                    next = compute_next_variant_offset(right - left, line_width, Some(next));
                }
                underline_style::DASHED => {
                    let cell_width = right - left;
                    let line_width = (0.6 * cell_width).floor();
                    let gap = (0.3 * cell_width).floor();
                    let dash = js_sys::Array::new();
                    for value in [line_width, gap, cell_width - line_width - gap] {
                        dash.push(&value.into());
                    }
                    ctx.call("setLineDash", &[dash.into()])?;
                    line(ctx, left, top, right, top)?;
                }
                _ => line(ctx, left, top, right, top)?,
            }
            if !filled {
                ctx.op("stroke", &[])?;
            }
            ctx.op("restore", &[])?;
            i += 1.0;
        }
        ctx.op("restore", &[])?;
        if !custom
            && self.config.options.font_size >= 12.0
            && !self.config.options.allow_transparency
            && chars != [32]
        {
            ctx.op("save", &[])?;
            ctx.set("textBaseline", &"alphabetic".into())?;
            let metrics = ctx.call("measureText", &[to_js_string(chars).into()])?;
            ctx.op("restore", &[])?;
            if js_sys::Reflect::has(&metrics, &"actualBoundingBoxDescent".into())?
                && js_sys::Reflect::get(&metrics, &"actualBoundingBoxDescent".into())?
                    .as_f64()
                    .is_some_and(|v| v > 0.0)
            {
                ctx.op("save", &[])?;
                ctx.clip_rect([
                    padding,
                    y_top - (line_width / 2.0).ceil(),
                    d.device_cell_width * width,
                    y_bottom - y_top + (line_width / 2.0).ceil(),
                ])?;
                ctx.set("lineWidth", &(d.device_pixel_ratio * 3.0).into())?;
                ctx.string("strokeStyle", &background.css_units())?;
                ctx.text("strokeText", chars, padding, padding + d.device_char_height)?;
                ctx.op("restore", &[])?;
            }
        }
        Ok(())
    }
    pub(super) fn overline(
        &self,
        ctx: &Context<'_>,
        padding: f64,
        width: f64,
    ) -> Result<(), JsValue> {
        let d = self.config.dimensions;
        let line_width = (self.config.options.font_size * d.device_pixel_ratio / 15.0)
            .floor()
            .max(1.0);
        let offset = if line_width % 2.0 == 1.0 { 0.5 } else { 0.0 };
        ctx.set("lineWidth", &line_width.into())?;
        ctx.set("strokeStyle", &ctx.get("fillStyle")?)?;
        ctx.op("beginPath", &[])?;
        line(
            ctx,
            padding,
            padding + offset,
            padding + d.device_char_width * width,
            padding + offset,
        )?;
        ctx.op("stroke", &[])
    }
    pub(super) fn strikethrough(
        &self,
        ctx: &Context<'_>,
        padding: f64,
        width: f64,
    ) -> Result<(), JsValue> {
        let d = self.config.dimensions;
        let line_width = (self.config.options.font_size * d.device_pixel_ratio / 10.0)
            .floor()
            .max(1.0);
        // Deliberately read the OLD width before assigning the new strike width.
        let offset = if ctx
            .get("lineWidth")?
            .as_f64()
            .is_some_and(|v| v % 2.0 == 1.0)
        {
            0.5
        } else {
            0.0
        };
        ctx.set("lineWidth", &line_width.into())?;
        ctx.set("strokeStyle", &ctx.get("fillStyle")?)?;
        ctx.op("beginPath", &[])?;
        let y = padding + (d.device_char_height / 2.0).floor() - offset;
        line(ctx, padding, y, padding + d.device_char_width * width, y)?;
        ctx.op("stroke", &[])
    }
}
fn line(ctx: &Context<'_>, x: f64, y: f64, right: f64, bottom: f64) -> Result<(), JsValue> {
    ctx.op("moveTo", &[x, y])?;
    ctx.op("lineTo", &[right, bottom])
}
