// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! TextureAtlas's temporary-canvas text and custom-character rasterizer.
use super::{
    atlas_canvas::{prepare_canvas, GlyphBitmap, PixelOptions},
    atlas_colors::AtlasColors,
    atlas_context::Context,
    atlas_pixels::{clear_background, GlyphScan},
    config::{AtlasConfig, FontWeight},
    custom_glyphs::{try_draw_custom_glyph, CustomGlyphDrawParams},
    texture_atlas::{GlyphKey, TEXTURE_SIZE},
};
use crate::{
    api::Size,
    js_error::to_js_error,
    renderer::utils::{
        is_powerline_glyph, is_restricted_powerline_glyph, treat_glyph_as_background_color,
    },
    utf16::set_property,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement};
use xterm_core::{
    buffer::{AttributeData, ExtendedAttrs},
    log_service::LogService,
    unicode_service::UnicodeService,
};

const GLYPH_PADDING: f64 = 2.0;
pub(super) struct Rasterizer {
    pub config: Rc<AtlasConfig>,
    pub colors: AtlasColors,
    pub attrs: RefCell<AttributeData>,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    upload_origin: Cell<[f64; 2]>,
    unicode: Rc<UnicodeService>,
    log: Rc<LogService>,
}
impl Rasterizer {
    pub fn new(
        document: &Document,
        config: Rc<AtlasConfig>,
        colors: AtlasColors,
        unicode: Rc<UnicodeService>,
        log: Rc<LogService>,
    ) -> Result<Self, JsValue> {
        let canvas: HtmlCanvasElement = document.create_element("canvas")?.unchecked_into();
        let d = config.dimensions;
        set_property(
            canvas.as_ref(),
            "width",
            &(d.device_cell_width * 4.0 + GLYPH_PADDING * 2.0).into(),
        )?;
        set_property(
            canvas.as_ref(),
            "height",
            &(d.device_cell_height + GLYPH_PADDING * 2.0).into(),
        )?;
        let options = js_sys::Object::new();
        set_property(
            options.as_ref(),
            "alpha",
            &config.options.allow_transparency.into(),
        )?;
        set_property(options.as_ref(), "willReadFrequently", &true.into())?;
        let context = canvas
            .get_context_with_context_options("2d", &options)?
            .ok_or_else(|| js_sys::Error::new("value must not be falsy"))?
            .unchecked_into();
        Ok(Self {
            config,
            colors,
            attrs: RefCell::new(AttributeData::default()),
            canvas,
            context,
            upload_origin: Cell::new([0.0, 0.0]),
            unicode,
            log,
        })
    }
    pub fn upload_origin(&self) -> [f64; 2] {
        self.upload_origin.get()
    }
    pub fn remove_canvas(&self) -> Result<(), JsValue> {
        let method = js_sys::Reflect::get(self.canvas.as_ref(), &"remove".into())?;
        method
            .dyn_ref::<js_sys::Function>()
            .ok_or_else(|| js_sys::TypeError::new("remove is not a function"))?
            .call0(self.canvas.as_ref())?;
        Ok(())
    }
    pub fn draw(
        &self,
        key: &GlyphKey,
        bg: u32,
        fg: u32,
        ext: u32,
        restrict: bool,
        element: Option<&HtmlElement>,
    ) -> Result<Option<GlyphBitmap>, JsValue> {
        let chars = match key {
            GlyphKey::Number(code) => vec![*code as u16],
            GlyphKey::Text(chars) => chars.clone(),
        };
        if let Some(element) = element {
            if self.canvas.parent_element().as_ref() != Some(element.as_ref()) {
                set_property(self.canvas.style().as_ref(), "display", &"none".into())?;
                let method = js_sys::Reflect::get(element.as_ref(), &"append".into())?;
                method
                    .dyn_ref::<js_sys::Function>()
                    .ok_or_else(|| js_sys::TypeError::new("append is not a function"))?
                    .call1(element.as_ref(), self.canvas.as_ref())?;
            }
        }
        let d = self.config.dimensions;
        let allowed = (d.device_cell_width * chars.len().max(2) as f64 + GLYPH_PADDING * 2.0)
            .min(d.device_max_texture_size);
        if f64::from(self.canvas.width()) < allowed {
            set_property(self.canvas.as_ref(), "width", &allowed.into())?;
        }
        let allowed_height = (d.device_cell_height + GLYPH_PADDING * 4.0).min(TEXTURE_SIZE.into());
        if f64::from(self.canvas.height()) < allowed_height {
            set_property(self.canvas.as_ref(), "height", &allowed_height.into())?;
        }
        let ctx = Context(&self.context);
        ctx.op("save", &[])?;
        *self.attrs.borrow_mut() = AttributeData {
            bg,
            fg,
            extended: ExtendedAttrs::new(ext, 0),
        };
        // This early return intentionally leaves the saved frame on the source canvas stack.
        if self.attrs.borrow().is_invisible() != 0 {
            return Ok(None);
        }
        let attrs = self.attrs.borrow().clone();
        let bold = attrs.is_bold() != 0;
        let italic = attrs.is_italic() != 0;
        let underline = attrs.is_underline() != 0;
        let overline = attrs.is_overline() != 0;
        let strikethrough = attrs.is_strikethrough() != 0;
        let paint = AtlasColors::paint(&attrs);
        let background = self
            .colors
            .background_for(paint)
            .map_err(|e| to_js_error(&e))?;
        ctx.set("globalCompositeOperation", &"copy".into())?;
        ctx.string("fillStyle", &background.css_units())?;
        ctx.op(
            "fillRect",
            &[
                0.0,
                0.0,
                self.canvas.width().into(),
                self.canvas.height().into(),
            ],
        )?;
        ctx.set("globalCompositeOperation", &"source-over".into())?;
        ctx.string("font", &self.font(bold, italic))?;
        ctx.set("textBaseline", &super::atlas_context::text_baseline())?;
        let first = chars.first().copied().map(u32::from);
        let powerline = chars.len() == 1 && first.is_some_and(is_powerline_glyph);
        let restricted = chars.len() == 1 && first.is_some_and(is_restricted_powerline_glyph);
        let foreground = self
            .colors
            .foreground_for(paint, first.is_some_and(treat_glyph_as_background_color))
            .map_err(|e| to_js_error(&e))?;
        ctx.string("fillStyle", &foreground.css_units())?;
        let padding = if restricted { 0.0 } else { GLYPH_PADDING * 2.0 };
        let custom = if self.config.custom_glyphs {
            let variant = self.attrs.borrow().get_underline_variant_offset() as i32;
            let background_css = background.borrow().css.clone();
            try_draw_custom_glyph(
                &self.context,
                &chars,
                CustomGlyphDrawParams {
                    x_offset: padding,
                    y_offset: padding,
                    device_cell_width: d.device_cell_width,
                    device_cell_height: d.device_cell_height,
                    device_char_width: d.device_char_width,
                    device_char_height: d.device_char_height,
                    font_size: self.config.options.font_size,
                    device_pixel_ratio: d.device_pixel_ratio,
                    background_color: Some(&background_css),
                    variant_offset: variant,
                },
                &self.log,
            )?
        } else {
            false
        };
        let mut threshold = !powerline;
        let width = match key {
            GlyphKey::Number(code) => self
                .unicode
                .wcwidth_i32(*code)
                .map(|w| i64::from(w.columns())),
            GlyphKey::Text(chars) => self.unicode.get_string_cell_width_utf16(chars),
        }
        .map_err(|e| to_js_error(&e))? as f64;
        if underline {
            self.underline(
                &ctx,
                super::atlas_styles::Underline {
                    chars: &chars,
                    padding,
                    width,
                    restrict,
                    custom,
                    background: &background,
                },
                &mut threshold,
            )?;
        }
        if overline {
            self.overline(&ctx, padding, width)?;
        }
        if !custom {
            ctx.text("fillText", &chars, padding, padding + d.device_char_height)?;
        }
        if chars == [b'_' as u16] && !self.config.options.allow_transparency {
            let mut beyond = self.underscore_empty(padding, &background, &foreground, threshold)?;
            if beyond {
                for offset in 1..=5 {
                    ctx.op("save", &[])?;
                    ctx.string("fillStyle", &background.css_units())?;
                    ctx.op(
                        "fillRect",
                        &[
                            0.0,
                            0.0,
                            self.canvas.width().into(),
                            self.canvas.height().into(),
                        ],
                    )?;
                    ctx.op("restore", &[])?;
                    ctx.text(
                        "fillText",
                        &chars,
                        padding,
                        padding + d.device_char_height - f64::from(offset),
                    )?;
                    beyond = self.underscore_empty(padding, &background, &foreground, threshold)?;
                    if !beyond {
                        break;
                    }
                }
            }
        }
        if strikethrough {
            self.strikethrough(&ctx, padding, width)?;
        }
        ctx.op("restore", &[])?;
        let background_rgba = background.borrow().rgba;
        let foreground_rgba = foreground.borrow().rgba;
        let bitmap = prepare_canvas(
            &self.context,
            GlyphScan {
                allowed_width: allowed as usize,
                padding: padding as usize,
                restricted,
                custom,
                cell: Size {
                    width: d.device_cell_width,
                    height: d.device_cell_height,
                },
                character: Size {
                    width: d.device_char_width,
                    height: d.device_char_height,
                },
                line_height: self.config.options.line_height,
            },
            PixelOptions {
                background_rgba,
                foreground_rgba,
                allow_transparency: self.config.options.allow_transparency,
                clear_threshold: threshold,
            },
        )?;
        if let Some(bitmap) = &bitmap {
            self.upload_origin
                .set([bitmap.bounds.left as f64, bitmap.bounds.top as f64]);
        }
        Ok(bitmap)
    }
    fn font(&self, bold: bool, italic: bool) -> Vec<u16> {
        let o = &self.config.options;
        let weight = if bold {
            &o.font_weight_bold
        } else {
            &o.font_weight
        };
        let mut result = if italic { "italic " } else { " " }
            .encode_utf16()
            .collect::<Vec<_>>();
        match weight {
            FontWeight::Text(units) => result.extend(units.iter().copied()),
            FontWeight::Number(number) => {
                result.extend(ryu_js::Buffer::new().format(*number).encode_utf16())
            }
        }
        result.push(32);
        result.extend(
            ryu_js::Buffer::new()
                .format(o.font_size * self.config.dimensions.device_pixel_ratio)
                .encode_utf16(),
        );
        result.extend("px ".encode_utf16());
        result.extend(o.font_family.iter().copied());
        result
    }
    fn underscore_empty(
        &self,
        padding: f64,
        bg: &crate::theme::ColorRef,
        fg: &crate::theme::ColorRef,
        threshold: bool,
    ) -> Result<bool, JsValue> {
        let d = self.config.dimensions;
        let mut image = self
            .context
            .get_image_data(padding, padding, d.device_cell_width, d.device_cell_height)?
            .data()
            .0;
        clear_background(&mut image, bg.borrow().rgba, fg.borrow().rgba, threshold)
            .map_err(|e| to_js_error(&e))
    }
}
