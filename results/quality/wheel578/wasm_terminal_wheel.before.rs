#![cfg(target_arch = "wasm32")]
//! Real terminal routing, not an input-only fake MouseHost. Raw service receiver
//! and ordinary-object registry compatibility are separate unfinished contracts.
use js_sys::{Array, Function, Object, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{Event, HtmlElement};
use xterm_web::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

struct Setup {
    terminal: Rc<Terminal>,
    parent: HtmlElement,
    style: web_sys::Element,
    output: Rc<RefCell<Vec<JsValue>>>,
    subscription: JsValue,
    _listener: Closure<dyn Fn(JsValue)>,
}
impl Setup {
    fn new(active: bool) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let style = document.create_element("style").unwrap();
        style.set_text_content(Some(xterm_web::XTERM_CSS));
        document.body().unwrap().append_child(&style).unwrap();
        let parent: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
        document.body().unwrap().append_child(&parent).unwrap();
        let options = Object::new();
        for (key, value) in [("cols", 10), ("rows", 4), ("scrollback", 0)] {
            Reflect::set(&options, &key.into(), &value.into()).unwrap();
        }
        let terminal = Rc::new(Terminal::new(options.into()).unwrap());
        terminal.open(parent.clone().into()).unwrap();
        let mouse = terminal
            .shared_native()
            .with_state(|state| state.mouse.clone())
            .unwrap();
        mouse.set_active_encoding("SGR").unwrap();
        mouse
            .set_active_protocol(if active { "ANY" } else { "NONE" })
            .unwrap();
        let output = Rc::new(RefCell::new(Vec::new()));
        let captured = output.clone();
        let listener = Closure::<dyn Fn(JsValue)>::new(move |value| {
            captured.borrow_mut().push(value);
        });
        let subscription = terminal
            .on_data()
            .call1(&JsValue::UNDEFINED, listener.as_ref())
            .unwrap();
        Self {
            terminal,
            parent,
            style,
            output,
            subscription,
            _listener: listener,
        }
    }
    fn wheel(&self) -> Event {
        let rect = self
            .terminal
            .screen_element()
            .unwrap()
            .get_bounding_client_rect();
        Function::new_with_args("x,y", "return new WheelEvent('wheel', {clientX:x,clientY:y,deltaY:1,deltaMode:1,bubbles:true,cancelable:true});")
            .call2(&JsValue::UNDEFINED, &(rect.left() + 4.0).into(), &(rect.top() + 4.0).into())
            .unwrap().dyn_into().unwrap()
    }
    fn dispatch(&self, event: &Event) {
        self.terminal
            .element()
            .unwrap()
            .dispatch_event(event)
            .unwrap();
    }
}
impl Drop for Setup {
    fn drop(&mut self) {
        let dispose: Function = Reflect::get(&self.subscription, &"dispose".into())
            .unwrap()
            .dyn_into()
            .unwrap();
        dispose.call0(&self.subscription).unwrap();
        self.terminal.dispose().unwrap();
        self.parent.remove();
        self.style.remove();
    }
}

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

#[wasm_bindgen_test]
fn active_and_passive_hooks_ignore_own_call_and_keep_event_identity() {
    for active in [false, true] {
        let setup = Setup::new(active);
        let trap = ErrorTrap::new();
        for getter in [false, true] {
            let event = setup.wheel();
            let expected = JsValue::from(event.clone());
            let observed = Rc::new(RefCell::new(Vec::new()));
            let captured = observed.clone();
            let callback = Closure::<dyn Fn(JsValue) -> bool>::new(move |event| {
                captured.borrow_mut().push(event);
                true
            });
            let poison = Function::new_with_args("fn,getter", "if(getter) Object.defineProperty(fn,'call',{get(){throw 'must not read call';}}); else fn.call = 42;");
            poison
                .call2(&JsValue::UNDEFINED, callback.as_ref(), &getter.into())
                .unwrap();
            setup.terminal.attach_custom_wheel_event_handler(
                callback.as_ref().unchecked_ref::<Function>().clone(),
            );
            setup.dispatch(&event);
            assert!(
                trap.errors.borrow().is_empty(),
                "callback.call must not be read"
            );
            assert_eq!(observed.borrow().len(), 1);
            assert!(Object::is(&observed.borrow()[0], &expected));
            assert!(event.default_prevented());
        }
        assert_eq!(setup.output.borrow().len(), 2);
    }
}

#[wasm_bindgen_test]
fn data_listeners_still_observe_own_call_and_preserve_thrown_identity() {
    let setup = Setup::new(true);
    let trap = ErrorTrap::new();
    let sentinel: JsValue = Object::new().into();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let captured = calls.clone();
    let listener = Closure::<dyn Fn(JsValue)>::new(move |event| {
        captured.borrow_mut().push(event);
    });
    Function::new_with_args(
        "fn,sentinel",
        "Object.defineProperty(fn,'call',{get(){throw sentinel;}});",
    )
    .call2(&JsValue::UNDEFINED, listener.as_ref(), &sentinel)
    .unwrap();
    let token = setup
        .terminal
        .on_data()
        .call1(&JsValue::UNDEFINED, listener.as_ref())
        .unwrap();
    let event = setup.wheel();
    setup.dispatch(&event);
    assert_eq!(trap.errors.borrow().len(), 1);
    assert!(Object::is(&trap.errors.borrow()[0], &sentinel));
    assert!(calls.borrow().is_empty());
    assert_eq!(
        setup.output.borrow().len(),
        1,
        "earlier listener runs before the throw"
    );
    assert!(!event.default_prevented());
    let dispose: Function = Reflect::get(&token, &"dispose".into())
        .unwrap()
        .dyn_into()
        .unwrap();
    dispose.call0(&token).unwrap();
}

#[wasm_bindgen_test]
fn only_literal_false_vetoes_both_live_routes() {
    let values: Array = Function::new_no_args("return [false,undefined,null,0,-0,NaN,'',Symbol('wheel'),0n,{},[],new Boolean(false),true,1,'false'];")
        .call0(&JsValue::UNDEFINED).unwrap().dyn_into().unwrap();
    for active in [false, true] {
        let setup = Setup::new(active);
        for (index, value) in values.iter().enumerate() {
            let callback = Closure::<dyn Fn(JsValue) -> JsValue>::new(move |_| value.clone());
            setup.terminal.attach_custom_wheel_event_handler(
                callback.as_ref().unchecked_ref::<Function>().clone(),
            );
            setup.output.borrow_mut().clear();
            let event = setup.wheel();
            setup.dispatch(&event);
            assert_eq!(
                setup.output.borrow().len(),
                usize::from(index != 0),
                "result class {index}, active={active}"
            );
            assert_eq!(event.default_prevented(), active || index != 0);
        }
    }
}

#[wasm_bindgen_test]
fn thrown_values_retain_identity_and_stop_output() {
    let values: Array =
        Function::new_no_args("return [undefined,null,false,0,NaN,Symbol('throw'),0n,{}];")
            .call0(&JsValue::UNDEFINED)
            .unwrap()
            .dyn_into()
            .unwrap();
    for active in [false, true] {
        let setup = Setup::new(active);
        let trap = ErrorTrap::new();
        for value in values.iter() {
            let thrown = value.clone();
            let callback = Closure::<dyn Fn(JsValue) -> Result<JsValue, JsValue>>::new(move |_| {
                Err(thrown.clone())
            });
            setup.terminal.attach_custom_wheel_event_handler(
                callback.as_ref().unchecked_ref::<Function>().clone(),
            );
            let event = setup.wheel();
            setup.dispatch(&event);
            let errors = trap.errors.borrow();
            assert!(Object::is(errors.last().unwrap(), &value));
            assert!(
                !event.default_prevented(),
                "exception precedes cancellation"
            );
            assert!(setup.output.borrow().is_empty());
        }
        assert_eq!(trap.errors.borrow().len(), values.length() as usize);
    }
}

#[wasm_bindgen_test]
fn hook_replacement_and_recursive_dispatch_do_not_hold_service_borrows() {
    for active in [false, true] {
        let setup = Setup::new(active);
        let inner = setup.wheel();
        let outer = setup.wheel();
        let seen = Rc::new(RefCell::new(Vec::new()));
        let inner_seen = seen.clone();
        let replacement = Closure::<dyn Fn(JsValue) -> bool>::new(move |event| {
            inner_seen.borrow_mut().push(event);
            true
        });
        let replacement_fn = replacement.as_ref().unchecked_ref::<Function>().clone();
        let terminal = setup.terminal.clone();
        let element = terminal.element().unwrap();
        let inner_event = inner.clone();
        let outer_seen = seen.clone();
        let callback = Closure::<dyn Fn(JsValue) -> bool>::new(move |event| {
            outer_seen.borrow_mut().push(event);
            terminal.attach_custom_wheel_event_handler(replacement_fn.clone());
            element.dispatch_event(&inner_event).unwrap();
            false
        });
        setup.terminal.attach_custom_wheel_event_handler(
            callback.as_ref().unchecked_ref::<Function>().clone(),
        );
        setup.dispatch(&outer);
        let observed = seen.borrow();
        assert_eq!(observed.len(), 2);
        assert!(Object::is(&observed[0], outer.as_ref()));
        assert!(Object::is(&observed[1], inner.as_ref()));
        assert_eq!(setup.output.borrow().len(), 1);
        assert!(inner.default_prevented());
        assert_eq!(outer.default_prevented(), active);
    }
}
