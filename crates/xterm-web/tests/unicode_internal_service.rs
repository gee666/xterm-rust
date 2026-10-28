#![cfg(target_arch = "wasm32")]
use js_sys::{Function, JsString, Object, Reflect, JSON};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_web::{unicode_addons::Unicode11Addon, Terminal};

#[wasm_bindgen(module = "/tests/support/unicode_api.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = unicodeWidthTrace)]
    fn unicode_width_trace(api: &JsValue, service: &JsValue) -> Result<JsValue, JsValue>;
}
wasm_bindgen_test_configure!(run_in_browser);

fn get(value: &JsValue, key: &str) -> JsValue {
    Reflect::get(value, &key.into()).unwrap()
}
fn width(service: &JsValue, text: JsString) -> Result<JsValue, JsValue> {
    get(service, "getStringCellWidth")
        .dyn_into::<Function>()
        .unwrap()
        .call1(service, &text.into())
}

#[wasm_bindgen_test]
fn internal_service_is_stable_ungated_and_shares_public_addon_selection() {
    let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
    let first = terminal.core_services();
    assert!(Object::is(&first, &terminal.core_services()));
    let service = get(&first, "unicodeService");
    assert!(Object::is(&service, &get(&first, "unicodeService")));
    assert!(terminal.unicode().is_err(), "public view remains gated");
    let emoji = JsString::from("🤣🤣");
    assert_eq!(width(&service, emoji.clone()).unwrap(), JsValue::from(2));
    // V6 sees these newer emoji as narrow; V11 advances two cells each.
    // The selected service is retained even when no public view is available.
    Reflect::set(
        &terminal.options(),
        &"allowProposedApi".into(),
        &JsValue::TRUE,
    )
    .unwrap();
    Unicode11Addon::new().activate(&terminal).unwrap();
    terminal
        .unicode()
        .unwrap()
        .set_active_version("11".into())
        .unwrap();
    assert_eq!(width(&service, emoji).unwrap(), JsValue::from(4));
    terminal
        .unicode()
        .unwrap()
        .set_active_version("6".into())
        .unwrap();
    assert_eq!(width(&service, "🤣🤣".into()).unwrap(), JsValue::from(2));
    terminal.dispose().unwrap();
    assert_eq!(width(&service, "A".into()).unwrap(), JsValue::from(1));
}

#[wasm_bindgen_test]
fn internal_width_retains_utf16_callback_order_receiver_and_error_identity() {
    let terminal = Terminal::new(JSON::parse(r#"{"allowProposedApi":true}"#).unwrap()).unwrap();
    let service = get(&terminal.core_services(), "unicodeService");
    let api: JsValue = terminal.unicode().unwrap().into();
    let actual = unicode_width_trace(&api, &service).unwrap();
    let expected = JSON::parse(include_str!("fixtures/unicode-width-api.json")).unwrap();
    assert_eq!(
        JSON::stringify(&actual).unwrap(),
        JSON::stringify(&expected).unwrap()
    );
    terminal.dispose().unwrap();
}
