#![cfg(target_arch = "wasm32")]
//! Exercise the real Terminal input host, not an input-only MouseHost substitute.
//! The typed callbacks here do not claim raw registry or callback receiver parity.
use js_sys::{Array, Function, Object, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{Event, HtmlElement};
use xterm_core::{
    lifecycle::DisposableHandle,
    mouse_state::{MouseEventTypes, MouseProtocol, MouseStateService},
};
use xterm_web::{js_error::callback_error, Terminal};
wasm_bindgen_test_configure!(run_in_browser);

struct Setup {
    terminal: Rc<Terminal>,
    mouse: Rc<MouseStateService>,
    parent: HtmlElement,
    output: Rc<RefCell<Vec<Vec<u16>>>>,
    listeners: Vec<DisposableHandle>,
}
impl Setup {
    fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let parent: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
        document.body().unwrap().append_child(&parent).unwrap();
        let options = Object::new();
        for (key, value) in [("cols", 10), ("rows", 4), ("scrollback", 0)] {
            Reflect::set(&options, &key.into(), &value.into()).unwrap();
        }
        let terminal = Rc::new(Terminal::new(options.into()).unwrap());
        terminal.open(parent.clone().into()).unwrap();
        let native = terminal.shared_native();
        let mouse = native.with_state(|state| state.mouse.clone()).unwrap();
        mouse.set_active_encoding("SGR").unwrap();
        mouse.set_active_protocol("ANY").unwrap();
        let output = Rc::new(RefCell::new(Vec::new()));
        let data = output.clone();
        let binary = output.clone();
        let listeners = vec![
            native
                .on_data()
                .subscribe(move |value| data.borrow_mut().push(value.clone())),
            native
                .on_binary()
                .subscribe(move |value| binary.borrow_mut().push(value.clone())),
        ];
        Self {
            terminal,
            mouse,
            parent,
            output,
            listeners,
        }
    }
    fn pointer(&self) -> Event {
        let rect = self
            .terminal
            .screen_element()
            .unwrap()
            .get_bounding_client_rect();
        Function::new_with_args("x,y", "return new MouseEvent('mousemove',{clientX:x,clientY:y,buttons:0,bubbles:true,cancelable:true});")
            .call2(&JsValue::UNDEFINED, &(rect.left() + 4.0).into(), &(rect.top() + 4.0).into())
            .unwrap().dyn_into().unwrap()
    }
    fn dispatch(&self) {
        self.terminal
            .element()
            .unwrap()
            .dispatch_event(&self.pointer())
            .unwrap();
    }
}
impl Drop for Setup {
    fn drop(&mut self) {
        for listener in &self.listeners {
            listener.dispose();
        }
        self.terminal.dispose().unwrap();
        self.parent.remove();
    }
}

// Like the other real DOM exception tests, trap only expected dispatch errors.
// Keep cleanup local so a failed assertion cannot leave a window handler behind.
struct ErrorTrap {
    errors: Rc<RefCell<Vec<JsValue>>>,
    listener: Closure<dyn Fn(Event)>,
}
impl ErrorTrap {
    fn new() -> Self {
        let errors = Rc::new(RefCell::new(Vec::new()));
        let observed = errors.clone();
        let listener = Closure::<dyn Fn(Event)>::new(move |event: Event| {
            observed
                .borrow_mut()
                .push(Reflect::get(&event, &"error".into()).unwrap());
            event.prevent_default();
        });
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("error", listener.as_ref().unchecked_ref())
            .unwrap();
        Self { errors, listener }
    }
}
impl Drop for ErrorTrap {
    fn drop(&mut self) {
        web_sys::window()
            .unwrap()
            .remove_event_listener_with_callback("error", self.listener.as_ref().unchecked_ref())
            .unwrap();
    }
}

fn throwing(value: &JsValue) -> Function {
    Function::new_with_args("value", "return function(){throw value;};")
        .call1(&JsValue::UNDEFINED, value)
        .unwrap()
        .dyn_into()
        .unwrap()
}

#[wasm_bindgen_test]
fn restriction_and_encoder_throws_keep_all_js_value_identities_through_terminal_dom() {
    let values: Array =
        Function::new_no_args("return [undefined,null,false,0,NaN,Symbol('throw'),0n,{}];")
            .call0(&JsValue::UNDEFINED)
            .unwrap()
            .dyn_into()
            .unwrap();
    for restriction in [true, false] {
        for value in values.iter() {
            let setup = Setup::new();
            let trap = ErrorTrap::new();
            let callback = throwing(&value);
            if restriction {
                setup.mouse.add_protocol(
                    "ERROR",
                    MouseProtocol {
                        events: MouseEventTypes::MOVE,
                        restrict: Rc::new(move |_| {
                            callback
                                .call0(&JsValue::UNDEFINED)
                                .map(|result| result.is_truthy())
                                .map_err(callback_error)
                        }),
                    },
                );
                setup.mouse.set_active_protocol("ERROR").unwrap();
            } else {
                setup.mouse.add_encoding(
                    "ERROR",
                    Rc::new(move |_| {
                        callback
                            .call0(&JsValue::UNDEFINED)
                            .map(|_| Vec::new())
                            .map_err(callback_error)
                    }),
                );
                setup.mouse.set_active_encoding("ERROR").unwrap();
            }
            setup.dispatch();
            assert_eq!(
                trap.errors.borrow().len(),
                1,
                "callback must report its exception exactly once"
            );
            assert!(
                Object::is(&trap.errors.borrow()[0], &value),
                "original JS thrown value survives the portable error chain"
            );
            assert!(
                setup.output.borrow().is_empty(),
                "no report after callback failure"
            );
            setup.mouse.set_active_protocol("ANY").unwrap();
            setup.mouse.set_active_encoding("SGR").unwrap();
            setup.dispatch();
            assert_eq!(
                setup.output.borrow().len(),
                1,
                "failed move must not commit debounce"
            );
            assert_eq!(trap.errors.borrow().len(), 1);
            setup.dispatch();
            assert_eq!(
                setup.output.borrow().len(),
                1,
                "successful retry commits debounce"
            );
        }
    }
}

#[wasm_bindgen_test]
fn callback_can_replace_registration_dispatch_recursively_then_throw() {
    for restriction in [true, false] {
        let setup = Setup::new();
        let trap = ErrorTrap::new();
        let sentinel: JsValue = Object::new().into();
        let thrown = sentinel.clone();
        let weak = Rc::downgrade(&setup.mouse);
        let element = setup.terminal.element().unwrap();
        let event = setup.pointer();
        let nested = move || {
            let mouse = weak.upgrade().unwrap();
            mouse.set_active_protocol("ANY").unwrap();
            mouse.set_active_encoding("SGR").unwrap();
            element.dispatch_event(&event).unwrap();
        };
        if restriction {
            setup.mouse.add_protocol(
                "ERROR",
                MouseProtocol {
                    events: MouseEventTypes::MOVE,
                    restrict: Rc::new(move |_| {
                        nested();
                        Err(callback_error(thrown.clone()))
                    }),
                },
            );
            setup.mouse.set_active_protocol("ERROR").unwrap();
        } else {
            setup.mouse.add_encoding(
                "ERROR",
                Rc::new(move |_| {
                    nested();
                    Err(callback_error(thrown.clone()))
                }),
            );
            setup.mouse.set_active_encoding("ERROR").unwrap();
        }
        setup.dispatch();
        assert_eq!(trap.errors.borrow().len(), 1);
        assert!(Object::is(&trap.errors.borrow()[0], &sentinel));
        assert_eq!(
            setup.output.borrow().len(),
            1,
            "only the nested report succeeds"
        );
        setup.dispatch();
        assert_eq!(
            setup.output.borrow().len(),
            1,
            "outer failure must not roll back nested cache"
        );
    }
}
