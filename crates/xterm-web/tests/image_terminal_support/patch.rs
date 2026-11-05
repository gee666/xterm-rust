// Copyright (c) 2023 The xterm.js authors. MIT license.
use super::get;
use wasm_bindgen::{JsCast, JsValue};
pub struct Patch {
    object: JsValue,
    key: &'static str,
    previous: JsValue,
}
impl Patch {
    pub fn new(object: JsValue, key: &'static str, value: JsValue) -> Self {
        let previous = get(&object, key);
        js_sys::Reflect::set(&object, &key.into(), &value).unwrap();
        Self {
            object,
            key,
            previous,
        }
    }
}
impl Drop for Patch {
    fn drop(&mut self) {
        js_sys::Reflect::set(&self.object, &self.key.into(), &self.previous).unwrap();
    }
}
pub fn try_call(value: &JsValue, key: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
    get(value, key)
        .unchecked_into::<js_sys::Function>()
        .apply(value, &args.iter().cloned().collect::<js_sys::Array>())
}
pub fn thrower(token: &JsValue) -> JsValue {
    js_sys::Function::new_with_args("token", "return function(){throw token}")
        .call1(&JsValue::UNDEFINED, token)
        .unwrap()
}
