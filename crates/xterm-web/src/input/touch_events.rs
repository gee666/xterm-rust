// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
use super::{number, reads, set, GestureContext, CHANGE, CONTEXT_MENU, END, START, TAP};
use crate::input::touch::{EndGesture, TouchData};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsValue;
use web_sys::Event;

pub(super) fn start(context: &Rc<GestureContext>, event: Event) -> Result<(), JsValue> {
    let timestamp = reads::now()?;
    context.cancel_frame();
    let length = reads::length(event.as_ref(), "targetTouches")?;
    let mut index = 0.0;
    while index < length {
        let touch = reads::item(event.as_ref(), "targetTouches", index)?;
        let key = reads::identifier(&touch)?;
        // Source stores id in the new object, even though no handler later uses
        // that field. This separate getter can reenter or fail before insertion.
        reads::identifier(&touch)?;
        let target = reads::field(&touch, "target")?;
        let initial_x = reads::number(&touch, "pageX")?;
        let initial_y = reads::number(&touch, "pageY")?;
        let rolling_x = reads::number(&touch, "pageX")?;
        let rolling_y = reads::number(&touch, "pageY")?;
        let data = TouchData::from_snapshots(
            target,
            timestamp,
            (initial_x, initial_y),
            (rolling_x, rolling_y),
        );
        context
            .active
            .borrow_mut()
            .insert(key, Rc::new(RefCell::new(data)));
        let target = reads::field(&touch, "target")?;
        let gesture = context.event(START, Some(&target))?;
        position(&gesture, &touch)?;
        context.dispatch(&gesture)?;
        index += 1.0;
    }
    context.finish_batch(&event)
}

pub(super) fn movement(context: &Rc<GestureContext>, event: Event) -> Result<(), JsValue> {
    let timestamp = reads::now()?;
    let length = reads::length(event.as_ref(), "changedTouches")?;
    let mut index = 0.0;
    while index < length {
        let touch = reads::item(event.as_ref(), "changedTouches", index)?;
        if !has_touch(context, &touch)? {
            reads::warn("end of an UNKNOWN touch", &touch)?;
            index += 1.0;
            continue;
        }
        let key = reads::identifier(&touch)?;
        let data = context.active.borrow().get(&key).cloned();
        let data = data.ok_or_else(|| reads::missing_record(false))?;
        let target = data.borrow().initial_target.clone();
        let gesture = context.event(CHANGE, Some(&target))?;
        let x = reads::number(&touch, "pageX")?;
        let previous_x = data.borrow().last_x().unwrap_or(f64::NAN);
        number(&gesture, "translationX", x - previous_x)?;
        let y = reads::number(&touch, "pageY")?;
        let previous_y = data.borrow().last_y().unwrap_or(f64::NAN);
        number(&gesture, "translationY", y - previous_y)?;
        position(&gesture, &touch)?;
        let client_x = reads::field(&touch, "clientX")?;
        set(&gesture, "clientX", &client_x)?;
        let client_y = reads::field(&touch, "clientY")?;
        set(&gesture, "clientY", &client_y)?;
        context.dispatch(&gesture)?;
        // Each push follows its own getter. Nested dispatch may leave the three
        // source arrays unequal; never replace this with one cached tuple commit.
        data.borrow_mut().begin_commit();
        let x = reads::number(&touch, "pageX")?;
        data.borrow_mut().push_x(x);
        let y = reads::number(&touch, "pageY")?;
        data.borrow_mut().push_y(y);
        data.borrow_mut().push_time(timestamp);
        index += 1.0;
    }
    context.finish_batch(&event)
}

pub(super) fn end(context: &Rc<GestureContext>, event: Event) -> Result<(), JsValue> {
    let timestamp = reads::now()?;
    let active_count = context.active.borrow().len();
    let length = reads::length(event.as_ref(), "changedTouches")?;
    let mut index = 0.0;
    while index < length {
        let touch = reads::item(event.as_ref(), "changedTouches", index)?;
        if !has_touch(context, &touch)? {
            reads::warn("move of an UNKNOWN touch", &touch)?;
            index += 1.0;
            continue;
        }
        let key = reads::identifier(&touch)?;
        let data = context.active.borrow().get(&key).cloned();
        let hold_now = reads::now()?;
        let data = data.ok_or_else(|| reads::missing_record(true))?;
        let action = data.borrow().end(hold_now, timestamp);
        match action {
            EndGesture::Tap | EndGesture::ContextMenu => {
                let kind = if matches!(action, EndGesture::Tap) {
                    TAP
                } else {
                    CONTEXT_MENU
                };
                let target = data.borrow().initial_target.clone();
                let gesture = context.event(kind, Some(&target))?;
                let x = data.borrow().last_x();
                optional_number(&gesture, "pageX", x)?;
                let y = data.borrow().last_y();
                optional_number(&gesture, "pageY", y)?;
                context.dispatch(&gesture)?;
            }
            EndGesture::Inertia(inertia) if active_count == 1 => {
                let target = data.borrow().initial_target.clone();
                let targets = context.inertia_targets(&target)?;
                context.inertia(Rc::new(targets), Rc::new(RefCell::new(inertia)))?;
            }
            EndGesture::Inertia(_) => {}
        }
        let target = data.borrow().initial_target.clone();
        let gesture = context.event(END, Some(&target))?;
        context.dispatch(&gesture)?;
        let key = reads::identifier(&touch)?;
        context.active.borrow_mut().remove(&key);
        index += 1.0;
    }
    context.finish_batch(&event)
}

fn has_touch(context: &GestureContext, touch: &JsValue) -> Result<bool, JsValue> {
    let key = reads::identifier(touch)?;
    Ok(context.active.borrow().contains_key(&key))
}

fn optional_number(event: &Event, name: &str, value: Option<f64>) -> Result<(), JsValue> {
    set(
        event,
        name,
        &value.map_or(JsValue::UNDEFINED, JsValue::from_f64),
    )
}
fn position(event: &Event, touch: &JsValue) -> Result<(), JsValue> {
    let x = reads::field(touch, "pageX")?;
    set(event, "pageX", &x)?;
    let y = reads::field(touch, "pageY")?;
    set(event, "pageY", &y)
}
