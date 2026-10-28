// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
use super::{number, GestureContext, Inertia, CHANGE};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsValue;
use web_sys::EventTarget;

pub(super) fn schedule(
    context: &Rc<GestureContext>,
    targets: Rc<Vec<EventTarget>>,
    state: Rc<RefCell<Inertia>>,
) -> Result<(), JsValue> {
    let weak = Rc::downgrade(context);
    let item = context.animation_queue.schedule(0.0, move |_| {
        if let Some(context) = weak.upgrade() {
            let now = js_sys::Date::now();
            let frame = state.borrow_mut().step(now);
            let event = context.event(CHANGE, None)?;
            number(&event, "translationX", frame.translation_x)?;
            number(&event, "translationY", frame.translation_y)?;
            // Inertia bypasses routing, ignored targets and tap timestamp state.
            // The original insertion-order target snapshot survives token removal.
            for target in targets.iter() {
                target.dispatch_event(&event)?;
            }
            if frame.continues {
                context.inertia(Rc::clone(&targets), Rc::clone(&state))?;
            }
        }
        Ok(())
    })?;
    // Dropping a queue-item handle does not cancel it. This deliberately matches
    // source assignment when a nested callback scheduled another pending frame.
    *context.frame.borrow_mut() = Some(item);
    Ok(())
}
