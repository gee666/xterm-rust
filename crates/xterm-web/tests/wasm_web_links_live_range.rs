#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::{terminal::Terminal, utility_addons::link_bindings::WebLinksAddon};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn utility_callback_ranges_drive_live_pointer_equality_and_selection() {
    let options = Object::new();
    Reflect::set(&options, &"cols".into(), &20.into()).unwrap();
    Reflect::set(&options, &"rows".into(), &4.into()).unwrap();
    let terminal: JsValue = Terminal::new(options.into()).unwrap().into();
    let create = Closure::<dyn Fn(JsValue, JsValue) -> JsValue>::new(|handler, options| {
        WebLinksAddon::new(handler, options).into()
    });
    let create_terminal = Closure::<dyn Fn() -> Result<JsValue, JsValue>>::new(|| {
        let options = Object::new();
        Reflect::set(&options, &"cols".into(), &20.into())?;
        Reflect::set(&options, &"rows".into(), &4.into())?;
        Ok(Terminal::new(options.into())?.into())
    });
    let program = Function::new_with_args(
        "terminal,createAddon,createTerminal",
        include_str!("../../../scripts/web-links-live-range-trace.js"),
    );
    let pending: Promise = program
        .call3(
            &JsValue::UNDEFINED,
            &terminal,
            create.as_ref(),
            create_terminal.as_ref(),
        )
        .unwrap()
        .dyn_into()
        .unwrap();
    let result = JsFuture::from(pending).await.unwrap();
    let actual: serde_json::Value = serde_json::from_str(
        &js_sys::JSON::stringify(&result)
            .unwrap()
            .as_string()
            .unwrap(),
    )
    .unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/web-links-live-range.json")).unwrap();
    assert_eq!(actual, expected);
}
