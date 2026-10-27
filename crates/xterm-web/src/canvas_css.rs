// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Browser fallback from common/Color.ts.
use crate::theme::ColorParser;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasGradient, CanvasRenderingContext2d, Document, HtmlCanvasElement};
use xterm_core::color::{css, Color};

/// One private pixel canvas per parser. Drawing uses copy compositing so the
/// previous parsed color cannot contaminate a later sample.
pub struct CanvasColorParser {
    context: CanvasRenderingContext2d,
    litmus: CanvasGradient,
}
impl CanvasColorParser {
    pub fn new(document: &Document) -> Result<Self, JsValue> {
        let canvas: HtmlCanvasElement = document.create_element("canvas")?.dyn_into()?;
        canvas.set_width(1);
        canvas.set_height(1);
        let options = js_sys::Object::new();
        js_sys::Reflect::set(
            &options,
            &JsValue::from_str("willReadFrequently"),
            &JsValue::TRUE,
        )?;
        let context: CanvasRenderingContext2d = canvas
            .get_context_with_context_options("2d", &options)?
            .ok_or_else(|| js_sys::Error::new("Canvas 2D context is unavailable"))?
            .dyn_into()?;
        context.set_global_composite_operation("copy")?;
        let litmus = context.create_linear_gradient(0.0, 0.0, 1.0, 1.0);
        Ok(Self { context, litmus })
    }

    pub fn to_color(&self, input: &str) -> Result<Color, JsValue> {
        if let Ok(color) = css::to_color(input) {
            return Ok(color);
        }
        self.context.set_fill_style_canvas_gradient(&self.litmus);
        self.context.set_fill_style_str(input);
        // Invalid assignments leave the gradient unchanged. Comparing only with
        // another string color would incorrectly reject valid identical values.
        if !self.context.fill_style().is_string() {
            return Err(unsupported());
        }
        self.context.fill_rect(0.0, 0.0, 1.0, 1.0);
        let data = self.context.get_image_data(0.0, 0.0, 1.0, 1.0)?.data();
        let pixel: [u8; 4] = data.0.as_slice().try_into().map_err(|_| {
            js_sys::Error::new("Canvas color sample must contain exactly one pixel")
        })?;
        if pixel[3] != 255 {
            return Err(unsupported());
        }
        Ok(Color {
            css: input.into(),
            rgba: u32::from_be_bytes(pixel),
        })
    }

    /// ThemeService intentionally converts every parse failure to its specified
    /// fallback, matching the source's parseColor try/catch. Direct callers of
    /// to_color retain actual canvas DOM errors instead of swallowing them.
    pub fn theme_parser(self: &Rc<Self>) -> ColorParser {
        let parser = Rc::clone(self);
        Rc::new(move |input| parser.to_color(input).map_err(|_| css::UnsupportedCss))
    }
}
fn unsupported() -> JsValue {
    js_sys::Error::new(&css::UnsupportedCss.to_string()).into()
}
