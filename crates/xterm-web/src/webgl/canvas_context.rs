// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
//! The real WebglRenderer canvas and context, created before any subscriptions.
use js_sys::{Function, Object, Reflect};
use std::cell::Cell;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{Document, HtmlCanvasElement, HtmlElement, WebGl2RenderingContext};

// Keep the source's string concatenation semantics for every falsy host return,
// including undefined, NaN and 0n. This is only a catching JS operator binding.
#[wasm_bindgen(
    inline_js = "export function unsupportedWebglContext(value) { return 'WebGL2 not supported ' + value; }"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = unsupportedWebglContext)]
    fn unsupported_context(value: &JsValue) -> Result<String, JsValue>;
}

pub struct CanvasContext {
    document: Document,
    canvas: HtmlCanvasElement,
    gl: WebGl2RenderingContext,
    detachment_attempted: Cell<bool>,
}
impl CanvasContext {
    /// Create the WebGL2 context first. A context failure must not leave renderer
    /// subscriptions, observers or DOM children behind. None preserves the
    /// source's explicit undefined preserveDrawingBuffer context attribute.
    pub fn new(
        document: &Document,
        preserve_drawing_buffer: Option<bool>,
    ) -> Result<Self, JsValue> {
        let canvas: HtmlCanvasElement = document.create_element("canvas")?.unchecked_into();
        let attributes = Object::new();
        attribute(&attributes, "antialias", &JsValue::FALSE)?;
        attribute(&attributes, "depth", &JsValue::FALSE)?;
        attribute(
            &attributes,
            "preserveDrawingBuffer",
            &preserve_drawing_buffer.map_or(JsValue::UNDEFINED, JsValue::from),
        )?;
        let gl = method(canvas.as_ref(), "getContext")?.call2(
            canvas.as_ref(),
            &"webgl2".into(),
            &attributes,
        )?;
        if !gl.is_truthy() {
            return Err(js_sys::Error::new(&unsupported_context(&gl)?).into());
        }
        let gl = gl.unchecked_into();
        Ok(Self {
            document: document.clone(),
            canvas,
            gl,
            detachment_attempted: Cell::new(false),
        })
    }
    pub fn main_document(&self) -> &Document {
        &self.document
    }
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }
    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }
    pub fn attach(&self, screen: &HtmlElement) -> Result<(), JsValue> {
        screen.append_child(&self.canvas)?;
        self.detachment_attempted.set(false);
        Ok(())
    }
    /// Physical observer dimensions are not the logical device.canvas size.
    /// Keep raw values until each native canvas setter performs its conversion.
    /// In particular height conversion must happen after the width mutation.
    pub fn set_device_pixel_dimensions(
        &self,
        width: &JsValue,
        height: &JsValue,
    ) -> Result<bool, JsValue> {
        let canvas = self.canvas.as_ref();
        if Reflect::get(canvas, &"width".into())? == *width
            && Reflect::get(canvas, &"height".into())? == *height
        {
            return Ok(false);
        }
        set(canvas, "width", width)?;
        set(canvas, "height", height)?;
        Ok(true)
    }
    pub fn detach(&self) -> Result<(), JsValue> {
        if self.detachment_attempted.replace(true) {
            return Ok(());
        }
        let parent = Reflect::get(self.canvas.as_ref(), &"parentElement".into())?;
        if !parent.is_null() && !parent.is_undefined() {
            method(&parent, "removeChild")?.call1(&parent, self.canvas.as_ref())?;
        }
        Ok(())
    }
}
impl Drop for CanvasContext {
    fn drop(&mut self) {
        if !self.detachment_attempted.get() {
            if let Err(error) = self.detach() {
                crate::write_scheduler::report_unhandled_error(&error);
            }
        }
    }
}
fn attribute(object: &Object, key: &str, value: &JsValue) -> Result<(), JsValue> {
    // A literal creates own data fields, unlike assignment through an inherited
    // setter. A null-prototype descriptor also avoids inherited descriptor fields.
    let descriptor = Object::create(JsValue::NULL.unchecked_ref());
    set(&descriptor, "value", value)?;
    for flag in ["writable", "enumerable", "configurable"] {
        set(&descriptor, flag, &JsValue::TRUE)?;
    }
    if !Reflect::define_property(object, &key.into(), &descriptor)? {
        return Err(js_sys::TypeError::new(&format!("Cannot define {key}")).into());
    }
    Ok(())
}
pub(super) fn set(receiver: &JsValue, key: &str, value: &JsValue) -> Result<(), JsValue> {
    if !Reflect::set(receiver, &key.into(), value)? {
        return Err(js_sys::TypeError::new(&format!("Cannot assign {key}")).into());
    }
    Ok(())
}
pub(super) fn method(receiver: &JsValue, name: &str) -> Result<Function, JsValue> {
    Reflect::get(receiver, &name.into())?
        .dyn_into()
        .map_err(|_| js_sys::TypeError::new(&format!("{name} is not a function")).into())
}
