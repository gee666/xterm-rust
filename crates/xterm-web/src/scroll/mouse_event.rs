// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Standard mouse metadata. Reads follow the constructor's source order, not a
//! browser-realm instanceof test or an eager snapshot of later-used properties.
pub use super::iframe::IframeCache;
use super::interop::{add, clock, get, method0, nullish, subtract};
use wasm_bindgen::JsValue;
use web_sys::{Event, Window};

pub use super::wheel_event::StandardWheelEvent;

#[derive(Clone, Debug)]
pub struct StandardMouseEvent {
    pub browser_event: Event,
    pub left_button: bool,
    pub middle_button: bool,
    pub right_button: bool,
    pub buttons: JsValue,
    pub target: JsValue,
    pub detail: JsValue,
    pub posx: f64,
    pub posy: f64,
    pub ctrl_key: JsValue,
    pub shift_key: JsValue,
    pub alt_key: JsValue,
    pub meta_key: JsValue,
    pub timestamp: JsValue,
}
impl StandardMouseEvent {
    pub fn new(
        target_window: &Window,
        event: &Event,
        cache: &IframeCache,
    ) -> Result<Self, JsValue> {
        let timestamp = clock()?;
        let left_button = get(event.as_ref(), "button")? == JsValue::from_f64(0.0);
        let middle_button = get(event.as_ref(), "button")? == JsValue::from_f64(1.0);
        let right_button = get(event.as_ref(), "button")? == JsValue::from_f64(2.0);
        let buttons = get(event.as_ref(), "buttons")?;
        let target = get(event.as_ref(), "target")?;
        let raw_detail = get(event.as_ref(), "detail")?;
        let mut detail = if nullish(&raw_detail) {
            1.0.into()
        } else {
            raw_detail
        };
        if get(event.as_ref(), "type")? == JsValue::from_str("dblclick") {
            detail = 2.0.into();
        }
        let ctrl_key = get(event.as_ref(), "ctrlKey")?;
        let shift_key = get(event.as_ref(), "shiftKey")?;
        let alt_key = get(event.as_ref(), "altKey")?;
        let meta_key = get(event.as_ref(), "metaKey")?;
        let (x, y) = if get(event.as_ref(), "pageX")?.as_f64().is_some() {
            (get(event.as_ref(), "pageX")?, get(event.as_ref(), "pageY")?)
        } else {
            let x = get(event.as_ref(), "clientX")?;
            let body = get(&get(&target, "ownerDocument")?, "body")?;
            let x = add(&x, &get(&body, "scrollLeft")?)?;
            let html = get(&get(&target, "ownerDocument")?, "documentElement")?;
            let x = add(&x, &get(&html, "scrollLeft")?)?;
            let y = get(event.as_ref(), "clientY")?;
            let body = get(&get(&target, "ownerDocument")?, "body")?;
            let y = add(&y, &get(&body, "scrollTop")?)?;
            let html = get(&get(&target, "ownerDocument")?, "documentElement")?;
            (x, add(&y, &get(&html, "scrollTop")?)?)
        };
        let offsets = cache.position_relative_to(target_window, &get(event.as_ref(), "view")?)?;
        let x = subtract(&x, &offsets.left)?;
        let posx = x
            .as_f64()
            .ok_or_else(|| js_sys::TypeError::new("Mouse x coordinate must be numeric"))?;
        let y = subtract(&y, &offsets.top)?;
        let posy = y
            .as_f64()
            .ok_or_else(|| js_sys::TypeError::new("Mouse y coordinate must be numeric"))?;
        Ok(Self {
            browser_event: event.clone(),
            left_button,
            middle_button,
            right_button,
            buttons,
            target,
            detail,
            posx,
            posy,
            ctrl_key,
            shift_key,
            alt_key,
            meta_key,
            timestamp,
        })
    }
    pub fn prevent_default(&self) -> Result<(), JsValue> {
        method0(self.browser_event.as_ref(), "preventDefault").map(|_| ())
    }
    pub fn stop_propagation(&self) -> Result<(), JsValue> {
        method0(self.browser_event.as_ref(), "stopPropagation").map(|_| ())
    }
}
