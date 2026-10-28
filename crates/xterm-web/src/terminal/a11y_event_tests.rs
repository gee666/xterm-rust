//! Actual Rust producer to WASM callback conversion. Not a JS terminal substitute.
use super::events;
use crate::{js_error::to_js_error, utf16::from_js_string};
use js_sys::{Array, Function, JsString, Object, Reflect};
use std::{collections::BTreeMap, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_core::{
    charset::Charset,
    event::Emitter,
    headless::Terminal,
    lifecycle::DisposableHandle,
    options::{OptionValue, OptionsService},
    terminal::TerminalEvent,
};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(inline_js = "
export function a11yRecorder(store) { return value => store.push(value); }
export function a11yThrower(store, sentinel) {
  return value => { store.push(value); throw sentinel; };
}
export function a11ySymbolFailure() { return Symbol('a11y sentinel'); }
export function a11yOpaqueFailure() {
  return {toString() { throw new Error('a11y failure must not be stringified'); }};
}
")]
extern "C" {
    #[wasm_bindgen(js_name = a11yRecorder)]
    fn recorder(store: &Array) -> Function;
    #[wasm_bindgen(js_name = a11yThrower)]
    fn thrower(store: &Array, sentinel: &JsValue) -> Function;
    #[wasm_bindgen(js_name = a11yOpaqueFailure)]
    fn opaque_failure() -> JsValue;
    #[wasm_bindgen(js_name = a11ySymbolFailure)]
    fn symbol_failure() -> JsValue;
}

struct Bridge {
    terminal: Terminal,
    callback: Function,
    forward: DisposableHandle,
}
impl Bridge {
    fn new(enabled: bool) -> Self {
        let terminal = Terminal::with_options(Rc::new(OptionsService::new(
            false,
            [
                ("cols", OptionValue::Number(40.0)),
                ("rows", OptionValue::Number(3.0)),
                ("screenReaderMode", OptionValue::Bool(enabled)),
            ],
        )))
        .unwrap();
        // This forwards the real native event through the same generic callback
        // converter used by browser event getters. No public test API is added.
        let output = Rc::new(Emitter::<Vec<u16>>::new());
        let callback = events::callable(output.event(), |text| events::text(text));
        let forward = terminal.on_event().try_subscribe(move |event| {
            if let TerminalEvent::AccessibilityChar(text) = event {
                output.fire(text)
            } else {
                Ok(())
            }
        });
        Self {
            terminal,
            callback,
            forward,
        }
    }
    fn subscribe(&self, listener: &Function) -> JsValue {
        self.callback.call1(&JsValue::UNDEFINED, listener).unwrap()
    }
    fn finish(self) {
        self.forward.dispose();
        self.terminal.dispose().unwrap();
    }
}
fn unsubscribe(subscription: &JsValue) {
    Reflect::get(subscription, &"dispose".into())
        .unwrap()
        .dyn_into::<Function>()
        .unwrap()
        .call0(subscription)
        .unwrap();
}
fn payloads(values: &Array) -> Vec<Vec<u16>> {
    values
        .iter()
        .map(|value| from_js_string(&value.dyn_into::<JsString>().unwrap()))
        .collect()
}

#[wasm_bindgen_test]
fn native_a11y_events_reach_javascript_with_exact_utf16_units() {
    let bridge = Bridge::new(true);
    let values = Array::new();
    let subscription = bridge.subscribe(&recorder(&values));
    bridge
        .terminal
        .write_utf16(&[0xdc00, 0xd83d, 0xde00, 0xd800, u16::from(b'A')])
        .unwrap();
    assert_eq!(
        payloads(&values),
        [
            vec![0xdc00],
            vec![0xd83d, 0xde00],
            vec![0xd800],
            vec![u16::from(b'A')]
        ]
    );
    unsubscribe(&subscription);
    bridge.finish();
}

#[wasm_bindgen_test]
fn custom_charset_payloads_keep_source_first_unit_and_nul_behavior() {
    let bridge = Bridge::new(true);
    let values = Array::new();
    let subscription = bridge.subscribe(&recorder(&values));
    let map = Charset::new(BTreeMap::from([
        (u16::from(b'!'), vec![]),
        (u16::from(b'['), vec![0xd800]),
        (u16::from(b']'), vec![0xdc00]),
        (u16::from(b'@'), vec![0xd83d, 0xde00]),
        (u16::from(b'?'), vec![0x69, 0x6a]),
        (u16::from(b'#'), vec![0, 0x78]),
    ]));
    bridge
        .terminal
        .state_handle_for_addon()
        .borrow_mut()
        .charset
        .set_g_charset(0, Some(map))
        .unwrap();
    bridge
        .terminal
        .write_utf16(&"![]@?#".encode_utf16().collect::<Vec<_>>())
        .unwrap();
    assert_eq!(
        payloads(&values),
        [
            vec![0x21],
            vec![0xd800],
            vec![0xdc00],
            vec![0xd83d],
            vec![0x69],
            vec![0]
        ]
    );
    unsubscribe(&subscription);
    bridge.finish();
}

#[wasm_bindgen_test]
fn disabled_a11y_and_pending_high_surrogates_do_not_invent_callbacks() {
    for enabled in [false, true] {
        let bridge = Bridge::new(enabled);
        let values = Array::new();
        let subscription = bridge.subscribe(&recorder(&values));
        bridge.terminal.write_utf16(&[0xd800]).unwrap();
        bridge.terminal.write_utf16(&[]).unwrap();
        assert_eq!(values.length(), 0);
        bridge.terminal.write_utf16(&[u16::from(b'A')]).unwrap();
        assert_eq!(
            payloads(&values),
            if enabled {
                vec![vec![0xd800], vec![u16::from(b'A')]]
            } else {
                vec![]
            }
        );
        unsubscribe(&subscription);
        bridge.finish();
    }
}

#[wasm_bindgen_test]
fn thrown_javascript_values_keep_identity_and_stop_before_cell_write() {
    let symbol = symbol_failure();
    for thrown in [
        opaque_failure(),
        symbol,
        JsValue::from_f64(17.0),
        JsValue::from_str("a11y failure"),
        JsValue::NULL,
        JsValue::UNDEFINED,
    ] {
        let bridge = Bridge::new(true);
        let values = Array::new();
        let subscription = bridge.subscribe(&thrower(&values, &thrown));
        let error = bridge
            .terminal
            .write_utf16(&[0xdc00, u16::from(b'B')])
            .unwrap_err();
        assert!(Object::is(&to_js_error(&error), &thrown));
        assert_eq!(payloads(&values), [vec![0xdc00]]);
        bridge
            .terminal
            .with_state(|state| {
                let buffer = state.active_buffer();
                let buffer = buffer.borrow();
                assert_eq!(buffer.x, 0);
                assert_eq!(
                    buffer
                        .lines
                        .get(buffer.ybase + buffer.y)
                        .unwrap()
                        .borrow()
                        .get_cell(0)
                        .unwrap()
                        .get_chars(),
                    Vec::<u16>::new()
                );
            })
            .unwrap();
        unsubscribe(&subscription);
        bridge.terminal.write_utf16(&[u16::from(b'C')]).unwrap();
        bridge.finish();
    }
}

#[wasm_bindgen_test]
fn real_browser_accessibility_host_forwards_unpaired_units_losslessly() {
    use crate::{
        accessibility::{AccessibilityEvent, AccessibilityHost},
        terminal::browser_services::WeakBrowserServices,
    };
    let document = web_sys::window().unwrap().document().unwrap();
    let parent = document
        .create_element("div")
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
    let options = Object::new();
    for (key, value) in [
        ("cols", JsValue::from_f64(40.0)),
        ("rows", JsValue::from_f64(3.0)),
        ("screenReaderMode", JsValue::TRUE),
    ] {
        assert!(Reflect::set(&options, &key.into(), &value).unwrap());
    }
    let terminal = super::Terminal::new(options.into()).unwrap();
    terminal.open(parent.clone().into()).unwrap();
    let services = terminal.shared.services.borrow().as_ref().unwrap().clone();
    let host = WeakBrowserServices(Rc::downgrade(&services));
    let output = Rc::new(Emitter::<Vec<u16>>::new());
    let callback = events::callable(output.event(), |text| events::text(text));
    let values = Array::new();
    let subscription = callback
        .call1(&JsValue::UNDEFINED, &recorder(&values))
        .unwrap();
    let listeners = host
        .subscribe(Rc::new(move |event| match event {
            AccessibilityEvent::Character(text) => {
                output.fire(&text).map_err(|error| to_js_error(&error))
            }
            _ => Ok(()),
        }))
        .unwrap();
    terminal
        .shared
        .native
        .write_utf16(&[0xdc00, 0xd800, u16::from(b'A')])
        .unwrap();
    assert_eq!(
        payloads(&values),
        [vec![0xdc00], vec![0xd800], vec![u16::from(b'A')]]
    );
    unsubscribe(&subscription);
    values.set_length(0);
    let thrown = opaque_failure();
    let throwing = callback
        .call1(&JsValue::UNDEFINED, &thrower(&values, &thrown))
        .unwrap();
    let before = terminal
        .shared
        .native
        .with_state(|state| state.active_buffer().borrow().x)
        .unwrap();
    let error = terminal
        .shared
        .native
        .write_utf16(&[0xdc00, u16::from(b'B')])
        .unwrap_err();
    assert!(Object::is(&to_js_error(&error), &thrown));
    assert_eq!(payloads(&values), [vec![0xdc00]]);
    assert_eq!(
        terminal
            .shared
            .native
            .with_state(|state| state.active_buffer().borrow().x)
            .unwrap(),
        before
    );
    unsubscribe(&throwing);
    for listener in listeners {
        listener.dispose();
    }
    terminal.dispose().unwrap();
    parent.remove();
}
