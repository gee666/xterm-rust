// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    type DragWindow;
    // web-sys's clearInterval import cannot catch a host exception. Preserve
    // the original thrown value and timer slot when the host call fails.
    #[wasm_bindgen::prelude::wasm_bindgen(catch, method, js_name = clearInterval)]
    fn clear_drag_interval(this: &DragWindow, timer: JsValue) -> Result<(), JsValue>;
}

pub(super) fn frame(state: &Rc<State>) -> Result<(), JsValue> {
    if state.disposed.get() || state.raf_id.get().is_some() {
        return Ok(());
    }
    if state.raf_callback.borrow().is_none() {
        let weak = Rc::downgrade(state);
        let callback = Closure::wrap(Box::new(move |_: f64| {
            if let Some(state) = weak.upgrade() {
                if state.disposed.get() {
                    return;
                }
                state.raf_id.set(None);
                let result = state.dimensions().and_then(|dimensions| {
                    state.selection.flush_refresh(dimensions).map_err(error)
                });
                if let Err(error) = result {
                    state.host.report_error(error);
                }
            }
        }) as Box<dyn FnMut(f64)>);
        *state.raf_callback.borrow_mut() = Some(callback);
    }
    let callback = state.raf_callback.borrow();
    let callback = callback
        .as_ref()
        .ok_or_else(|| error("missing selection animation callback"))?;
    state.raf_id.set(Some(
        state
            .window
            .request_animation_frame(callback.as_ref().unchecked_ref())?,
    ));
    Ok(())
}
pub(super) fn drag_callbacks(weak: &std::rc::Weak<State>) -> [js_sys::Function; 2] {
    [true, false].map(|move_event| {
        let weak = weak.clone();
        let callback = Closure::wrap(Box::new(move |event: web_sys::Event| {
            if let Some(state) = weak.upgrade() {
                if state.disposed.get() {
                    return;
                }
                match event.dyn_into::<MouseEvent>() {
                    Ok(event) => {
                        let result = if move_event {
                            events::mouse_move(&state, &event)
                        } else {
                            events::mouse_up(&state, &event)
                        };
                        if let Err(error) = result {
                            state.host.report_error(error);
                        }
                    }
                    Err(_) => state
                        .host
                        .report_error(error("selection requires MouseEvent")),
                }
            }
        }) as Box<dyn Fn(web_sys::Event)>);
        // The document may retain a callback after a failed removal. Transfer
        // ownership to JS and capture only Weak<State>, as DomListener does.
        callback.into_js_value().unchecked_into()
    })
}
pub(super) fn stop_drag(state: &State) -> Result<(), JsValue> {
    // Stable callbacks survive successful removal, failure and recursive calls.
    // Clone before entering the host, and do not clear the timer until it returns.
    let callbacks = state.drag_callbacks.clone();
    if let Some(document) = state.screen.owner_document() {
        for (event_type, callback) in ["mousemove", "mouseup"].into_iter().zip(callbacks) {
            document.remove_event_listener_with_callback(event_type, &callback)?;
        }
    }
    let timer = state
        .drag_id
        .get()
        .map_or(JsValue::UNDEFINED, JsValue::from);
    state
        .window
        .unchecked_ref::<DragWindow>()
        .clear_drag_interval(timer)?;
    state.drag_id.set(None);
    // Keep the interval closure alive: teardown may be called from its callback.
    Ok(())
}
pub(super) fn drag(state: &Rc<State>) -> Result<(), JsValue> {
    state.stop_drag()?;
    let callbacks = state.drag_callbacks.clone();
    if let Some(document) = state.screen.owner_document() {
        for (event_type, callback) in ["mousemove", "mouseup"].into_iter().zip(callbacks) {
            document.add_event_listener_with_callback(event_type, &callback)?;
        }
    }
    if state.drag_callback.borrow().is_none() {
        let weak = Rc::downgrade(state);
        let callback = Closure::wrap(Box::new(move || {
            if let Some(state) = weak.upgrade() {
                if state.disposed.get() {
                    return;
                }
                let result = drag_tick(&state);
                if let Err(error) = result {
                    state.host.report_error(error);
                }
            }
        }) as Box<dyn FnMut()>);
        *state.drag_callback.borrow_mut() = Some(callback);
    }
    let callback = state.drag_callback.borrow();
    let callback = callback
        .as_ref()
        .ok_or_else(|| error("missing selection drag callback"))?;
    let timer = state
        .window
        .set_interval_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            pointer::DRAG_SCROLL_INTERVAL_MS,
        )?;
    state.drag_id.set(Some(timer));
    Ok(())
}
fn drag_tick(state: &Rc<State>) -> Result<(), JsValue> {
    let model = state.selection.model();
    if model.selection_start.is_none() || model.selection_end.is_none() {
        return Ok(());
    }
    let amount = state.drag_amount.get();
    if amount == 0.0 {
        return Ok(());
    }
    if !amount.is_finite() || amount.fract() != 0.0 || amount.abs() > pointer::DRAG_SCROLL_MAX_SPEED
    {
        return Err(error("invalid selection drag scroll amount"));
    }
    state.host.scroll_lines(amount as i64, false)?;
    let dimensions = state.dimensions()?;
    state
        .selection
        .update_after_drag_scroll(&state.buffers, dimensions, amount)
        .map_err(error)
}
