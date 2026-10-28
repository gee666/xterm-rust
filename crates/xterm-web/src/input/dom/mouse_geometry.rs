// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use crate::input::mouse_coords::{MouseGeometry, ReportCoords};
use web_sys::Event;

fn nullish(value: &JsValue) -> bool {
    value.is_null() || value.is_undefined()
}

// Fallible structural counterpart of Dom.getWindow. The typed helper assumes
// native getters; this path must retain repeated reads and original exceptions.
fn window(element: &JsValue) -> Result<JsValue, JsValue> {
    if !nullish(element) {
        let document = values::get_js(element, "ownerDocument")?;
        if !nullish(&document) && values::get_js(&document, "defaultView")?.is_truthy() {
            let document = values::get_js(element, "ownerDocument")?;
            return values::get_js(&document, "defaultView");
        }
        if values::get_js(element, "view")?.is_truthy() {
            return values::get_js(element, "view");
        }
    }
    values::global_window()
}

fn padded_coordinate(
    event: &Event,
    field: &str,
    rect: &JsValue,
    edge: &str,
    padding: f64,
) -> Result<f64, JsValue> {
    // Both operands are obtained before ToNumeric calls can reenter or throw.
    let client = values::get(event, field)?;
    let origin = values::get_js(rect, edge)?;
    let difference = values::subtract(&client, &origin)?;
    let result = values::subtract(&difference, &JsValue::from_f64(padding))?;
    values::number_value(&result)
}

pub(super) fn report_coords(state: &State, event: &Event) -> Result<Option<ReportCoords>, JsValue> {
    let window = window(state.screen.as_ref())?;
    let rect = values::call(
        state.screen.as_ref(),
        "getBoundingClientRect",
        &js_sys::Array::new(),
    )?;
    let args = js_sys::Array::new();
    args.push(state.screen.as_ref());
    let style = values::call(&window, "getComputedStyle", &args)?;
    let padding = |name: &str| -> Result<f64, JsValue> {
        let args = js_sys::Array::new();
        args.push(&JsValue::from_str(name));
        values::padding(&values::call(&style, "getPropertyValue", &args)?)
    };
    let left = padding("padding-left")?;
    let top = padding("padding-top")?;
    let x = padded_coordinate(event, "clientX", &rect, "left", left)?;
    let y = padded_coordinate(event, "clientY", &rect, "top", top)?;
    if !state.host.mouse_valid_char_size()? {
        return Ok(None);
    }
    let [canvas_width, canvas_height] = state.host.mouse_canvas_size()?;
    let [cell_width, cell_height] = state.host.mouse_cell_size()?;
    Ok(MouseGeometry {
        rect_left: 0.0,
        rect_top: 0.0,
        padding_left: 0.0,
        padding_top: 0.0,
        valid_char_size: true,
        cell_width,
        cell_height,
        canvas_width,
        canvas_height,
    }
    .report_coords(x, y))
}
