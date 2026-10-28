// Copyright (c) 2023 The xterm.js authors. All rights reserved. MIT license.
use super::{
    raw_font::{FontDescriptor, RawFont},
    width_cache::{Font, FontVariant, WidthCache, WidthError, WidthMeasurer},
};
use std::marker::PhantomData;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlCanvasElement, OffscreenCanvas,
    OffscreenCanvasRenderingContext2d,
};
#[wasm_bindgen(
    inline_js = "export function xtermOffscreenCanvasAvailable() { return typeof OffscreenCanvas !== 'undefined'; }"
)]
extern "C" {
    #[wasm_bindgen(catch,js_name=xtermOffscreenCanvasAvailable)]
    fn offscreen_canvas_available() -> Result<bool, JsValue>;
}
enum Context {
    Dom(CanvasRenderingContext2d),
    Offscreen(OffscreenCanvasRenderingContext2d),
}
fn set(object: &JsValue, name: &str, value: &JsValue) -> Result<(), JsValue> {
    if js_sys::Reflect::set(object, &name.into(), value)? {
        Ok(())
    } else {
        Err(js_sys::TypeError::new(&format!("Cannot assign canvas {name}")).into())
    }
}
impl Context {
    fn new(document: &Document) -> Result<Self, JsValue> {
        if offscreen_canvas_available()? {
            let canvas = OffscreenCanvas::new(1, 1)?;
            let context = canvas
                .get_context("2d")?
                .ok_or_else(|| js_sys::Error::new("value must not be falsy"))?;
            let value: &JsValue = context.as_ref();
            if !value.is_truthy() {
                return Err(js_sys::Error::new("value must not be falsy").into());
            }
            // The requested IDL interface may belong to another window realm.
            Ok(Self::Offscreen(context.unchecked_into()))
        } else {
            // HTML owner-document creation supplies the canvas interface; a
            // parent-realm instanceof test would reject iframe canvases.
            let canvas: HtmlCanvasElement = document.create_element("canvas")?.unchecked_into();
            set(canvas.as_ref(), "width", &1.into())?;
            set(canvas.as_ref(), "height", &1.into())?;
            let context = canvas
                .get_context("2d")?
                .ok_or_else(|| js_sys::Error::new("value must not be falsy"))?;
            let value: &JsValue = context.as_ref();
            if !value.is_truthy() {
                return Err(js_sys::Error::new("value must not be falsy").into());
            }
            Ok(Self::Dom(context.unchecked_into()))
        }
    }
    fn as_js(&self) -> &JsValue {
        match self {
            Self::Dom(c) => c.as_ref(),
            Self::Offscreen(c) => c.as_ref(),
        }
    }
    fn set_font(&self, units: &[u16]) -> Result<(), JsValue> {
        set(
            self.as_js(),
            "font",
            &crate::utf16::to_js_string(units).into(),
        )
    }
    fn measure(&self, text: &[u16]) -> Result<f64, JsValue> {
        let context = self.as_js();
        let method = js_sys::Reflect::get(context, &"measureText".into())?;
        if !method.is_function() {
            return Err(js_sys::TypeError::new("measureText is not a function").into());
        }
        let method: js_sys::Function = method.unchecked_into();
        let metrics = method.call1(context, &crate::utf16::to_js_string(text))?;
        js_sys::Reflect::get(&metrics, &"width".into())?
            .as_f64()
            .ok_or_else(|| js_sys::TypeError::new("TextMetrics.width must be a number").into())
    }
}
pub struct CanvasWidthMeasurer<F: FontDescriptor = Font> {
    contexts: [Context; 4],
    font: PhantomData<fn() -> F>,
}
impl<F: FontDescriptor> CanvasWidthMeasurer<F> {
    pub fn new(document: &Document) -> Result<Self, JsValue> {
        Ok(Self {
            contexts: [
                Context::new(document)?,
                Context::new(document)?,
                Context::new(document)?,
                Context::new(document)?,
            ],
            font: PhantomData,
        })
    }
}
impl<F: FontDescriptor> WidthMeasurer for CanvasWidthMeasurer<F> {
    type Font = F;
    type Error = JsValue;
    fn configure_variant(&self, font: &F, variant: FontVariant) -> Result<(), JsValue> {
        let units = font
            .variant_units(variant)
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        self.contexts[variant as usize].set_font(&units)
    }
    fn measure(&self, text: &[u16], variant: FontVariant) -> Result<f64, JsValue> {
        self.contexts[variant as usize].measure(text)
    }
}
pub type SourceCanvasCache = WidthCache<CanvasWidthMeasurer<RawFont>>;
impl WidthCache<CanvasWidthMeasurer<RawFont>> {
    /// Caller performs its CSS updates first; reads here are fresh captured raw
    /// arguments, not the renderer's earlier coerced Font snapshot.
    pub fn set_font_from_options(
        &self,
        options: &xterm_core::options::OptionsService,
    ) -> Result<(), WidthError<JsValue>> {
        let font = RawFont::from_options(options)
            .map_err(|e| WidthError::Measure(crate::js_error::to_js_error(&e)))?;
        self.set_font(font)
    }
}
