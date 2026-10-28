// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use crate::input::{
    mouse_policy::{MouseInput, MouseRouteError, PointerEventKind, PointerInput},
    mouse_wheel::{cursor_sequence, WheelInput},
};
use web_sys::Event;
use xterm_core::mouse_state::{MouseAction, MouseButton, MouseEvent as CoreMouseEvent};

pub(super) fn bind_fixed(bindings: &mut MouseBindings) -> Result<(), JsValue> {
    let state = &bindings.state;
    bindings.fixed.push(listen(
        state,
        state.element.as_ref(),
        "mousedown",
        false,
        down,
    )?);
    bindings.fixed.push(listen(
        state,
        state.element.as_ref(),
        "wheel",
        true,
        passive_wheel,
    )?);
    bindings.gesture = Some(state.host.mouse_gesture_target(&state.screen)?);
    bindings.fixed.push(listen(
        state,
        state.screen.as_ref(),
        "-xterm-gesturestart",
        false,
        |state, _| {
            state.policy.touch_start();
            Ok(())
        },
    )?);
    bindings.fixed.push(listen(
        state,
        state.screen.as_ref(),
        "-xterm-gesturechange",
        false,
        touch,
    )?);
    Ok(())
}

fn send(state: &State, event: &Event) -> Result<bool, JsValue> {
    let mouse = event
        .dyn_ref::<MouseEvent>()
        .ok_or_else(|| error("Mouse report requires MouseEvent"))?;
    let snapshot = state.host.mouse_snapshot()?;
    let coords = geometry(state, snapshot)?
        .report_coords(f64::from(mouse.client_x()), f64::from(mouse.client_y()));
    if coords.is_none() {
        return Ok(false);
    }
    let override_type =
        js_sys::Reflect::get(event, &JsValue::from_str("overrideType"))?.as_string();
    let kind = PointerEventKind::from_names(&event.type_(), override_type.as_deref());
    let wheel = event.dyn_ref::<WheelEvent>();
    if kind == PointerEventKind::Wheel {
        let wheel = wheel.ok_or_else(|| error("Wheel report requires WheelEvent"))?;
        if !state.host.allow_custom_wheel(wheel)? {
            return Ok(false);
        }
    }
    let pointer = PointerInput {
        kind,
        buttons: js_sys::Reflect::get(event, &JsValue::from_str("buttons"))?
            .as_f64()
            .map(|_| mouse.buttons()),
        button: js_sys::Reflect::get(event, &JsValue::from_str("button"))?
            .as_f64()
            .map(|_| mouse.button()),
        ctrl: mouse.ctrl_key(),
        alt: mouse.alt_key(),
        shift: mouse.shift_key(),
        delta_y: wheel.map_or(0.0, WheelEvent::delta_y),
        delta_mode: wheel.map_or(0.0, |event| f64::from(event.delta_mode())),
    };
    state
        .policy
        .route(
            &state.service,
            MouseInput {
                pointer,
                coords,
                wheel_payload: wheel.map(|_| &()),
            },
            state.host.mouse_snapshot()?.routing,
            |output| state.host.mouse_output(output),
        )
        .map_err(|failure| match failure {
            MouseRouteError::MissingWheelPayload => error("Wheel route lost its event payload"),
            MouseRouteError::Output(error) => error,
        })
}

fn down(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    event.prevent_default();
    state.host.mouse_focus()?;
    let mouse = event
        .dyn_ref::<MouseEvent>()
        .ok_or_else(|| error("mousedown requires MouseEvent"))?;
    if !state.service.are_mouse_events_active() || state.host.should_force_selection(mouse)? {
        return Ok(());
    }
    send(state, &event)?;
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
    send(state, &event)?;
    let mouse = event
        .dyn_ref::<MouseEvent>()
        .ok_or_else(|| error("mouseup requires MouseEvent"))?;
    if mouse.buttons() == 0 {
        clear_globals(state)?;
    }
    Ok(())
}

fn drag(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    let mouse = event
        .dyn_ref::<MouseEvent>()
        .ok_or_else(|| error("mousemove requires MouseEvent"))?;
    if mouse.buttons() != 0 {
        send(state, &event)?;
    }
    Ok(())
}

pub(super) fn movement(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    let mouse = event
        .dyn_ref::<MouseEvent>()
        .ok_or_else(|| error("mousemove requires MouseEvent"))?;
    if mouse.buttons() == 0 {
        send(state, &event)?;
    }
    Ok(())
}

pub(super) fn active_wheel(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    send(state, &event)?;
    event.prevent_default();
    event.stop_propagation();
    Ok(())
}

fn passive_wheel(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    if state.requested.get().0 & MouseEventTypes::WHEEL.0 != 0 {
        return Ok(());
    }
    let wheel = event
        .dyn_ref::<WheelEvent>()
        .ok_or_else(|| error("wheel requires WheelEvent"))?;
    if !state.host.allow_custom_wheel(wheel)? {
        return Ok(());
    }
    let snapshot = state.host.mouse_snapshot()?;
    if snapshot.has_scrollback || wheel.delta_y() == 0.0 {
        return Ok(());
    }
    let lines = state.policy.consume_wheel(
        WheelInput {
            delta_y: wheel.delta_y(),
            delta_mode: f64::from(wheel.delta_mode()),
            ctrl: wheel.ctrl_key(),
            alt: wheel.alt_key(),
            shift: wheel.shift_key(),
        },
        snapshot.routing,
    );
    if lines != 0.0 {
        state.host.mouse_output(MouseOutput::UserData(
            cursor_sequence(wheel.delta_y() < 0.0, snapshot.application_cursor).to_vec(),
        ))?;
    }
    event.prevent_default();
    event.stop_propagation();
    Ok(())
}

fn touch(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    event.prevent_default();
    event.stop_propagation();
    // A missing synthetic gesture property behaves like undefined arithmetic in
    // JavaScript. The real recognizer supplies finite client/translation values.
    let number = |name| -> Result<f64, JsValue> {
        Ok(js_sys::Reflect::get(&event, &JsValue::from_str(name))?
            .as_f64()
            .unwrap_or(f64::NAN))
    };
    let translation = number("translationY")?;
    let snapshot = state.host.mouse_snapshot()?;
    let wheel = state.requested.get().0 & MouseEventTypes::WHEEL.0 != 0;
    if !wheel && snapshot.has_scrollback {
        return state.host.mouse_touch_scroll(translation);
    }
    let lines = state
        .policy
        .consume_touch(translation, Some(snapshot.css_cell_height));
    if lines == 0.0 || lines.is_nan() {
        return Ok(());
    }
    if !lines.is_finite() || lines.abs() > usize::MAX as f64 {
        return Err(error("Gesture produces an unrepresentable line count"));
    }
    let coords = if wheel {
        geometry(state, snapshot)?.report_coords(number("clientX")?, number("clientY")?)
    } else {
        None
    };
    if wheel && coords.is_none() {
        return Ok(());
    }
    let sequence = cursor_sequence(lines < 0.0, snapshot.application_cursor);
    for _ in 0..lines.abs() as usize {
        if let Some(coords) = coords {
            let live = state.host.mouse_snapshot()?;
            let mut report = CoreMouseEvent {
                col: coords.col,
                row: coords.row,
                x: coords.x,
                y: coords.y,
                button: MouseButton::Wheel,
                action: if lines < 0.0 {
                    MouseAction::Up
                } else {
                    MouseAction::Down
                },
                ctrl: Some(false),
                alt: Some(false),
                shift: Some(false),
            };
            state.policy.trigger(
                &state.service,
                &mut report,
                live.routing.cols,
                live.routing.rows,
                |output| state.host.mouse_output(output),
            )?;
        } else {
            state
                .host
                .mouse_output(MouseOutput::UserData(sequence.to_vec()))?;
        }
    }
    Ok(())
}
