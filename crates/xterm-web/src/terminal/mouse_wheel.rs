//! Raw wheel callbacks live on the actual JS service receiver, not a second
//! terminal-owned slot. Borrowed methods and accessor reentry observe that slot.
use js_sys::{Array, Reflect};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/terminal/mouse_wheel.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = installMouseWheelMethods)]
    fn install_methods(receiver: &JsValue, set: &JsValue, allow: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = readWheelHandler)]
    fn read_handler(receiver: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = invokeWheelHandler)]
    fn invoke_handler(receiver: &JsValue, event: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = writeWheelHandler)]
    fn set_handler(receiver: &JsValue, handler: &JsValue) -> Result<(), JsValue>;
}

pub(super) fn install(receiver: &JsValue) -> Result<(), JsValue> {
    // JS owns these stateless callbacks. They capture neither the terminal nor
    // its receiver, so retaining a method cannot form a Rust/JS ownership cycle.
    let set =
        Closure::<dyn Fn(JsValue, JsValue) -> Result<(), JsValue>>::new(|receiver, handler| {
            set_handler(&receiver, &handler)
        })
        .into_js_value();
    let allow =
        Closure::<dyn Fn(JsValue, JsValue) -> Result<bool, JsValue>>::new(|receiver, event| {
            allow_event(&receiver, &event)
        })
        .into_js_value();
    install_methods(receiver, &set, &allow)
}

fn allow_event(receiver: &JsValue, event: &JsValue) -> Result<bool, JsValue> {
    if !read_handler(receiver)?.is_truthy() {
        return Ok(true);
    }
    // Source tests the property then reads it again for the member call. Never
    // cache across an accessor; the second read may replace or throw the value.
    Ok(invoke_handler(receiver, event)?.as_bool() != Some(false))
}

pub(super) fn call_method(
    receiver: &JsValue,
    name: &str,
    argument: &JsValue,
) -> Result<JsValue, JsValue> {
    let method = Reflect::get(receiver, &name.into())?;
    Reflect::apply(method.unchecked_ref(), receiver, &Array::of1(argument))
}
