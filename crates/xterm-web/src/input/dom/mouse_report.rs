// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use crate::input::mouse_wheel::{TRACKPAD_MULTIPLIER, TRACKPAD_THRESHOLD};
use web_sys::Event;
use xterm_core::mouse_state::{MouseAction, MouseButton, MouseEvent as CoreMouseEvent};

fn raw_button(event: &Event) -> Result<Option<MouseButton>, JsValue> {
    if !values::less(&values::get(event, "button")?, 3.0)? {
        return Ok(Some(MouseButton::None));
    }
    let value = values::get(event, "button")?;
    if value.is_undefined() {
        return Ok(None);
    }
    // Do not apply ToInt32 before source range checks or move debounce.
    Ok(Some(MouseButton::RawNumber(values::number_value(&value)?)))
}

pub(super) fn send(state: &State, event: &Event) -> Result<bool, JsValue> {
    let Some(coords) = report_coords(state, event)? else {
        return Ok(false);
    };
    let override_type = values::get(event, "overrideType")?;
    let kind = if override_type.is_truthy() {
        override_type
    } else {
        values::get(event, "type")?
    };
    let (button, action) = match kind.as_string().as_deref() {
        Some("mousemove") => {
            let button = if values::get(event, "buttons")?.is_undefined() {
                if values::get(event, "button")?.is_undefined() {
                    Some(MouseButton::None)
                } else {
                    raw_button(event)?
                }
            } else if values::bit(event, "buttons", 1)? {
                Some(MouseButton::Left)
            } else if values::bit(event, "buttons", 4)? {
                Some(MouseButton::Middle)
            } else if values::bit(event, "buttons", 2)? {
                Some(MouseButton::Right)
            } else {
                Some(MouseButton::None)
            };
            (button, MouseAction::Move)
        }
        Some("mouseup") => (raw_button(event)?, MouseAction::Up),
        Some("mousedown") => (raw_button(event)?, MouseAction::Down),
        Some("wheel") => {
            // Source treats overrideType structurally. The opaque WheelEvent
            // view is only passed to the host; all field reads remain caught.
            if !state.host.allow_custom_wheel(event.unchecked_ref())? {
                return Ok(false);
            }
            let delta = values::get(event, "deltaY")?;
            if values::strict_zero(&delta) {
                return Ok(false);
            }
            let height = state.host.mouse_device_cell_height()?;
            let dpr = state.host.mouse_dpr()?;
            if values::strict_zero(&consume_wheel(state, event, height, dpr)?) {
                return Ok(false);
            }
            (
                Some(MouseButton::Wheel),
                if values::less(&delta, 0.0)? {
                    MouseAction::Up
                } else {
                    MouseAction::Down
                },
            )
        }
        _ => return Ok(false),
    };
    let Some(button) = button else {
        return Ok(false);
    };
    if button.numeric_value() > MouseButton::Wheel.numeric_value() {
        return Ok(false);
    }
    if button != MouseButton::Wheel
        && state.host.mouse_require_alt()?
        && state.service.are_mouse_events_active()
        && !values::truth(event, "altKey")?
    {
        return Ok(false);
    }
    let strip_alt = button != MouseButton::Wheel
        && state.host.mouse_require_alt()?
        && state.service.are_mouse_events_active();
    let mut report = CoreMouseEvent {
        col: coords.col,
        row: coords.row,
        x: coords.x,
        y: coords.y,
        button,
        action,
        ctrl: Some(values::truth(event, "ctrlKey")?),
        alt: Some(!strip_alt && values::truth(event, "altKey")?),
        shift: Some(values::truth(event, "shiftKey")?),
    };
    trigger(state, &mut report)
}

pub(super) fn consume_wheel(
    state: &State,
    event: &Event,
    height: Option<f64>,
    dpr: Option<f64>,
) -> Result<JsValue, JsValue> {
    if values::strict_zero(&values::get(event, "deltaY")?) || values::truth(event, "shiftKey")? {
        return Ok(0.into());
    }
    let (Some(height), Some(dpr)) = (height, dpr) else {
        return Ok(0.into());
    };
    let amount = values::get(event, "deltaY")?;
    let mut amount = if values::truth(event, "altKey")?
        || values::truth(event, "ctrlKey")?
        || values::truth(event, "shiftKey")?
    {
        let fast = state.host.mouse_fast_scroll_sensitivity()?;
        let first = values::multiply(&amount, &fast)?;
        let slow = state.host.mouse_scroll_sensitivity()?;
        values::multiply(&first, &slow)?
    } else {
        let slow = state.host.mouse_scroll_sensitivity()?;
        values::multiply(&amount, &slow)?
    };
    if values::get(event, "deltaMode")?.as_f64() == Some(f64::from(WheelEvent::DOM_DELTA_PIXEL)) {
        // Number division rejects a BigInt before trackpad reads, as source does.
        let divided = values::divide(&amount, &JsValue::from_f64(height / dpr + 0.0))?;
        let mut numeric = divided
            .as_f64()
            .ok_or_else(|| error("Pixel division must produce a number"))?;
        if values::number(event, "deltaY")?.abs() < TRACKPAD_THRESHOLD {
            numeric *= TRACKPAD_MULTIPLIER;
        }
        amount = JsValue::from_f64(state.policy.consume_wheel_pixels(numeric));
    } else if values::get(event, "deltaMode")?.as_f64()
        == Some(f64::from(WheelEvent::DOM_DELTA_PAGE))
    {
        amount = values::multiply(&amount, &JsValue::from_f64(state.host.mouse_rows()?))?;
    }
    Ok(amount)
}

pub(super) fn trigger(state: &State, event: &mut CoreMouseEvent) -> Result<bool, JsValue> {
    // Validate before requesting later bounds, then reuse the shared validator,
    // encoder and cache. The repeated numeric checks have no callback effects.
    if event.col < 0.0 {
        return Ok(false);
    }
    let cols = state.host.mouse_cols()?;
    if event.col >= cols || event.row < 0.0 {
        return Ok(false);
    }
    let rows = state.host.mouse_rows()?;
    state
        .policy
        .trigger(&state.service, event, cols, rows, |output| {
            state.host.mouse_output(output)
        })
}
