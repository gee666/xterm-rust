#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Function, Promise};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_core::headless;
use xterm_web::terminal::{link_services::create_registry, options};
wasm_bindgen_test_configure!(run_in_browser);

// This calls the same registry constructor as exported Terminal. The source
// probe calls the original Terminal's real first provider after parsing OSC8.
fn probe(handler: JsValue, uri: String, actions: Array) -> Result<JsValue, JsValue> {
    let native = Rc::new(headless::Terminal::new(80, 4, 0).unwrap());
    native
        .options()
        .set("linkHandler", options::from_js(&handler)?)
        .unwrap();
    native
        .write(&format!("\x1b]8;;{uri}\x1b\\linked\x1b]8;;\x1b\\!"))
        .unwrap();
    let registry = create_registry(Rc::clone(&native), &web_sys::window().unwrap())?;
    let result = Rc::new(RefCell::new(None));
    let output = result.clone();
    registry.get(0).unwrap().provide_links(
        1,
        Box::new(move |links| {
            *output.borrow_mut() = links;
            Ok(())
        }),
    )?;
    let links = result.borrow_mut().take().unwrap();
    let event = web_sys::MouseEvent::new("mousemove")?;
    let invoke = Function::new_with_args("link,action,event", "link[action]?.(event, link.text);");
    for link in &links {
        let xterm_web::links::browser_linkifier::BrowserLink::JavaScript(link) = link else {
            panic!("Browser registry must retain a live OSC payload")
        };
        for action in actions.iter() {
            invoke.call3(&JsValue::UNDEFINED, link, &action, event.as_ref())?;
        }
    }
    Ok(JsValue::from_f64(links.len() as f64))
}

#[wasm_bindgen_test(async)]
async fn primitive_handlers_preserve_nullish_lookup_receiver_and_exceptions() {
    let callback = Closure::wrap(
        Box::new(probe) as Box<dyn Fn(JsValue, String, Array) -> Result<JsValue, JsValue>>
    );
    let run = Function::new_with_args(
        "probe",
        include_str!("../../../scripts/osc-handler-trace.js"),
    );
    let pending: Promise = run
        .call1(&JsValue::UNDEFINED, callback.as_ref())
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
        serde_json::from_str(include_str!("fixtures/osc-handler.json")).unwrap();
    assert_eq!(actual, expected);
}
