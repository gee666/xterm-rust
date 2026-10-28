#![cfg(target_arch = "wasm32")]
// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Source-derived deterministic arrow timer/DOM assertions, with no sleeps.
use js_sys::{Array, Function, Reflect};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use web_sys::{PointerEvent, PointerEventInit};
use xterm_web::scroll::arrow::{ArrowOptions, ScrollbarArrow};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(inline_js = "
export function installArrowClock(){
 const originals={setTimeout,clearTimeout,setInterval,clearInterval};let next=1;
 const timeouts=new Map(),intervals=new Map();let latestInterval;
 window.setTimeout=function(fn,ms){const id=next++;timeouts.set(id,{fn,ms,receiver:this});return id;};
 window.clearTimeout=function(id){timeouts.delete(id);};
 window.setInterval=function(fn,ms){const id=next++;const entry={fn,ms,receiver:this};intervals.set(id,entry);latestInterval=entry;return id;};
 window.clearInterval=function(id){intervals.delete(id);};
 return {restore(){Object.assign(window,originals);},step(ms){for(const[id,t]of timeouts){if(t.ms===ms){timeouts.delete(id);t.fn();return t.receiver;}}throw Error('timeout not scheduled');},tick(){if(!latestInterval)throw Error('interval missing');latestInterval.fn();},delay(){return latestInterval?.ms;},intervalReceiver(){return latestInterval?.receiver;},counts(){return[timeouts.size,intervals.size];}};
}
")]
extern "C" {
    #[wasm_bindgen(catch,js_name=installArrowClock)]
    fn install_clock() -> Result<JsValue, JsValue>;
}
struct Clock(JsValue);
impl Drop for Clock {
    fn drop(&mut self) {
        call(&self.0, "restore", &[]).unwrap();
    }
}
fn call(object: &JsValue, key: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
    let method = Reflect::get(object, &key.into())?.dyn_into::<Function>()?;
    let values = Array::new();
    for arg in args {
        values.push(arg);
    }
    method.apply(object, &values)
}
fn options(action: Rc<dyn Fn() -> Result<(), JsValue>>) -> ArrowOptions {
    ArrowOptions {
        class_name: "test-arrow".into(),
        bg_width: 10.0.into(),
        bg_height: 20.0.into(),
        top: Some(0.0.into()),
        left: Some(0.0.into()),
        bottom: None,
        right: None,
        handle_activate: action,
    }
}
fn down() -> PointerEvent {
    let init = PointerEventInit::new();
    init.set_pointer_id(999);
    init.set_button(0);
    init.set_buttons(1);
    init.set_cancelable(true);
    PointerEvent::new_with_event_init_dict("pointerdown", &init).unwrap()
}
#[wasm_bindgen_test]
fn background_and_icon_use_default_hint_but_math_uses_number_hint() {
    let log = Array::new();
    let object = Function::new_with_args(
        "log",
        "return{[Symbol.toPrimitive](hint){log.push(hint);return hint==='number'?12:16;}};",
    )
    .call1(&JsValue::UNDEFINED, &log)
    .unwrap();
    let value = xterm_web::terminal::options::from_js(&object).unwrap();
    let mut o = options(Rc::new(|| Ok(())));
    o.bg_width = value.clone();
    o.bg_height = value.clone();
    o.top = Some(value);
    let arrow = ScrollbarArrow::new(web_sys::window().unwrap(), o).unwrap();
    assert_eq!(
        log.iter()
            .map(|v| v.as_string().unwrap())
            .collect::<Vec<_>>(),
        ["default", "default", "number", "number", "default"]
    );
    assert_eq!(
        arrow
            .bg_dom_node
            .style()
            .get_property_value("width")
            .unwrap(),
        "16px"
    );
    assert_eq!(
        arrow.dom_node.style().get_property_value("width").unwrap(),
        "12px"
    );
    assert_eq!(
        arrow.bg_dom_node.style().get_property_value("top").unwrap(),
        "0px"
    );
    assert_eq!(
        arrow.dom_node.style().get_property_value("top").unwrap(),
        "16px"
    );
    arrow.dispose().unwrap();
}
#[wasm_bindgen_test]
fn activation_is_immediate_then_200ms_then_24hz_and_pointerup_cancels() {
    let clock = Clock(install_clock().unwrap());
    let count = Rc::new(Cell::new(0));
    let observed = count.clone();
    let arrow = ScrollbarArrow::new(
        web_sys::window().unwrap(),
        options(Rc::new(move || {
            observed.set(observed.get() + 1);
            Ok(())
        })),
    )
    .unwrap();
    let event = down();
    arrow.dom_node.dispatch_event(event.as_ref()).unwrap();
    assert!(event.default_prevented());
    assert_eq!(count.get(), 1);
    assert_eq!(
        Array::from(&call(&clock.0, "counts", &[]).unwrap()).to_vec(),
        [JsValue::from(1), JsValue::from(0)]
    );
    call(&clock.0, "step", &[200.0.into()]).unwrap();
    assert_eq!(count.get(), 1);
    assert_eq!(
        call(&clock.0, "delay", &[]).unwrap().as_f64(),
        Some(1000.0 / 24.0)
    );
    assert_eq!(
        call(&clock.0, "intervalReceiver", &[]).unwrap(),
        JsValue::from(web_sys::window().unwrap())
    );
    call(&clock.0, "tick", &[]).unwrap();
    assert_eq!(count.get(), 2);
    web_sys::window()
        .unwrap()
        .dispatch_event(PointerEvent::new("pointerup").unwrap().as_ref())
        .unwrap();
    assert_eq!(
        Array::from(&call(&clock.0, "counts", &[]).unwrap()).to_vec(),
        [JsValue::from(0), JsValue::from(0)]
    );
    arrow.dispose().unwrap();
}
#[wasm_bindgen_test]
fn retained_interval_callback_keeps_its_source_runner_after_cancel() {
    let clock = Clock(install_clock().unwrap());
    let count = Rc::new(Cell::new(0));
    let observed = count.clone();
    let arrow = ScrollbarArrow::new(
        web_sys::window().unwrap(),
        options(Rc::new(move || {
            observed.set(observed.get() + 1);
            Ok(())
        })),
    )
    .unwrap();
    arrow.dom_node.dispatch_event(down().as_ref()).unwrap();
    call(&clock.0, "step", &[200.0.into()]).unwrap();
    web_sys::window()
        .unwrap()
        .dispatch_event(PointerEvent::new("pointerup").unwrap().as_ref())
        .unwrap();
    // Clearing a host interval removes scheduling, not a function retained by
    // an intercepting host. The source closure still owns that specific runner.
    call(&clock.0, "tick", &[]).unwrap();
    assert_eq!(count.get(), 2);
    arrow.dispose().unwrap();
}
