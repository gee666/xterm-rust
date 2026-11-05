// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Catching Canvas2D calls preserve thrown values and UTF-16 text at the host boundary.
use crate::utf16::{set_property, to_js_string};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Path2d};
#[wasm_bindgen::prelude::wasm_bindgen(module = "/src/webgl/atlas_baseline.js")]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = atlasTextBaseline)]
    pub(super) fn text_baseline() -> JsValue;
}

pub(super) struct Context<'a>(pub &'a CanvasRenderingContext2d);
impl Context<'_> {
    pub fn get(&self, name: &str) -> Result<JsValue, JsValue> {
        js_sys::Reflect::get(self.0.as_ref(), &name.into())
    }
    pub fn set(&self, name: &str, value: &JsValue) -> Result<(), JsValue> {
        set_property(self.0.as_ref(), name, value)
    }
    pub fn string(&self, name: &str, units: &[u16]) -> Result<(), JsValue> {
        self.set(name, to_js_string(units).as_ref())
    }
    pub fn method(&self, name: &str) -> Result<js_sys::Function, JsValue> {
        self.get(name)?
            .dyn_into::<js_sys::Function>()
            .map_err(|_| js_sys::TypeError::new(&format!("{name} is not a function")).into())
    }
    pub fn call(&self, name: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
        self.apply(&self.method(name)?, args)
    }
    pub fn apply(&self, function: &js_sys::Function, args: &[JsValue]) -> Result<JsValue, JsValue> {
        let array = js_sys::Array::new();
        for value in args {
            array.push(value);
        }
        function.apply(self.0.as_ref(), &array)
    }
    pub fn op(&self, name: &str, args: &[f64]) -> Result<(), JsValue> {
        self.call(
            name,
            &args.iter().copied().map(JsValue::from).collect::<Vec<_>>(),
        )?;
        Ok(())
    }
    pub fn text(&self, name: &str, units: &[u16], x: f64, y: f64) -> Result<(), JsValue> {
        self.call(name, &[to_js_string(units).into(), x.into(), y.into()])?;
        Ok(())
    }
    pub fn clip_rect(&self, rect: [f64; 4]) -> Result<(), JsValue> {
        let path = Path2d::new()?;
        let method = js_sys::Reflect::get(path.as_ref(), &"rect".into())?;
        let function = method
            .dyn_ref::<js_sys::Function>()
            .ok_or_else(|| js_sys::TypeError::new("rect is not a function"))?;
        let array = js_sys::Array::new();
        for value in rect {
            array.push(&value.into());
        }
        function.apply(path.as_ref(), &array)?;
        self.call("clip", &[path.into()])?;
        Ok(())
    }
}
