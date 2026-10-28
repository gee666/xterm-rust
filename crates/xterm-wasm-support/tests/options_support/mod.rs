//! Test-only bridge to unchanged JS OptionsService and the native Rust service.
use js_sys::{Array, Function, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use xterm_core::headless::PublicOptions;
use xterm_wasm_support::options::{
    constructor_options, error_to_js, from_js, into_js, OptionsBinding,
};

#[wasm_bindgen(module = "/tests/reference/options-reference.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub fn referenceOptions(input: &JsValue) -> Result<JsValue, JsValue>;
}
#[wasm_bindgen(module = "/tests/reference/options-assertions.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn compareOptions(
        reference: &JsValue,
        reference_context: &JsValue,
        actual_context: &JsValue,
        get: &Function,
        set: &Function,
        layout: &Array,
        public_object: &JsValue,
        public_error: &JsValue,
        operations: &str,
        event_log: &Function,
        assign: &Function,
    ) -> Result<(), JsValue>;
}

pub fn run(factory: &str, operations: &str) {
    let factory = Function::new_no_args(factory);
    let expected = factory.call0(&JsValue::UNDEFINED).unwrap();
    let actual = factory.call0(&JsValue::UNDEFINED).unwrap();
    let input = |context: &JsValue| Reflect::get(context, &"input".into()).unwrap();
    let reference = referenceOptions(&input(&expected)).unwrap();
    let service = constructor_options(&input(&actual), false).unwrap();
    let layout = Array::new();
    for (key, enumerable) in service.property_layout() {
        layout.push(&Array::of2(&key.into(), &enumerable.into()));
    }
    let events = Rc::new(RefCell::new(Vec::new()));
    let output = events.clone();
    service
        .on_any_option_change()
        .subscribe(move |key| output.borrow_mut().push(key.clone()));
    let core = service.clone();
    let get = Closure::<dyn Fn(String) -> Result<JsValue, JsValue>>::new(move |key: String| {
        into_js(&core.get(&key).map_err(|e| error_to_js(&e))?)
    });
    let core = service.clone();
    let set = Closure::<dyn Fn(String, JsValue) -> Result<(), JsValue>>::new(
        move |key: String, value: JsValue| {
            core.set(&key, from_js(&value)?)
                .map_err(|e| error_to_js(&e))
        },
    );
    let event_log = Closure::<dyn Fn() -> Array>::new(move || {
        events
            .borrow()
            .iter()
            .map(|key| JsValue::from_str(key))
            .collect()
    });
    let (binding, view, error) = match OptionsBinding::new(PublicOptions::new(service.clone())) {
        Ok(binding) => {
            let binding = Rc::new(binding);
            (
                Some(binding.clone()),
                binding.object().into(),
                JsValue::UNDEFINED,
            )
        }
        Err(error) => (None, JsValue::UNDEFINED, error),
    };
    let assign =
        Closure::<dyn Fn(JsValue) -> Result<(), JsValue>>::new(
            move |input: JsValue| match &binding {
                Some(binding) => binding.assign(&input),
                None => Err(js_sys::TypeError::new("public options unavailable").into()),
            },
        );
    compareOptions(
        &reference,
        &expected,
        &actual,
        get.as_ref().unchecked_ref(),
        set.as_ref().unchecked_ref(),
        &layout,
        &view,
        &error,
        operations,
        event_log.as_ref().unchecked_ref(),
        assign.as_ref().unchecked_ref(),
    )
    .unwrap();
    service.try_dispose().unwrap();
}
