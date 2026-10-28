use super::support::event;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::Event;
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
