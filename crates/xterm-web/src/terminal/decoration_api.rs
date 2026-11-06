//! Public decoration handles use the same cache and render lifecycle as addons.
use super::{marker_api::BrowserMarker, Terminal};
use crate::js_error::{callback_error, to_js_error};
use js_sys::Function;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_core::{
    decoration::{Decoration, DecorationOptions},
    event::Event,
    options::{OptionError, OptionObject, OptionValue},
};

#[wasm_bindgen(module = "/src/terminal/decoration_bridge.js")]
extern "C" {
    #[wasm_bindgen(js_name = decorationRegistration)]
    fn registration() -> Function;
    #[wasm_bindgen(catch, js_name = pinDecoration)]
    fn pin(owner: &JsValue, record: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = decorationField)]
    fn field(weak: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = decorationFinish)]
    fn finish(weak: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = decorationListener)]
    fn listener(weak: &JsValue, value: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = decorateRecord)]
    fn decorate(
        record: &JsValue,
        dispose: JsValue,
        element: JsValue,
        set_element: JsValue,
        is_disposed: JsValue,
        render: JsValue,
        disposed: JsValue,
    ) -> Result<JsValue, JsValue>;
}
fn option_error(error: JsValue) -> OptionError {
    OptionError::Dispatch(callback_error(error))
}
fn source(weak: JsValue) -> OptionValue {
    OptionValue::Object(OptionObject::with_fallible_properties(
        weak,
        |_| Ok(OptionValue::from("[object Object]")),
        |weak, key| {
            field(weak, key)
                .and_then(|v| xterm_wasm_support::options::from_js(&v))
                .map_err(option_error)
        },
    ))
}
fn element(value: &Rc<dyn std::any::Any>) -> Result<JsValue, JsValue> {
    if let Some(value) = value.downcast_ref::<web_sys::HtmlElement>() {
        Ok(value.clone().into())
    } else if let Some(value) = value.downcast_ref::<JsValue>() {
        Ok(value.clone())
    } else {
        Err(js_sys::TypeError::new("Decoration element is not a browser value").into())
    }
}
fn event<T: 'static>(event: Event<T>, payload: fn(&T) -> Result<JsValue, JsValue>) -> JsValue {
    Closure::<dyn Fn(JsValue) -> Result<JsValue, JsValue>>::new(move |weak| {
        let result = event.register_with_context(move |value, _| {
            listener(&weak, &payload(value).map_err(callback_error)?).map_err(callback_error)
        });
        Ok(if result.collect {
            xterm_wasm_support::events::disposable(result.handle)
        } else {
            JsValue::UNDEFINED
        })
    })
    .into_js_value()
}
fn handle(record: &JsValue, decoration: Decoration) -> Result<JsValue, JsValue> {
    let value = decoration.clone();
    let dispose = Closure::<dyn Fn() -> Result<(), JsValue>>::new(move || {
        value.dispose().map_err(|e| to_js_error(&e))
    })
    .into_js_value();
    let value = decoration.clone();
    let get = Closure::<dyn Fn() -> Result<JsValue, JsValue>>::new(move || {
        value
            .element()
            .as_ref()
            .map(element)
            .transpose()
            .map(|v| v.unwrap_or(JsValue::UNDEFINED))
    })
    .into_js_value();
    let value = decoration.clone();
    let set = Closure::<dyn Fn(JsValue)>::new(move |element: JsValue| {
        value.set_element(if element.is_undefined() {
            None
        } else {
            Some(Rc::new(element))
        });
    })
    .into_js_value();
    let value = decoration.clone();
    let disposed = Closure::<dyn Fn() -> bool>::new(move || value.is_disposed()).into_js_value();
    decorate(
        record,
        dispose,
        get,
        set,
        disposed,
        event(decoration.on_render(), element),
        event(decoration.on_dispose(), super::events::empty),
    )
}

#[wasm_bindgen]
impl Terminal {
    /// The JS method keeps its actual receiver as the GC owner. A native service
    /// never strongly owns the options/listeners that can refer back to Terminal.
    #[wasm_bindgen(getter, js_name = registerDecoration)]
    pub fn decoration_registration(&self) -> Function {
        registration()
    }

    #[wasm_bindgen(js_name = _registerDecoration)]
    pub fn register_decoration_record(
        &self,
        record: JsValue,
        marker: &BrowserMarker,
        owner: JsValue,
    ) -> Result<JsValue, JsValue> {
        let weak = pin(&owner, &record)?;
        let options = Rc::new(RefCell::new(DecorationOptions::new(marker.native())));
        let initialize = |decoration: &Decoration| {
            handle(&record, decoration.clone()).map_err(callback_error)?;
            let weak = weak.clone();
            decoration.set_host_cleanup(Rc::new(move || finish(&weak).map_err(callback_error)))
        };
        self.shared
            .decorations
            .register_decoration_source(options, source(weak.clone()), Some(&initialize))
            .map_err(|e| to_js_error(&e))?;
        Ok(record)
    }
}
