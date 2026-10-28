#![cfg(target_arch = "wasm32")]
use js_sys::{Function, JsString, Object, Promise, Reflect};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{HtmlElement, MouseEvent};
use xterm_web::{utf16::from_js_string, Terminal};
wasm_bindgen_test_configure!(run_in_browser);
fn mouse(root: &HtmlElement, kind: &str, x: f64, y: f64) {
    let create = Function::new_with_args(
        "type,x,y",
        "return new MouseEvent(type,{clientX:x,clientY:y,bubbles:true});",
    );
    let event: MouseEvent = create
        .call3(&JsValue::UNDEFINED, &kind.into(), &x.into(), &y.into())
        .unwrap()
        .dyn_into()
        .unwrap();
    root.dispatch_event(&event).unwrap();
}
#[wasm_bindgen_test(async)]
async fn exported_terminal_writes_rust_osc8_renders_hovers_clicks_and_disposes() {
    let document = web_sys::window().unwrap().document().unwrap();
    let style = document.create_element("style").unwrap();
    // The same attributed production asset ships with the browser distribution.
    style.set_text_content(Some(include_str!("../assets/xterm.css")));
    document.body().unwrap().append_child(&style).unwrap();
    let parent: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
    parent.style().set_css_text("width:800px;height:160px;");
    document.body().unwrap().append_child(&parent).unwrap();
    let clicks = Rc::new(Cell::new(0));
    let clicked = clicks.clone();
    let ranges = Rc::new(RefCell::new(Vec::new()));
    let activated_ranges = ranges.clone();
    let hover_ranges = ranges.clone();
    let activate = Closure::wrap(Box::new(
        move |_: MouseEvent, text: JsString, range: JsValue| -> Result<(), JsValue> {
            assert_eq!(
                from_js_string(&text),
                "https://example.com/rust"
                    .encode_utf16()
                    .collect::<Vec<_>>()
            );
            clicked.set(clicked.get() + 1);
            activated_ranges.borrow_mut().push(range);
            Ok(())
        },
    )
        as Box<dyn FnMut(MouseEvent, JsString, JsValue) -> Result<(), JsValue>>);
    let hover = Closure::wrap(Box::new(
        move |_: MouseEvent, _: JsString, range: JsValue| -> Result<(), JsValue> {
            hover_ranges.borrow_mut().push(range);
            Ok(())
        },
    )
        as Box<dyn FnMut(MouseEvent, JsString, JsValue) -> Result<(), JsValue>>);
    let handler = Object::new();
    Reflect::set(&handler, &"activate".into(), activate.as_ref()).unwrap();
    Reflect::set(&handler, &"hover".into(), hover.as_ref()).unwrap();
    let options = Object::new();
    for (key, value) in [
        ("cols", JsValue::from_f64(10.0)),
        ("rows", 4.0.into()),
        ("linkHandler", handler.into()),
    ] {
        Reflect::set(&options, &key.into(), &value).unwrap();
    }
    let terminal = Terminal::new(options.into()).unwrap();
    terminal.open(parent.clone().into()).unwrap();
    let written = Promise::new(&mut |resolve, _| {
        terminal
            .write(
                JsString::from(
                    "\x1b]8;id=live;https://example.com/rust\x1b\\linked\x1b]8;;\x1b\\!",
                )
                .into(),
                Some(resolve),
            )
            .unwrap();
    });
    JsFuture::from(written).await.unwrap();
    let resolver: Rc<RefCell<Option<Function>>> = Rc::new(RefCell::new(None));
    let resolution = resolver.clone();
    let rendered = Promise::new(&mut |resolve, _| {
        *resolver.borrow_mut() = Some(resolve);
    });
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
    let unsubscribe: Function = Reflect::get(&subscription, &"dispose".into())
        .unwrap()
        .dyn_into()
        .unwrap();
    unsubscribe.call0(&subscription).unwrap();
    drop(observer);
    let root = terminal.element().unwrap();
    let dimensions = terminal.render_dimensions().unwrap();
    assert!(dimensions.css.cell.width > 0.0 && dimensions.css.cell.height > 0.0);
    assert!(root
        .query_selector(".xterm-rows")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap()
        .contains("linked!"));
    let rect = root.get_bounding_client_rect();
    let x = rect.left() + 2.5 * dimensions.css.cell.width;
    let y = rect.top() + 0.5 * dimensions.css.cell.height;
    mouse(&root, "mousemove", x, y);
    assert!(root.class_list().contains("xterm-cursor-pointer"));
    assert_eq!(
        terminal
            .linkifier_handle()
            .unwrap()
            .current_link()
            .unwrap()
            .text,
        "https://example.com/rust"
            .encode_utf16()
            .collect::<Vec<_>>()
    );
    mouse(&root, "mousedown", x, y);
    mouse(&root, "mouseup", x, y);
    assert_eq!(clicks.get(), 1);
    let ranges = ranges.borrow();
    assert_eq!(ranges.len(), 2);
    assert_eq!(
        ranges[0], ranges[1],
        "hover and activate must share the same JS range object"
    );
    let start = Reflect::get(&ranges[0], &"start".into()).unwrap();
    let end = Reflect::get(&ranges[0], &"end".into()).unwrap();
    assert_eq!(
        Reflect::get(&start, &"x".into()).unwrap().as_f64(),
        Some(1.0)
    );
    assert_eq!(Reflect::get(&end, &"x".into()).unwrap().as_f64(), Some(6.0));
    drop(ranges);
    terminal.dispose().unwrap();
    assert!(!root.is_connected());
    mouse(&root, "mousemove", x, y);
    mouse(&root, "mousedown", x, y);
    mouse(&root, "mouseup", x, y);
    assert_eq!(clicks.get(), 1);
    parent.remove();
    style.remove();
}
