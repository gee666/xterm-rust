// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from browser/services/CharSizeService.ts.
use crate::{char_size::CharSizeState, renderer::width_cache::Font};
use std::cell::RefCell;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, HtmlElement, OffscreenCanvas, OffscreenCanvasRenderingContext2d};

const MEASURE_REPEAT: usize = 32;
enum Strategy {
    Metrics(OffscreenCanvasRenderingContext2d),
    Dom(HtmlElement),
}
impl Strategy {
    fn metrics() -> Result<Self, JsValue> {
        let canvas = OffscreenCanvas::new(100, 100)?;
        let context: OffscreenCanvasRenderingContext2d = canvas
            .get_context("2d")?
            .ok_or_else(|| js_sys::Error::new("Canvas 2D context is unavailable"))?
            .dyn_into()?;
        let metrics = context.measure_text("W")?;
        for field in ["width", "fontBoundingBoxAscent", "fontBoundingBoxDescent"] {
            if !js_sys::Reflect::has(metrics.as_ref(), &JsValue::from_str(field))? {
                return Err(js_sys::Error::new("Required font metrics not supported").into());
            }
        }
        Ok(Self::Metrics(context))
    }
    fn dom(document: &Document, parent: &HtmlElement) -> Result<Self, JsValue> {
        let element: HtmlElement = document.create_element("span")?.dyn_into()?;
        element.set_class_name("xterm-char-measure-element");
        element.set_text_content(Some(&"W".repeat(MEASURE_REPEAT)));
        element.set_attribute("aria-hidden", "true")?;
        element.style().set_property("white-space", "pre")?;
        element.style().set_property("font-kerning", "none")?;
        parent.append_child(&element)?;
        Ok(Self::Dom(element))
    }
    fn measure(&self, font: &Font) -> Result<(Option<f64>, Option<f64>), JsValue> {
        match self {
            Self::Metrics(context) => {
                context.set_font(&format!("{}px {}", font.size, font.family));
                let metrics = context.measure_text("W")?;
                let ascent = js_sys::Reflect::get(
                    metrics.as_ref(),
                    &JsValue::from_str("fontBoundingBoxAscent"),
                )?
                .as_f64();
                let descent = js_sys::Reflect::get(
                    metrics.as_ref(),
                    &JsValue::from_str("fontBoundingBoxDescent"),
                )?
                .as_f64();
                Ok((
                    Some(metrics.width()),
                    ascent.zip(descent).map(|(a, d)| a + d),
                ))
            }
            Self::Dom(element) => {
                element.style().set_property("font-family", &font.family)?;
                element
                    .style()
                    .set_property("font-size", &format!("{}px", font.size))?;
                Ok((
                    Some(f64::from(element.offset_width()) / MEASURE_REPEAT as f64),
                    Some(f64::from(element.offset_height())),
                ))
            }
        }
    }
}
impl Drop for Strategy {
    fn drop(&mut self) {
        if let Self::Dom(element) = self {
            element.remove();
        }
    }
}

pub struct CharSizeService {
    pub state: CharSizeState,
    strategy: Strategy,
    font: RefCell<Font>,
}
impl CharSizeService {
    pub fn new(document: &Document, parent: &HtmlElement, font: Font) -> Result<Self, JsValue> {
        // The reference deliberately falls back on any metric-construction
        // failure, including missing OffscreenCanvas or bounding-box properties.
        let strategy = match Strategy::metrics() {
            Ok(strategy) => strategy,
            Err(_) => Strategy::dom(document, parent)?,
        };
        Ok(Self {
            state: CharSizeState::default(),
            strategy,
            font: RefCell::new(font),
        })
    }
    pub fn measure(&self) -> Result<(), JsValue> {
        let (width, height) = self.strategy.measure(&self.font.borrow())?;
        self.state
            .update_measurement(width, height)
            .map_err(|error| js_sys::Error::new(&error.to_string()).into())
    }
    pub fn set_font(&self, font: Font) -> Result<(), JsValue> {
        *self.font.borrow_mut() = font;
        self.measure()
    }
}
