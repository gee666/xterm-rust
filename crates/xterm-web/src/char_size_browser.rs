// Copyright (c) 2016 The xterm.js authors. All rights reserved. MIT license.
//! CharSizeService strategies read live options without guards over host calls.
use crate::{char_size::CharSizeState, renderer::width_cache::Font};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{Document, HtmlElement, OffscreenCanvas, OffscreenCanvasRenderingContext2d};
use xterm_core::{
    lifecycle::{Disposable, DisposableHandle},
    options::{OptionError, OptionValue, OptionsService},
};
use xterm_wasm_support::js_error;

#[wasm_bindgen(inline_js = "export function xtermMeasureNumber(value) { return Number(value); }")]
extern "C" {
    #[wasm_bindgen(catch, js_name = xtermMeasureNumber)]
    fn measure_number(value: &JsValue) -> Result<f64, JsValue>;
}
const MEASURE_REPEAT: usize = 32;
#[derive(Clone, Copy)]
enum FontProperty {
    Family,
    Size,
}
enum FontSource {
    Snapshot(RefCell<Font>),
    Live(Rc<OptionsService>),
}
impl FontSource {
    fn value(&self, property: FontProperty) -> Result<OptionValue, JsValue> {
        match self {
            Self::Snapshot(font) => {
                let font = font.borrow();
                Ok(match property {
                    FontProperty::Family => OptionValue::from(font.family.as_str()),
                    FontProperty::Size => OptionValue::Number(font.size),
                })
            }
            Self::Live(options) => options
                .get(match property {
                    FontProperty::Family => "fontFamily",
                    FontProperty::Size => "fontSize",
                })
                .map_err(|error| js_error::to_js_error(&error)),
        }
    }
    fn text(&self, property: FontProperty) -> Result<Vec<u16>, JsValue> {
        self.value(property)?
            .to_utf16()
            .map_err(|error| js_error::to_js_error(&error))
    }
    fn metrics_font(&self) -> Result<Vec<u16>, JsValue> {
        // Size conversion may mutate the family before its subsequent lookup.
        let mut text = self.text(FontProperty::Size)?;
        text.extend("px ".encode_utf16());
        text.extend(self.text(FontProperty::Family)?);
        Ok(text)
    }
}
enum Strategy {
    Metrics(OffscreenCanvasRenderingContext2d),
    Dom(HtmlElement),
}
fn measure_text(context: &JsValue) -> Result<JsValue, JsValue> {
    let method: js_sys::Function =
        js_sys::Reflect::get(context, &JsValue::from_str("measureText"))?
            .dyn_into()
            .map_err(|_| js_sys::TypeError::new("measureText is not callable"))?;
    method.call1(context, &JsValue::from_str("W"))
}
impl Strategy {
    fn metrics() -> Result<Self, JsValue> {
        let canvas = OffscreenCanvas::new(100, 100)?;
        let context: OffscreenCanvasRenderingContext2d = canvas
            .get_context("2d")?
            .ok_or_else(|| js_sys::Error::new("Canvas 2D context is unavailable"))?
            // The requested 2D context has this WebIDL interface in its realm.
            .unchecked_into();
        let metrics = measure_text(context.as_ref())?;
        for field in ["width", "fontBoundingBoxAscent", "fontBoundingBoxDescent"] {
            if !js_sys::Reflect::has(&metrics, &JsValue::from_str(field))? {
                return Err(js_sys::Error::new("Required font metrics not supported").into());
            }
        }
        Ok(Self::Metrics(context))
    }
    fn dom(document: &Document, parent: &HtmlElement) -> Result<Self, JsValue> {
        // HTML createElement('span') has HTMLElement methods across iframe realms.
        let element: HtmlElement = document.create_element("span")?.unchecked_into();
        element.set_class_name("xterm-char-measure-element");
        crate::utf16::set_text_content(
            element.as_ref(),
            crate::utf16::to_js_string(
                &"W".repeat(MEASURE_REPEAT)
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            ),
        )?;
        element.set_attribute("aria-hidden", "true")?;
        element.style().set_property("white-space", "pre")?;
        element.style().set_property("font-kerning", "none")?;
        parent.append_child(&element)?;
        Ok(Self::Dom(element))
    }
    fn measure(&self, font: &FontSource) -> Result<(Option<f64>, Option<f64>), JsValue> {
        match self {
            Self::Metrics(context) => {
                let value = crate::utf16::to_js_string(&font.metrics_font()?);
                crate::utf16::set_property(context.as_ref(), "font", value.as_ref())?;
                let metrics = measure_text(context.as_ref())?;
                let width = js_sys::Reflect::get(&metrics, &JsValue::from_str("width"))?.as_f64();
                let ascent =
                    js_sys::Reflect::get(&metrics, &JsValue::from_str("fontBoundingBoxAscent"))?
                        .as_f64();
                let descent =
                    js_sys::Reflect::get(&metrics, &JsValue::from_str("fontBoundingBoxDescent"))?
                        .as_f64();
                Ok((width, ascent.zip(descent).map(|(a, d)| a + d)))
            }
            Self::Dom(element) => {
                let style = js_sys::Reflect::get(element.as_ref(), &JsValue::from_str("style"))?;
                // Preserve the raw value: WebIDL conversion belongs to the chosen
                // setter, not an earlier Rust string projection.
                let family =
                    xterm_wasm_support::options::into_js(&font.value(FontProperty::Family)?)?;
                crate::utf16::set_property(&style, "fontFamily", &family)?;
                let style = js_sys::Reflect::get(element.as_ref(), &JsValue::from_str("style"))?;
                let mut size = font.text(FontProperty::Size)?;
                size.extend("px".encode_utf16());
                crate::utf16::set_property(
                    &style,
                    "fontSize",
                    crate::utf16::to_js_string(&size).as_ref(),
                )?;
                let width = measure_number(&js_sys::Reflect::get(
                    element.as_ref(),
                    &JsValue::from_str("offsetWidth"),
                )?)?;
                let height = measure_number(&js_sys::Reflect::get(
                    element.as_ref(),
                    &JsValue::from_str("offsetHeight"),
                )?)?;
                Ok((Some(width / MEASURE_REPEAT as f64), Some(height)))
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
    font: FontSource,
    option_listener: RefCell<Option<DisposableHandle>>,
}
impl CharSizeService {
    fn build(document: &Document, parent: &HtmlElement, font: FontSource) -> Result<Self, JsValue> {
        // Source deliberately falls back on any metric-construction failure.
        let strategy = match Strategy::metrics() {
            Ok(strategy) => strategy,
            Err(_) => Strategy::dom(document, parent)?,
        };
        Ok(Self {
            state: CharSizeState::default(),
            strategy,
            font,
            option_listener: RefCell::new(None),
        })
    }
    pub fn new(document: &Document, parent: &HtmlElement, font: Font) -> Result<Self, JsValue> {
        Self::build(document, parent, FontSource::Snapshot(RefCell::new(font)))
    }
    /// The browser path reads options at measurement time and registers the source
    /// family/size watcher. Construction probes metrics but does not measure yet.
    pub fn with_options(
        document: &Document,
        parent: &HtmlElement,
        options: Rc<OptionsService>,
    ) -> Result<Rc<Self>, JsValue> {
        let service = Rc::new(Self::build(
            document,
            parent,
            FontSource::Live(Rc::clone(&options)),
        )?);
        let weak = Rc::downgrade(&service);
        let listener = options.try_on_option_change(move |key| {
            if matches!(key, "fontFamily" | "fontSize") {
                if let Some(service) = weak.upgrade() {
                    service
                        .measure()
                        .map_err(|error| OptionError::Dispatch(js_error::callback_error(error)))?;
                }
            }
            Ok(())
        });
        *service.option_listener.borrow_mut() = Some(listener);
        Ok(service)
    }
    pub fn measure(&self) -> Result<(), JsValue> {
        let (width, height) = self.strategy.measure(&self.font)?;
        self.state
            .update_measurement(width, height)
            .map_err(|error| js_error::to_js_error(&error))
    }
    pub fn set_font(&self, font: Font) -> Result<(), JsValue> {
        match &self.font {
            FontSource::Snapshot(current) => *current.borrow_mut() = font,
            FontSource::Live(_) => {
                return Err(js_sys::TypeError::new(
                    "live font service is controlled by OptionsService",
                )
                .into())
            }
        }
        self.measure()
    }
}
impl Disposable for CharSizeService {
    fn dispose(&self) {
        let listener = self.option_listener.take();
        if let Some(listener) = listener {
            listener.dispose();
        }
        self.state.dispose();
    }
}
impl Drop for CharSizeService {
    fn drop(&mut self) {
        self.dispose();
    }
}
