// Copyright (c) 2023 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from browser/renderer/dom/WidthCache.ts.
use super::width_cache::{Font, FontVariant, WidthMeasurer};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlCanvasElement, OffscreenCanvas,
    OffscreenCanvasRenderingContext2d, TextMetrics,
};

enum Context {
    Dom(CanvasRenderingContext2d),
    Offscreen(OffscreenCanvasRenderingContext2d),
}
impl Context {
    fn new(document: &Document) -> Result<Self, JsValue> {
        if js_sys::Reflect::has(&js_sys::global(), &JsValue::from_str("OffscreenCanvas"))? {
            let canvas = OffscreenCanvas::new(1, 1)?;
            let context = canvas
                .get_context("2d")?
                .ok_or_else(|| js_sys::Error::new("value must not be falsy"))?;
            Ok(Self::Offscreen(context.dyn_into()?))
        } else {
            let canvas: HtmlCanvasElement = document.create_element("canvas")?.dyn_into()?;
            canvas.set_width(1);
            canvas.set_height(1);
            let context = canvas
                .get_context("2d")?
                .ok_or_else(|| js_sys::Error::new("value must not be falsy"))?;
            Ok(Self::Dom(context.dyn_into()?))
        }
    }
    fn set_font(&self, font: &str) {
        match self {
            Self::Dom(context) => context.set_font(font),
            Self::Offscreen(context) => context.set_font(font),
        }
    }
    fn as_js(&self) -> &JsValue {
        match self {
            Self::Dom(context) => context.as_ref(),
            Self::Offscreen(context) => context.as_ref(),
        }
    }
    fn measure(&self, text: &[u16]) -> Result<f64, JsValue> {
        let context = self.as_js();
        let method: js_sys::Function =
            js_sys::Reflect::get(context, &JsValue::from_str("measureText"))?.dyn_into()?;
        // The web-sys &str overload would replace lone UTF-16 surrogates.
        let metrics: TextMetrics = method
            .call1(context, &crate::utf16::to_js_string(text))?
            .dyn_into()?;
        Ok(metrics.width())
    }
}

pub struct CanvasWidthMeasurer {
    contexts: [Context; 4],
}
impl CanvasWidthMeasurer {
    pub fn new(document: &Document) -> Result<Self, JsValue> {
        // Each font variant needs its own canvas/context, as in the reference.
        Ok(Self {
            contexts: [
                Context::new(document)?,
                Context::new(document)?,
                Context::new(document)?,
                Context::new(document)?,
            ],
        })
    }
}
impl WidthMeasurer for CanvasWidthMeasurer {
    type Error = JsValue;
    fn configure(&mut self, font: &Font) -> Result<(), JsValue> {
        for (index, context) in self.contexts.iter().enumerate() {
            let italic = if index >= FontVariant::Italic as usize {
                "italic"
            } else {
                ""
            };
            let weight = if index % 2 == 1 {
                &font.weight_bold
            } else {
                &font.weight
            };
            context.set_font(format!("{italic} {weight} {}px {}", font.size, font.family).trim());
        }
        Ok(())
    }
    fn measure(&mut self, text: &[u16], variant: FontVariant) -> Result<f64, JsValue> {
        self.contexts[variant as usize].measure(text)
    }
}
