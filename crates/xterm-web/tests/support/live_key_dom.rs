use super::support::event;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, Event, HtmlElement, Node};
use xterm_web::dom_listener::DomListener;

pub fn define(event: &Event, name: &str, field: &str, value: &JsValue) {
    let descriptor = js_sys::Object::new();
    js_sys::Reflect::set(&descriptor, &field.into(), value).unwrap();
    js_sys::Reflect::set(&descriptor, &"configurable".into(), &true.into()).unwrap();
    assert!(
        js_sys::Reflect::define_property(event.unchecked_ref(), &name.into(), &descriptor).unwrap()
    );
}
pub struct Errors {
    listener: DomListener,
    pub values: Rc<RefCell<Vec<JsValue>>>,
}
impl Errors {
    pub fn new() -> Self {
        let values = Rc::new(RefCell::new(Vec::new()));
        let output = values.clone();
        let window = web_sys::window().unwrap();
        let listener = DomListener::new_reentrant(window.as_ref(), "error", true, move |event| {
            let error = js_sys::Reflect::get(event.as_ref(), &"error".into()).unwrap();
            output.borrow_mut().push(error);
            event.prevent_default();
            event.stop_immediate_propagation();
        })
        .unwrap();
        Self { listener, values }
    }
    pub fn empty(&self) {
        assert!(
            self.values.borrow().is_empty(),
            "unexpected listener exception"
        );
    }
}
impl Drop for Errors {
    fn drop(&mut self) {
        self.listener.dispose().unwrap();
    }
}
pub struct ForeignDocument {
    frame: Element,
    root: HtmlElement,
    previous: Node,
}
impl ForeignDocument {
    pub fn keyboard(root: &HtmlElement) -> (Self, Event) {
        let document = root.owner_document().unwrap();
        let frame = document.create_element("iframe").unwrap();
        document.body().unwrap().append_child(&frame).unwrap();
        let child = js_sys::Reflect::get(frame.as_ref(), &"contentDocument".into()).unwrap();
        let body = js_sys::Reflect::get(&child, &"body".into()).unwrap();
        let previous = root.parent_node().unwrap();
        let body: Node = body.unchecked_into();
        body.append_child(root).unwrap();
        let window = js_sys::Reflect::get(frame.as_ref(), &"contentWindow".into()).unwrap();
        let constructor = js_sys::Reflect::get(&window, &"KeyboardEvent".into()).unwrap();
        assert!(constructor.is_function());
        let init = js_sys::Object::new();
        for (name, value) in [
            ("key", "Enter".into()),
            ("code", "Enter".into()),
            ("bubbles", true.into()),
            ("cancelable", true.into()),
        ] {
            js_sys::Reflect::set(&init, &name.into(), &value).unwrap();
        }
        let arguments = js_sys::Array::new();
        arguments.push(&"keydown".into());
        arguments.push(&init);
        let event: Event = js_sys::Reflect::construct(constructor.unchecked_ref(), &arguments)
            .unwrap()
            .unchecked_into();
        define(&event, "keyCode", "value", &13.into());
        (
            Self {
                frame,
                root: root.clone(),
                previous,
            },
            event,
        )
    }
}
impl Drop for ForeignDocument {
    fn drop(&mut self) {
        self.previous.append_child(&self.root).unwrap();
        self.frame.remove();
    }
}

pub fn down(key: &str, code: &str, number: u32) -> Event {
    let event = event(
        "KeyboardEvent",
        "keydown",
        &[
            ("key", key.into()),
            ("code", code.into()),
            ("keyCode", number.into()),
        ],
    );
    // Deprecated constructor keyCode initialization differs across engines.
    // These getter tests require the same explicit field value in each one.
    define(&event, "keyCode", "value", &number.into());
    event
}
