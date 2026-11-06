#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::HtmlElement;
use xterm_web::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn image_core_views_and_streaming_handlers_use_the_real_terminal() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let parent: HtmlElement = document.create_element("div")?.dyn_into()?;
    document.body().unwrap().append_child(&parent)?;
    let options = Object::new();
    for (key, value) in [
        ("cols", 10.into()),
        ("rows", 4.into()),
        ("allowProposedApi", true.into()),
    ] {
        Reflect::set(&options, &key.into(), &value)?;
    }
    let terminal = Terminal::new(options.into())?;
    let core = terminal.core_services();
    assert!(Reflect::get(&core, &"_themeService".into())?.is_undefined());
    assert!(Reflect::get(&core, &"_coreBrowserService".into())?.is_undefined());
    terminal.open(parent.clone().into())?;
    let terminal: JsValue = terminal.into();
    let result = async {
        let run = Function::new_with_args(
            "terminal",
            include_str!("../../../scripts/image-core-trace.js"),
        );
        let pending: Promise = run.call1(&JsValue::UNDEFINED, &terminal)?.dyn_into()?;
        JsFuture::from(pending).await
    }
    .await;
    let dispose: Function = Reflect::get(&terminal, &"dispose".into())?.dyn_into()?;
    dispose.call0(&terminal)?;
    parent.remove();
    let actual: serde_json::Value =
        serde_json::from_str(&js_sys::JSON::stringify(&result?)?.as_string().unwrap()).unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/image-core.json")).unwrap();
    assert_eq!(actual, expected);
    Ok(())
}
