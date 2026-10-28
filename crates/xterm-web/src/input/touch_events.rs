// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
use super::{error, number, GestureContext, CHANGE, CONTEXT_MENU, END, START, TAP};
use crate::input::touch::{EndGesture, TouchData};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Event, Touch, TouchEvent};

pub(super) fn start(context: &Rc<GestureContext>, event: Event) -> Result<(), JsValue> {
    let touch_event = touch_event(&event);
    let timestamp = js_sys::Date::now();
    context.cancel_frame();
    let touches = touch_event.target_touches();
    for index in 0..touches.length() {
        let touch = touches
            .item(index)
            .ok_or_else(|| error("Missing target touch"))?;
        let target = touch.target();
        let data = TouchData::new(
            target.clone(),
            timestamp,
            coordinate(&touch, "pageX")?,
            coordinate(&touch, "pageY")?,
        );
        context
            .active
            .borrow_mut()
            .insert(touch.identifier(), Rc::new(RefCell::new(data)));
        let gesture = context.event_for_touch(START, target.as_ref())?;
        position(&gesture, &touch)?;
        context.dispatch(&gesture, target.as_ref())?;
    }
    context.finish_batch(&event);
    Ok(())
}

pub(super) fn movement(context: &Rc<GestureContext>, event: Event) -> Result<(), JsValue> {
    let touch_event = touch_event(&event);
    let timestamp = js_sys::Date::now();
    let touches = touch_event.changed_touches();
    for index in 0..touches.length() {
        let touch = touches
            .item(index)
            .ok_or_else(|| error("Missing changed touch"))?;
        let data = context.active.borrow().get(&touch.identifier()).cloned();
        let Some(data) = data else {
            // The original warning names are reversed; keep the observable text.
            web_sys::console::warn_2(&"end of an UNKNOWN touch".into(), touch.as_ref());
            continue;
        };
        let x = coordinate(&touch, "pageX")?;
        let y = coordinate(&touch, "pageY")?;
        let (target, (dx, dy)) = {
            let data = data.borrow();
            (data.initial_target.clone(), data.translation(x, y))
        };
        let gesture = context.event_for_touch(CHANGE, target.as_ref())?;
        number(&gesture, "translationX", dx)?;
        number(&gesture, "translationY", dy)?;
        position(&gesture, &touch)?;
        number(&gesture, "clientX", coordinate(&touch, "clientX")?)?;
        number(&gesture, "clientY", coordinate(&touch, "clientY")?)?;
        context.dispatch(&gesture, target.as_ref())?;
        // Dispatch may replace the map record, but source mutates the captured
        // old object after callbacks. Never look it up again here.
        data.borrow_mut().commit_move(timestamp, x, y);
    }
    context.finish_batch(&event);
    Ok(())
}

pub(super) fn end(context: &Rc<GestureContext>, event: Event) -> Result<(), JsValue> {
    let touch_event = touch_event(&event);
    let timestamp = js_sys::Date::now();
    let active_count = context.active.borrow().len();
    let touches = touch_event.changed_touches();
    for index in 0..touches.length() {
        let touch = touches
            .item(index)
            .ok_or_else(|| error("Missing ended touch"))?;
        let data = context.active.borrow().get(&touch.identifier()).cloned();
        let Some(data) = data else {
            web_sys::console::warn_2(&"move of an UNKNOWN touch".into(), touch.as_ref());
            continue;
        };
        let hold_now = js_sys::Date::now();
        let (target, position, action) = {
            let data = data.borrow();
            (
                data.initial_target.clone(),
                data.last(),
                data.end(hold_now, timestamp),
            )
        };
        match action {
            EndGesture::Tap | EndGesture::ContextMenu => {
                let kind = if matches!(action, EndGesture::Tap) {
                    TAP
                } else {
                    CONTEXT_MENU
                };
                let gesture = context.event_for_touch(kind, target.as_ref())?;
                number(&gesture, "pageX", position.0)?;
                number(&gesture, "pageY", position.1)?;
                context.dispatch(&gesture, target.as_ref())?;
            }
            EndGesture::Inertia(inertia) if active_count == 1 => {
                context.inertia(
                    Rc::new(context.inertia_targets(target.as_ref())),
                    Rc::new(RefCell::new(inertia)),
                )?;
            }
            EndGesture::Inertia(_) => {}
        }
        let gesture = context.event_for_touch(END, target.as_ref())?;
        context.dispatch(&gesture, target.as_ref())?;
        // A callback may have inserted a new record with this ID. The original
        // deletes the current property after END rather than only the old record.
        context.active.borrow_mut().remove(&touch.identifier());
    }
    context.finish_batch(&event);
    Ok(())
}

fn touch_event(event: &Event) -> TouchEvent {
    // Upstream reads structural touch-list properties, without an instanceof
    // check. The generated getters read those same JS properties and propagate
    // host errors for malformed lists rather than imposing a new event brand.
    event.clone().unchecked_into()
}

fn position(event: &Event, touch: &Touch) -> Result<(), JsValue> {
    number(event, "pageX", coordinate(touch, "pageX")?)?;
    number(event, "pageY", coordinate(touch, "pageY")?)
}

fn coordinate(touch: &Touch, name: &str) -> Result<f64, JsValue> {
    // Structural source events may expose fractional coordinates even when the
    // generated Touch IDL getter is an integer. Preserve the actual JS number.
    js_sys::Reflect::get(touch.as_ref(), &name.into())?
        .as_f64()
        .ok_or_else(|| error("Touch coordinate is not a number"))
}
