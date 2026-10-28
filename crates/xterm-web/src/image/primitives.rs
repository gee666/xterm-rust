// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
//! Actual browser primitives used by image storage and protocol hosts.
use super::js::{call, construct, function, get, nullish, set};
use js_sys::{Array, Promise, Reflect};
use wasm_bindgen::JsValue;

pub(super) fn int32(value: f64) -> i32 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    let wrapped = value.trunc().rem_euclid(4_294_967_296.0);
    if wrapped >= 2_147_483_648.0 {
        (wrapped - 4_294_967_296.0) as i32
    } else {
        wrapped as i32
    }
}

pub fn create_canvas(document: &JsValue, width: f64, height: f64) -> Result<JsValue, JsValue> {
    let document = if nullish(document) {
        let global = js_sys::global();
        if !Reflect::has(&global, &"document".into())? {
            return Err(js_sys::ReferenceError::new("document is not defined").into());
        }
        get(&global, "document")?
    } else {
        document.clone()
    };
    let canvas = call(&document, "createElement", &["canvas".into()])?;
    set(&canvas, "width", &int32(width).into())?;
    set(&canvas, "height", &int32(height).into())?;
    Ok(canvas)
}

pub fn create_image_data(
    context: &JsValue,
    width: f64,
    height: f64,
    buffer: &JsValue,
) -> Result<JsValue, JsValue> {
    let constructor = get(&js_sys::global(), "ImageData")?;
    let has_buffer = buffer.is_truthy();
    if !constructor.is_function() {
        let data = call(context, "createImageData", &[width.into(), height.into()])?;
        if has_buffer {
            let view = construct(
                "Uint8ClampedArray",
                &[buffer.clone(), 0.into(), (width * height * 4.0).into()],
            )?;
            call(&get(&data, "data")?, "set", &[view])?;
        }
        return Ok(data);
    }
    // Source reads the constructor again after typeof, before argument effects.
    let constructor = get(&js_sys::global(), "ImageData")?;
    let arguments = Array::new();
    if has_buffer {
        arguments.push(&construct(
            "Uint8ClampedArray",
            &[buffer.clone(), 0.into(), (width * height * 4.0).into()],
        )?);
    }
    arguments.push(&width.into());
    arguments.push(&height.into());
    Reflect::construct(&function(constructor, "ImageData")?, &arguments)
}

pub fn create_image_bitmap(image: &JsValue) -> Result<JsValue, JsValue> {
    let constructor = get(&js_sys::global(), "createImageBitmap")?;
    if !constructor.is_function() {
        return Ok(Promise::resolve(&JsValue::UNDEFINED).into());
    }
    function(
        get(&js_sys::global(), "createImageBitmap")?,
        "createImageBitmap",
    )?
    .call1(&JsValue::UNDEFINED, image)
}
