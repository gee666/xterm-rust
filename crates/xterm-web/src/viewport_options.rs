// Copyright (c) 2024 The xterm.js authors. All rights reserved. Licensed under MIT.
use super::SmoothScrollableElement;
use crate::{
    js_error::to_js_error,
    terminal::options::{into_js, set_data},
};
use js_sys::Object;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use xterm_core::options::{OptionValue as V, OptionsService};

const DEFAULT_SCROLL_BAR_WIDTH: f64 = 14.0;
fn nullish(value: V, default: V) -> V {
    if matches!(value, V::Undefined | V::Null) {
        default
    } else {
        value
    }
}
fn scrollbar(options: &OptionsService, key: &str) -> Result<V, JsValue> {
    let value = options
        .get("scrollbar")
        .map_err(|error| to_js_error(&error))?;
    if matches!(value, V::Undefined | V::Null) {
        Ok(V::Undefined)
    } else {
        value.get_property(key).map_err(|error| to_js_error(&error))
    }
}
fn option(options: &OptionsService, key: &str) -> Result<JsValue, JsValue> {
    into_js(&options.get(key).map_err(|error| to_js_error(&error))?)
}

pub(super) fn changes(options: &OptionsService) -> Result<JsValue, JsValue> {
    let show = nullish(scrollbar(options, "showScrollbar")?, V::Bool(true));
    let arrows = nullish(scrollbar(options, "showArrows")?, V::Bool(false));
    let width = if show.is_truthy() {
        nullish(
            scrollbar(options, "width")?,
            V::Number(DEFAULT_SCROLL_BAR_WIDTH),
        )
    } else {
        V::Number(0.0)
    };
    let object = Object::new();
    set_data(
        &object,
        "mouseWheelScrollSensitivity",
        &option(options, "scrollSensitivity")?,
    )?;
    set_data(
        &object,
        "fastScrollSensitivity",
        &option(options, "fastScrollSensitivity")?,
    )?;
    set_data(
        &object,
        "vertical",
        &JsValue::from_f64(if show.is_truthy() { 1.0 } else { 2.0 }),
    )?;
    set_data(&object, "verticalScrollbarSize", &into_js(&width)?)?;
    set_data(&object, "verticalHasArrows", &into_js(&arrows)?)?;
    Ok(object.into())
}

pub(super) fn initial(options: &OptionsService) -> Result<JsValue, JsValue> {
    let object = Object::new();
    for (key, value) in [
        ("vertical", JsValue::from_f64(1.0)),
        ("horizontal", JsValue::from_f64(2.0)),
        ("useShadows", JsValue::FALSE),
        ("mouseWheelSmoothScroll", JsValue::TRUE),
    ] {
        set_data(&object, key, &value)?;
    }
    // This first read remains observable even though the spread below replaces
    // the property with its later read, matching the source object literal.
    let arrows = nullish(scrollbar(options, "showArrows")?, V::Bool(false));
    set_data(&object, "verticalHasArrows", &into_js(&arrows)?)?;
    let changed = changes(options)?;
    for key in [
        "mouseWheelScrollSensitivity",
        "fastScrollSensitivity",
        "vertical",
        "verticalScrollbarSize",
        "verticalHasArrows",
    ] {
        let value = js_sys::Reflect::get(&changed, &key.into())?;
        set_data(&object, key, &value)?;
    }
    Ok(object.into())
}

pub(super) fn update(
    element: &Rc<SmoothScrollableElement>,
    options: &OptionsService,
) -> Result<(), JsValue> {
    element.update_options(&changes(options)?)
}
