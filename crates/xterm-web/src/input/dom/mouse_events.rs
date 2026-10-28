// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use crate::input::mouse_wheel::cursor_sequence;
use web_sys::Event;

pub(super) fn bind_fixed(bindings: &MouseBindings) -> Result<(), JsValue> {
    let state = &bindings.state;
    let down = listen(state, state.element.as_ref(), "mousedown", false, down)?;
    bindings.fixed.borrow_mut().push(down);
    let wheel = listen(state, state.element.as_ref(), "wheel", true, passive_wheel)?;
    bindings.fixed.borrow_mut().push(wheel);
    let gesture = state.host.mouse_gesture_target(&state.screen)?;
    *bindings.gesture.borrow_mut() = Some(gesture);
    let start = listen(
        state,
        state.screen.as_ref(),
        "-xterm-gesturestart",
        false,
        |state, _| {
            state.policy.touch_start();
            Ok(())
        },
    )?;
    bindings.fixed.borrow_mut().push(start);
    let change = listen(
        state,
        state.screen.as_ref(),
        "-xterm-gesturechange",
        false,
        touch::change,
    )?;
    bindings.fixed.borrow_mut().push(change);
    Ok(())
}

fn down(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    values::prevent(&event)?;
    state.host.mouse_focus()?;
    if !state.service.are_mouse_events_active()
        || state.host.should_force_selection(event.unchecked_ref())?
    {
        return Ok(());
    }
    report::send(state, &event)?;
    let document = state
        .element
        .owner_document()
        .unwrap_or_else(|| state.document.clone());
    let requested = state.requested.get();
    if requested.0 & MouseEventTypes::UP.0 != 0 {
        let listener = listen(state, document.as_ref(), "mouseup", false, up)?;
        let old = state.dynamic.borrow_mut().up_listener.replace(listener);
        clear(old)?;
    }
    if requested.0 & MouseEventTypes::DRAG.0 != 0 {
        let listener = listen(state, document.as_ref(), "mousemove", false, drag)?;
        let old = state.dynamic.borrow_mut().drag_listener.replace(listener);
        clear(old)?;
    }
    Ok(())
}

fn up(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    report::send(state, &event)?;
    if !values::truth(&event, "buttons")? {
        clear_globals(state)?;
    }
    Ok(())
}

fn drag(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    if values::truth(&event, "buttons")? {
        report::send(state, &event)?;
    }
    Ok(())
}

pub(super) fn movement(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    if !values::truth(&event, "buttons")? {
        report::send(state, &event)?;
    }
    Ok(())
}

pub(super) fn active_wheel(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    report::send(state, &event)?;
    values::cancel(&event)
}

fn passive_wheel(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    if state.requested.get().0 & MouseEventTypes::WHEEL.0 != 0 {
        return Ok(());
    }
    if !state.host.allow_custom_wheel(event.unchecked_ref())? {
        return Ok(());
    }
    if state.host.mouse_has_scrollback()? {
        return Ok(());
    }
    if values::strict_zero(&values::get(&event, "deltaY")?) {
        return Ok(());
    }
    let height = state.host.mouse_device_cell_height()?;
    let dpr = state.host.mouse_dpr()?;
    if report::consume_wheel(state, &event, height, dpr)? != 0.0 {
        let application = state.host.mouse_application_cursor()?;
        let up = values::less(&values::get(&event, "deltaY")?, 0.0)?;
        state.host.mouse_output(MouseOutput::UserData(
            cursor_sequence(up, application).to_vec(),
        ))?;
    }
    values::cancel(&event)
}
