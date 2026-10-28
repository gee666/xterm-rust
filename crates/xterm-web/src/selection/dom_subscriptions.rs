// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use crate::js_error::callback_error;

fn listen<T: 'static>(
    state: &Rc<State>,
    event: Event<T>,
    callback: impl Fn(&Rc<State>, &T) -> Result<(), JsValue> + 'static,
) {
    let weak = Rc::downgrade(state);
    let subscription = event.try_subscribe(move |event| {
        if let Some(state) = weak.upgrade() {
            if !state.disposed.get() {
                callback(&state, event).map_err(callback_error)?;
            }
        }
        Ok(())
    });
    state.subscriptions.borrow_mut().push(subscription);
}
pub(super) fn bind(state: &Rc<State>) -> Result<(), JsValue> {
    listen(state, state.selection.on_request_refresh(), |state, _| {
        schedule::frame(state)
    });
    listen(state, state.selection.on_stop_drag(), |state, _| {
        state.stop_drag()
    });
    listen(
        state,
        state.selection.on_request_redraw(),
        |state, event| state.host.redraw(*event),
    );
    listen(state, state.selection.on_selection_change(), |state, _| {
        state.host.selection_changed()
    });
    listen(
        state,
        state.selection.on_linux_mouse_selection(),
        |state, text| state.host.linux_mouse_selection(text),
    );
    listen(state, state.host.on_user_input()?, |state, _| {
        let dimensions = state.dimensions()?;
        if state
            .selection
            .has_selection(dimensions)
            .map_err(selection_error)?
        {
            state
                .selection
                .clear_selection(&state.buffers, dimensions)
                .map_err(selection_error)?;
        }
        Ok(())
    });
    listen(state, state.buffers.on_resize(), |state, event| {
        if event.rows_changed {
            state
                .selection
                .clear_selection(&state.buffers, state.coordinate_dimensions(0)?)
                .map_err(selection_error)?;
        }
        Ok(())
    });
    listen(
        state,
        state.buffers.buffers.on_buffer_activate(),
        |state, _| {
            state
                .selection
                .clear_selection(&state.buffers, state.coordinate_dimensions(0)?)
                .map_err(selection_error)?;
            bind_trim(state)
        },
    );
    bind_trim(state)
}
fn bind_trim(state: &Rc<State>) -> Result<(), JsValue> {
    if let Some(previous) = state.trim_subscription.borrow_mut().take() {
        previous.dispose();
    }
    let active = state.buffers.buffer();
    let event = active
        .try_borrow()
        .map_err(error)?
        .lines
        .on_trim_emitter
        .event();
    let weak = Rc::downgrade(state);
    let subscription = event.try_subscribe(move |amount| {
        if let Some(state) = weak.upgrade() {
            if !state.disposed.get() {
                // Buffer trims can fire while its mutable borrow is held. No
                // live row query is needed until the scheduled redraw frame.
                let result = state.coordinate_dimensions(0).and_then(|dimensions| {
                    let amount = u32::try_from(*amount).map_err(error)?;
                    state
                        .selection
                        .handle_trim(&state.buffers, dimensions, amount)
                        .map_err(selection_error)
                });
                result.map_err(callback_error)?;
            }
        }
        Ok(())
    });
    *state.trim_subscription.borrow_mut() = Some(subscription);
    Ok(())
}
