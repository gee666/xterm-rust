#![cfg(target_arch = "wasm32")]
use js_sys::{Function, JsString, Object, Promise, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::HtmlElement;
use xterm_web::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

// Keep this regression independent of the existing Terminal link scenarios so
// an unchanged-production baseline can run exactly the same assertion program.
async fn render(terminal: &Terminal) {
    let resolver: Rc<RefCell<Option<Function>>> = Rc::default();
    let resolution = resolver.clone();
    let rendered = Promise::new(&mut |resolve, _| *resolver.borrow_mut() = Some(resolve));
    let observer = Closure::wrap(Box::new(move |_: JsValue| -> Result<(), JsValue> {
        if let Some(resolve) = resolution.borrow_mut().take() {
            resolve.call0(&JsValue::UNDEFINED)?;
        }
        Ok(())
    }) as Box<dyn FnMut(JsValue) -> Result<(), JsValue>>);
    let subscription = terminal
        .on_render()
        .call1(&JsValue::UNDEFINED, observer.as_ref())
        .unwrap();
    terminal.refresh(0.0, 3.0).unwrap();
    JsFuture::from(rendered).await.unwrap();
    method(&subscription, "dispose")
        .call0(&subscription)
        .unwrap();
}
fn method(object: &JsValue, name: &str) -> Function {
    Reflect::get(object, &name.into())
        .unwrap()
        .dyn_into()
        .unwrap()
}
#[wasm_bindgen_test(async)]
async fn screen_events_pointer_padding_and_detached_disposal_match_original() {
    let document = web_sys::window().unwrap().document().unwrap();
    let style = document.create_element("style").unwrap();
    style.set_text_content(Some(include_str!("../assets/xterm.css")));
    document.body().unwrap().append_child(&style).unwrap();
    let parent: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
    let options = Object::new();
    for (name, value) in [("cols", 20.0), ("rows", 4.0)] {
        Reflect::set(&options, &name.into(), &value.into()).unwrap();
    }
    let terminal = Terminal::new(options.into()).unwrap();
    terminal.open(parent.clone().into()).unwrap();
    let written = Promise::new(&mut |resolve, _| {
        terminal
            .write(
                JsString::from(
                    "\x1b]8;;\x1b\\.....\x1b]8;;https://example.com/screen\x1b\\L\x1b]8;;\x1b\\!",
                )
                .into(),
                Some(resolve),
            )
            .unwrap();
    });
    JsFuture::from(written).await.unwrap();
    render(&terminal).await;
    let root = terminal.element().unwrap();
    let cell = terminal.render_dimensions().unwrap().css.cell;
    assert!(cell.width > 0.0 && cell.height > 0.0);
    let probe = Function::new_with_args(
        "root,width,height",
        include_str!("../../../scripts/link-screen-trace.js"),
    )
    .call3(
        &JsValue::UNDEFINED,
        root.as_ref(),
        &cell.width.into(),
        &cell.height.into(),
    )
    .unwrap();
    Reflect::set(
        &terminal.options(),
        &"linkHandler".into(),
        &Reflect::get(&probe, &"handler".into()).unwrap(),
    )
    .unwrap();
    let snapshots = method(&probe, "run").call0(&probe).unwrap();
    terminal.dispose().unwrap();
    assert!(!root.is_connected());
    let after_dispose = method(&probe, "afterDispose").call0(&probe).unwrap();
    parent.remove();
    style.remove();
    let result = Object::new();
    Reflect::set(&result, &"snapshots".into(), &snapshots).unwrap();
    Reflect::set(&result, &"afterDispose".into(), &after_dispose).unwrap();
    let actual: serde_json::Value = serde_json::from_str(
        &js_sys::JSON::stringify(&result)
            .unwrap()
            .as_string()
            .unwrap(),
    )
    .unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/link-screen.json")).unwrap();
    assert_eq!(actual, expected);
}
