#![cfg(target_arch = "wasm32")]
//! Real Terminal regression for source-timed recursive selection input.
use js_sys::{Function, JsString, Object, Promise, Reflect};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{HtmlElement, MouseEvent};
use xterm_web::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

fn mouse(root: &HtmlElement, kind: &str, x: f64, y: f64) -> MouseEvent {
    let create = Function::new_with_args(
        "kind,x,y",
        "return new MouseEvent(kind,{clientX:x,clientY:y,detail:1,bubbles:true,cancelable:true});",
    );
    let event: MouseEvent = create
        .call3(&JsValue::UNDEFINED, &kind.into(), &x.into(), &y.into())
        .unwrap()
        .dyn_into()
        .unwrap();
    root.dispatch_event(&event).unwrap();
    event
}

#[wasm_bindgen_test(async)]
async fn selection_change_can_dispatch_nested_mousedown_then_drag() {
    let document = web_sys::window().unwrap().document().unwrap();
    let parent: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
    parent.style().set_css_text("width:800px;height:160px");
    document.body().unwrap().append_child(&parent).unwrap();
    let options = Object::new();
    Reflect::set(&options, &"cols".into(), &10.0.into()).unwrap();
    Reflect::set(&options, &"rows".into(), &4.0.into()).unwrap();
    let terminal = Rc::new(Terminal::new(options.into()).unwrap());
    terminal.open(parent.clone().into()).unwrap();
    let written = Promise::new(&mut |resolve, _| {
        terminal
            .write(JsString::from("abcdefghij").into(), Some(resolve))
            .unwrap();
    });
    JsFuture::from(written).await.unwrap();
    let root = terminal.element().unwrap();
    let dimensions = terminal.render_dimensions().unwrap();
    let rect = root.get_bounding_client_rect();
    let x = |column: f64| rect.left() + (column + 0.1) * dimensions.css.cell.width;
    let y = rect.top() + 0.5 * dimensions.css.cell.height;
    terminal.select_all().unwrap();
    let changes = Rc::new(Cell::new(0));
    let nested_prevented = Rc::new(Cell::new(false));
    let weak = Rc::downgrade(&terminal);
    let nested_root = root.clone();
    let nested_changes = changes.clone();
    let prevented = nested_prevented.clone();
    let nested_x = x(4.0);
    let observer = Closure::wrap(Box::new(move |_: JsValue| -> Result<(), JsValue> {
        let count = nested_changes.get() + 1;
        nested_changes.set(count);
        if count == 1 {
            let terminal = weak.upgrade().unwrap();
            terminal.select_all()?;
            prevented.set(mouse(&nested_root, "mousedown", nested_x, y).default_prevented());
        }
        Ok(())
    }) as Box<dyn Fn(JsValue) -> Result<(), JsValue>>);
    let subscription = terminal
        .on_selection_change()
        .call1(&JsValue::UNDEFINED, observer.as_ref())
        .unwrap();
    mouse(&root, "mousedown", x(1.0), y);
    mouse(&root, "mousemove", x(7.0), y);
    mouse(&root, "mouseup", x(7.0), y);
    assert!(
        nested_prevented.get(),
        "nested mousedown must reach selection"
    );
    assert_eq!(terminal.get_selection().unwrap(), JsString::from("efg"));
    assert_eq!(changes.get(), 4);
    let dispose: Function = Reflect::get(&subscription, &"dispose".into())
        .unwrap()
        .dyn_into()
        .unwrap();
    dispose.call0(&subscription).unwrap();
    terminal.dispose().unwrap();
    parent.remove();
}
