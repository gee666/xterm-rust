#![cfg(target_arch = "wasm32")]
// Licensed under MIT. The fixture is the unchanged upstream _inputEvent body.
#[path = "support/event_dom.rs"]
mod descriptors;
#[path = "support/input_dom.rs"]
mod support;
use descriptors::{define, Errors};
use std::{cell::Cell, cell::RefCell, rc::Rc};
use support::*;
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_test::*;
use web_sys::Event;
use xterm_web::utf16::to_js_string;
wasm_bindgen_test_configure!(run_in_browser);

type Getter = Closure<dyn Fn() -> Result<JsValue, JsValue>>;
struct Observed {
    event: Event,
    reads: Rc<RefCell<Vec<&'static str>>>,
    // Descriptors borrow their closures for the full dispatch/oracle invocation.
    _getters: Vec<Getter>,
}
impl Observed {
    fn new(first: JsValue, second: JsValue, insert: bool, composed: bool, throws: &str) -> Self {
        let event = event("InputEvent", "input", &[("bubbles", true.into())]);
        let reads = Rc::new(RefCell::new(Vec::new()));
        let mut getters = Vec::new();
        for field in ["data", "inputType", "composed"] {
            let out = reads.clone();
            let first = first.clone();
            let second = second.clone();
            let count = Cell::new(0);
            let throws = throws.to_owned();
            let getter = Getter::new(move || {
                let previous = count.replace(count.get() + 1);
                let label = if field == "data" {
                    if previous == 0 {
                        "guard data"
                    } else {
                        "payload data"
                    }
                } else {
                    field
                };
                out.borrow_mut().push(label);
                if label == throws {
                    return Err(JsValue::from_str(label));
                }
                Ok(match field {
                    "data" if previous == 0 => first.clone(),
                    "data" => second.clone(),
                    "inputType" => if insert {
                        "insertText"
                    } else {
                        "insertCompositionText"
                    }
                    .into(),
                    _ => composed.into(),
                })
            });
            define(&event, field, "get", getter.as_ref());
            getters.push(getter);
        }
        Self {
            event,
            reads,
            _getters: getters,
        }
    }
}
fn receiver(screen_reader: bool, seen: bool, handled: bool) -> JsValue {
    js_sys::Function::new_with_args(
        "screenReader,seen,handled",
        "return {
      _keyDownSeen:seen, _keyPressHandled:handled, _unprocessedDeadKey:true,
      optionsService:{rawOptions:{screenReaderMode:screenReader}}, sent:[],
      coreService:{triggerDataEvent(data,user){ this.owner.sent.push([data,user]); }}
    };",
    )
    .call3(
        &JsValue::UNDEFINED,
        &screen_reader.into(),
        &seen.into(),
        &handled.into(),
    )
    .unwrap()
}
fn source(host: &JsValue, event: &Event) -> Result<JsValue, JsValue> {
    let service = js_sys::Reflect::get(host, &"coreService".into()).unwrap();
    js_sys::Reflect::set(&service, &"owner".into(), host).unwrap();
    js_sys::Function::new_with_args("ev", include_str!("fixtures/input-event-body.js"))
        .call1(host, event)
}
fn set_state(setup: &Setup, seen: bool, handled: bool, screen_reader: bool) {
    if handled {
        let press = event("KeyboardEvent", "keypress", &[]);
        define(&press, "charCode", "value", &97.into());
        setup.dom.textarea.dispatch_event(&press).unwrap();
    }
    if seen {
        setup.host.veto.set(true);
        setup
            .dom
            .textarea
            .dispatch_event(&event("KeyboardEvent", "keydown", &[]))
            .unwrap();
        setup.host.veto.set(false);
    }
    setup.host.data.borrow_mut().clear();
    setup.host.effects.borrow_mut().clear();
    let mut options = setup.host.keyboard.get();
    options.screen_reader_mode = screen_reader;
    setup.host.keyboard.set(options);
}

#[wasm_bindgen_test]
fn real_input_events_match_source_guards_and_repeated_text_reads() {
    let errors = Errors::new();
    let second: JsValue = to_js_string(&[0xd800, 97]).into();
    for first in [
        JsValue::NULL,
        JsValue::UNDEFINED,
        "".into(),
        false.into(),
        0.into(),
        f64::NAN.into(),
        true.into(),
        js_sys::Object::new().into(),
        "first".into(),
    ] {
        for bits in 0..32 {
            let insert = bits & 1 != 0;
            let composed = bits & 2 != 0;
            let seen = bits & 4 != 0;
            let handled = bits & 8 != 0;
            let screen_reader = bits & 16 != 0;
            let original = Observed::new(first.clone(), second.clone(), insert, composed, "");
            let host = receiver(screen_reader, seen, handled);
            let accepted = source(&host, &original.event).unwrap().as_bool().unwrap();
            let rust = Observed::new(first.clone(), second.clone(), insert, composed, "");
            let setup = Setup::new();
            set_state(&setup, seen, handled, screen_reader);
            setup.dom.textarea.dispatch_event(&rust.event).unwrap();
            assert_eq!(
                *rust.reads.borrow(),
                *original.reads.borrow(),
                "bits={bits}"
            );
            assert_eq!(
                setup.host.data.borrow().len(),
                usize::from(accepted),
                "bits={bits}"
            );
            if accepted {
                assert_eq!(setup.host.data.borrow()[0], (vec![0xd800, 97], true));
            }
            errors.empty();
        }
    }
}

#[wasm_bindgen_test]
fn throwing_input_getters_match_source_error_identity_and_read_prefix() {
    let errors = Errors::new();
    for field in ["guard data", "inputType", "composed", "payload data"] {
        let original = Observed::new("first".into(), "second".into(), true, false, field);
        let thrown = source(&receiver(false, false, false), &original.event).unwrap_err();
        let rust = Observed::new("first".into(), "second".into(), true, false, field);
        let setup = Setup::new();
        setup.dom.textarea.dispatch_event(&rust.event).unwrap();
        assert_eq!(*errors.values.borrow(), [thrown]);
        assert_eq!(*rust.reads.borrow(), *original.reads.borrow());
        assert!(setup.host.data.borrow().is_empty());
        errors.values.borrow_mut().clear();
    }
    // Object-valued exceptions must not be stringified by the DOM adapter.
    let setup = Setup::new();
    let marker: JsValue = js_sys::Object::new().into();
    let thrown = marker.clone();
    let getter = Getter::new(move || Err(thrown.clone()));
    let input = event("InputEvent", "input", &[]);
    define(&input, "data", "get", getter.as_ref());
    setup.dom.textarea.dispatch_event(&input).unwrap();
    assert_eq!(*errors.values.borrow(), [marker]);
}

#[wasm_bindgen_test]
fn admitted_event_sends_second_empty_string_instead_of_first_data() {
    let setup = Setup::new();
    let errors = Errors::new();
    let input = Observed::new("first".into(), "".into(), true, false, "");
    setup.dom.textarea.dispatch_event(&input.event).unwrap();
    assert_eq!(*setup.host.data.borrow(), [(units(""), true)]);
    errors.empty();
}

#[wasm_bindgen_test(async)]
async fn composed_getter_can_reenter_keydown_before_the_live_seen_check() {
    let setup = Setup::new();
    let errors = Errors::new();
    let input = Observed::new("first".into(), "second".into(), true, false, "");
    let area = setup.dom.textarea.clone();
    setup.host.veto.set(true);
    let getter = Getter::new(move || {
        area.dispatch_event(&event("KeyboardEvent", "keydown", &[]))?;
        Ok(true.into())
    });
    define(&input.event, "composed", "get", getter.as_ref());
    setup.dom.textarea.dispatch_event(&input.event).unwrap();
    next_turn().await;
    assert_eq!(*input.reads.borrow(), ["guard data", "inputType"]);
    assert!(setup.host.data.borrow().is_empty());
    errors.empty();
}
