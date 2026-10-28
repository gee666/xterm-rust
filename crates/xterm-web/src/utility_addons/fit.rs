// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. FitAddon DOM measurement adapter.
use crate::api::RenderDimensions;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::{CssStyleDeclaration, Element, HtmlElement};
use xterm_addons::fit::{Dimensions, FitError, FitMeasurements, FitTerminal};

#[derive(Clone, Copy, Debug)]
pub struct ScrollbarOptions {
    pub scrollback: f64,
    pub show: Option<bool>,
    pub width: Option<f64>,
}

pub struct BrowserFitTerminal {
    pub element: Rc<dyn Fn() -> Option<HtmlElement>>,
    pub dimensions: Rc<dyn Fn() -> Option<RenderDimensions>>,
    pub scrollbar: Rc<dyn Fn() -> Result<ScrollbarOptions, FitError>>,
    pub resize: Rc<dyn Fn(Dimensions) -> Result<(), FitError>>,
}
fn error(value: JsValue) -> FitError {
    FitError::host(crate::js_error::JsCallbackError(value))
}
fn style(element: &Element) -> Result<CssStyleDeclaration, FitError> {
    let window = element
        .owner_document()
        .and_then(|document| document.default_view())
        .or_else(web_sys::window)
        .ok_or_else(|| FitError::Message("no window for fit measurement".into()))?;
    window
        .get_computed_style(element)
        .map_err(error)?
        .ok_or_else(|| FitError::Message("getComputedStyle returned no style".into()))
}
impl FitTerminal for BrowserFitTerminal {
    fn measurements(&self) -> Result<Option<FitMeasurements>, FitError> {
        let Some(element) = (self.element)() else {
            return Ok(None);
        };
        let Some(parent) = element.parent_element() else {
            return Ok(None);
        };
        let Some(dimensions) = (self.dimensions)() else {
            return Ok(None);
        };
        if dimensions.css.cell.width == 0.0 || dimensions.css.cell.height == 0.0 {
            return Ok(None);
        }
        let scrollbar = (self.scrollbar)()?;
        let parent = style(&parent)?;
        let parent_height = parent.get_property_value("height").map_err(error)?;
        let parent_width = parent.get_property_value("width").map_err(error)?;
        let element = style(&element)?;
        Ok(Some(FitMeasurements {
            cell_width: dimensions.css.cell.width,
            cell_height: dimensions.css.cell.height,
            parent_width,
            parent_height,
            padding: [
                element.get_property_value("padding-top").map_err(error)?,
                element
                    .get_property_value("padding-bottom")
                    .map_err(error)?,
                element.get_property_value("padding-right").map_err(error)?,
                element.get_property_value("padding-left").map_err(error)?,
            ],
            scrollback: scrollbar.scrollback,
            show_scrollbar: scrollbar.show,
            scrollbar_width: scrollbar.width,
        }))
    }
    fn resize(&self, dimensions: Dimensions) -> Result<(), FitError> {
        (self.resize)(dimensions)
    }
}
