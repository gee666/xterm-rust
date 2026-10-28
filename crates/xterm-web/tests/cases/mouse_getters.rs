use super::{support::input::event, Setup};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::Event;
use xterm_web::dom_listener::DomListener;

pub(super) type Getter = Closure<dyn Fn() -> Result<JsValue, JsValue>>;
pub(super) fn getter(
    event: &Event,
    name: &str,
    read: impl Fn() -> Result<JsValue, JsValue> + 'static,
) -> Getter {
    let closure = Closure::wrap(Box::new(read) as Box<dyn Fn() -> Result<JsValue, JsValue>>);
    let descriptor = js_sys::Object::new();
    js_sys::Reflect::set(&descriptor, &"get".into(), closure.as_ref()).unwrap();
    js_sys::Reflect::set(&descriptor, &"configurable".into(), &true.into()).unwrap();
    assert!(
        js_sys::Reflect::define_property(event.unchecked_ref(), &name.into(), &descriptor).unwrap()
    );
    closure
}
fn activate(setup: &Setup) {
    setup.host.input.mouse.set_active_protocol("ANY").unwrap();
    setup.host.input.mouse.set_active_encoding("SGR").unwrap();
    setup.host.reads.borrow_mut().clear();
}

struct ErrorTrap {
    errors: Rc<RefCell<Vec<JsValue>>>,
    _listener: DomListener,
}
impl ErrorTrap {
    fn new() -> Self {
        let errors = Rc::new(RefCell::new(Vec::new()));
        let captured = errors.clone();
        let listener = DomListener::new_reentrant_fallible(
            web_sys::window().unwrap().as_ref(),
            "error",
            false,
            None,
            move |event| {
                captured
                    .borrow_mut()
                    .push(js_sys::Reflect::get(&event, &"error".into())?);
                event.prevent_default();
                Ok(())
            },
        )
        .unwrap();
        Self {
            errors,
            _listener: listener,
        }
    }
}

#[wasm_bindgen_test]
fn wheel_never_reads_button_fields() {
    let setup = Setup::new();
    activate(&setup);
    let wheel = setup.wheel();
    // Hold the closures until after dispatch, without leaking them.
    let _buttons = getter(&wheel, "buttons", || Err("irrelevant buttons".into()));
    let _button = getter(&wheel, "button", || Err("irrelevant button".into()));
    setup.dom.element.dispatch_event(&wheel).unwrap();
    assert_eq!(setup.host.input.data.borrow().len(), 1);
    assert!(wheel.default_prevented());
}

#[wasm_bindgen_test]
fn truthy_override_avoids_type_and_unknown_event_fields() {
    let setup = Setup::new();
    activate(&setup);
    for override_type in [JsValue::from_str("unknown"), JsValue::from_f64(1.0)] {
        let wheel = setup.wheel();
        js_sys::Reflect::set(&wheel, &"overrideType".into(), &override_type).unwrap();
        let _type = getter(&wheel, "type", || Err("irrelevant type".into()));
        let _ctrl = getter(&wheel, "ctrlKey", || Err("irrelevant ctrl".into()));
        setup.dom.element.dispatch_event(&wheel).unwrap();
        assert!(wheel.default_prevented());
    }
    assert_eq!(setup.host.wheel_calls.get(), 0);
    assert!(setup.host.input.data.borrow().is_empty());
}

#[wasm_bindgen_test]
fn zero_and_veto_do_not_read_later_fields_or_wheel_metrics() {
    let setup = Setup::new();
    activate(&setup);
    setup.host.fail_read.set(Some("device-height"));
    let zero = setup.wheel();
    let _delta = getter(&zero, "deltaY", || Ok(0.into()));
    let _ctrl = getter(&zero, "ctrlKey", || Err("irrelevant ctrl".into()));
    setup.dom.element.dispatch_event(&zero).unwrap();
    assert_eq!(
        setup.host.reads.borrow().as_slice(),
        &["valid", "canvas", "cell"]
    );
    assert!(zero.default_prevented());
    setup.host.reads.borrow_mut().clear();
    setup.host.veto.set(true);
    let vetoed = setup.wheel();
    let _delta_vetoed = getter(&vetoed, "deltaY", || Err("delta after veto".into()));
    setup.dom.element.dispatch_event(&vetoed).unwrap();
    assert_eq!(
        setup.host.reads.borrow().as_slice(),
        &["valid", "canvas", "cell"]
    );
    assert!(vetoed.default_prevented());
    assert!(setup.host.input.data.borrow().is_empty());
}

#[wasm_bindgen_test]
fn wheel_callback_mutation_precedes_three_live_delta_reads() {
    let setup = Setup::new();
    activate(&setup);
    let value = Rc::new(Cell::new(1.0));
    let reads = Rc::new(Cell::new(0));
    let getter_value = value.clone();
    let getter_reads = reads.clone();
    let wheel = setup.wheel();
    let _delta = getter(&wheel, "deltaY", move || {
        getter_reads.set(getter_reads.get() + 1);
        Ok(getter_value.get().into())
    });
    *setup.host.wheel_hook.borrow_mut() = Some(Rc::new(move |_| value.set(-1.0)));
    setup.dom.element.dispatch_event(&wheel).unwrap();
    assert_eq!(reads.get(), 3);
    assert_eq!(
        setup.host.input.data.borrow()[0].0,
        "\x1b[<64;1;1M".encode_utf16().collect::<Vec<_>>()
    );
}

#[wasm_bindgen_test]
fn missing_buttons_uses_the_source_fallback_getter_sequence() {
    let setup = Setup::new();
    activate(&setup);
    let movement = setup.pointer("mousemove", 0);
    let button_reads = Rc::new(Cell::new(0));
    let counted = button_reads.clone();
    let _buttons = getter(&movement, "buttons", || Ok(JsValue::UNDEFINED));
    let _button = getter(&movement, "button", move || {
        counted.set(counted.get() + 1);
        Ok(0.into())
    });
    setup.dom.element.dispatch_event(&movement).unwrap();
    assert_eq!(button_reads.get(), 3);
    assert_eq!(setup.host.input.data.borrow().len(), 1);
}

#[wasm_bindgen_test]
fn inactive_and_pointer_paths_skip_unrelated_host_reads() {
    let setup = Setup::new();
    setup.host.fail_read.set(Some("require-alt"));
    setup.host.reads.borrow_mut().clear();
    setup.host.input.mouse.set_active_protocol("NONE").unwrap();
    assert!(setup.host.reads.borrow().is_empty());
    setup.host.fail_read.set(Some("fast"));
    activate(&setup);
    setup
        .dom
        .element
        .dispatch_event(&setup.pointer("mousemove", 0))
        .unwrap();
    assert_eq!(setup.host.input.data.borrow().len(), 1);
    let reads = setup.host.reads.borrow();
    for key in ["fast", "slow", "dpr", "device-height", "snapshot"] {
        assert!(!reads.contains(&key));
    }
}

#[wasm_bindgen_test]
fn numeric_value_of_can_reenter_protocol_changes() {
    let setup = Setup::new();
    activate(&setup);
    let wheel = setup.wheel();
    let value = js_sys::Object::new();
    let mouse = setup.host.input.mouse.clone();
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    let conversion = Closure::wrap(Box::new(move || -> Result<JsValue, JsValue> {
        count.set(count.get() + 1);
        mouse
            .set_active_protocol("NONE")
            .map_err(|error| xterm_web::js_error::to_js_error(&error))?;
        Ok(1.into())
    }) as Box<dyn Fn() -> Result<JsValue, JsValue>>);
    js_sys::Reflect::set(&value, &"valueOf".into(), conversion.as_ref()).unwrap();
    let _delta = getter(&wheel, "deltaY", move || Ok(value.clone().into()));
    setup.dom.element.dispatch_event(&wheel).unwrap();
    assert_eq!(
        calls.get(),
        2,
        "multiply and later direction compare use the live object"
    );
    assert_eq!(setup.host.input.mouse.active_protocol(), "NONE");
    assert!(setup.host.input.data.borrow().is_empty());
    assert!(wheel.default_prevented());
}

#[wasm_bindgen_test]
fn numeric_coercion_failure_retains_identity_after_sensitivity_read() {
    let setup = Setup::new();
    activate(&setup);
    let trap = ErrorTrap::new();
    let wheel = setup.wheel();
    let value = js_sys::Object::new();
    let sentinel: JsValue = js_sys::Object::new().into();
    let thrown = sentinel.clone();
    let conversion =
        Closure::wrap(
            Box::new(move || -> Result<JsValue, JsValue> { Err(thrown.clone()) })
                as Box<dyn Fn() -> Result<JsValue, JsValue>>,
        );
    js_sys::Reflect::set(&value, &"valueOf".into(), conversion.as_ref()).unwrap();
    let _delta = getter(&wheel, "deltaY", move || Ok(value.clone().into()));
    setup.dom.element.dispatch_event(&wheel).unwrap();
    assert_eq!(trap.errors.borrow().len(), 1);
    assert!(js_sys::Object::is(&trap.errors.borrow()[0], &sentinel));
    assert_eq!(
        setup.host.reads.borrow().as_slice(),
        &["valid", "canvas", "cell", "device-height", "dpr", "slow"]
    );
    assert!(setup.host.input.data.borrow().is_empty());
    assert!(!wheel.default_prevented());
}

#[wasm_bindgen_test]
fn thrown_client_getter_preserves_identity_and_stops_later_reads() {
    let setup = Setup::new();
    activate(&setup);
    let trap = ErrorTrap::new();
    let sentinel: JsValue = js_sys::Object::new().into();
    let thrown = sentinel.clone();
    let wheel = setup.wheel();
    let _client = getter(&wheel, "clientX", move || Err(thrown.clone()));
    setup.dom.element.dispatch_event(&wheel).unwrap();
    assert_eq!(trap.errors.borrow().len(), 1);
    assert!(js_sys::Object::is(&trap.errors.borrow()[0], &sentinel));
    assert!(setup.host.reads.borrow().is_empty());
    assert_eq!(setup.host.wheel_calls.get(), 0);
    assert!(setup.host.input.data.borrow().is_empty());
    assert!(!wheel.default_prevented());
}
