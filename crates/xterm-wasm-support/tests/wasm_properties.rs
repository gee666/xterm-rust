#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Reflect};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_core::options::OptionValue as V;
use xterm_wasm_support::options::{error_to_js, from_js, into_js};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn native_property_operations_use_the_original_host_object_and_receiver() {
    let context=Function::new_no_args(r#"
const marker={tag:'setter'};let stored=1;
const object={extra:3,get flag(){if(this!==object)throw Error('getter receiver');return stored;},set flag(value){if(this!==object)throw Error('setter receiver');stored=value;},set bad(value){throw marker;}};
return {object,marker};
"#).call0(&JsValue::UNDEFINED).unwrap();
    let object = Reflect::get(&context, &"object".into()).unwrap();
    let value = from_js(&object).unwrap();
    assert_eq!(value.get_property("flag").unwrap(), V::Number(1.0));
    value.set_property("flag", V::Number(2.0)).unwrap();
    assert_eq!(
        Reflect::get(&object, &"flag".into()).unwrap(),
        JsValue::from(2)
    );
    assert!(value.delete_property("extra").unwrap());
    assert!(!Reflect::has(&object, &"extra".into()).unwrap());
    let V::Object(handle) = &value else { panic!() };
    assert_eq!(handle.property_keys().unwrap(), ["flag", "bad"]);
    let failure = value.set_property("bad", V::Bool(true)).unwrap_err();
    assert_eq!(
        error_to_js(&failure),
        Reflect::get(&context, &"marker".into()).unwrap()
    );
    // The native acyclic-data policy must not be imposed on a real JS graph.
    value.set_property("self", value.clone()).unwrap();
    assert_eq!(Reflect::get(&object, &"self".into()).unwrap(), object);
    assert_eq!(
        into_js(&value.get_property("self").unwrap()).unwrap(),
        object
    );
    value.delete_property("self").unwrap();
}
