#![cfg(target_arch = "wasm32")]

use js_sys::{Function, Object, Reflect, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::{enable_ligatures, LigaturesAddon};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/support/ligature_scenarios.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = lifecycleScenarios)]
    fn lifecycle_scenarios(create: &Function) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = loadingScenarios)]
    async fn loading_scenarios(create: &Function, bytes: &Uint8Array) -> Result<JsValue, JsValue>;
}

fn factory() -> Closure<dyn FnMut(JsValue) -> Result<JsValue, JsValue>> {
    Closure::wrap(
        Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
            as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
    )
}

fn assert_source(section: &str, actual: &JsValue) {
    let expected = js_sys::JSON::parse(include_str!("fixtures/ligature-addon.json")).unwrap();
    let expected = Reflect::get(&expected, &section.into()).unwrap();
    // Compare each operation separately so failures retain a useful operation key.
    for key in Object::keys(&Object::from(expected.clone())) {
        let expected = Reflect::get(&expected, &key).unwrap();
        let actual = Reflect::get(actual, &key).unwrap();
        assert_eq!(
            js_sys::JSON::stringify(&actual).unwrap(),
            js_sys::JSON::stringify(&expected).unwrap(),
            "{section}, {}",
            key.as_string().unwrap()
        );
    }
}

#[wasm_bindgen_test]
fn lifecycle_ranges_style_errors_and_reactivation_match_source() {
    let create = factory();
    let actual = lifecycle_scenarios(create.as_ref().unchecked_ref()).unwrap();
    assert_source("lifecycle", &actual);
}

#[wasm_bindgen_test(async)]
async fn font_load_fallback_staleness_clone_refresh_and_disposal_match_source() {
    let create = factory();
    let bytes = Uint8Array::from(
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/FiraCode-Regular.otf")
            .as_slice(),
    );
    let actual = loading_scenarios(create.as_ref().unchecked_ref(), &bytes)
        .await
        .unwrap();
    assert_source("loading", &actual);
}

#[wasm_bindgen_test]
fn direct_enable_registers_without_open_and_defaults_to_no_fallback() {
    // The upstream suite repeats the registration assertion twice. Keep both
    // registrations, including the absence check before calling the helper.
    for _ in 0..2 {
        let term = Function::new_no_args("return {options:{fontFamily:''},registerCharacterJoiner(callback){this.joiner=callback;return 1}}")
            .call0(&JsValue::UNDEFINED).unwrap();
        assert!(Reflect::get(&term, &"joiner".into())
            .unwrap()
            .is_undefined());
        let id = enable_ligatures(term.clone(), JsValue::UNDEFINED).unwrap();
        assert_eq!(id, 1.0);
        let callback = Reflect::get(&term, &"joiner".into()).unwrap();
        assert!(callback.is_function());
        let result = callback
            .unchecked_into::<Function>()
            .call1(&term, &"a -> b www c".into())
            .unwrap();
        assert_eq!(js_sys::Array::from(&result).length(), 0);
    }
}

#[wasm_bindgen_test]
fn actual_css_style_is_set_and_cleared() {
    let term = Function::new_no_args("return {element:document.createElement('div'),registerCharacterJoiner(){return 0},deregisterCharacterJoiner(){}}")
        .call0(&JsValue::UNDEFINED).unwrap();
    let element = Reflect::get(&term, &"element".into()).unwrap();
    let style = Reflect::get(&element, &"style".into()).unwrap();
    let addon = LigaturesAddon::new(JsValue::UNDEFINED).unwrap();
    addon.activate(term).unwrap();
    assert_eq!(
        Reflect::get(&style, &"fontFeatureSettings".into()).unwrap(),
        "\"calt\""
    );
    addon.dispose().unwrap();
    assert_eq!(
        Reflect::get(&style, &"fontFeatureSettings".into()).unwrap(),
        ""
    );
}
