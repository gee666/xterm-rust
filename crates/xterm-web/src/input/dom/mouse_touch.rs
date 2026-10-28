// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use crate::input::mouse_wheel::cursor_sequence;
use web_sys::Event;
use xterm_core::mouse_state::{MouseAction, MouseButton, MouseEvent as CoreMouseEvent};

pub(super) fn change(state: &Rc<State>, event: Event) -> Result<(), JsValue> {
    values::cancel(&event)?;
    let wheel = state.requested.get().0 & MouseEventTypes::WHEEL.0 != 0;
    if !wheel && state.host.mouse_has_scrollback()? {
        return state
            .host
            .mouse_touch_scroll(values::number(&event, "translationY")?);
    }
    let Some(height) = state.host.mouse_css_cell_height()? else {
        return Ok(());
    };
    if height == 0.0 || height.is_nan() {
        return Ok(());
    }
    let translation = values::number(&event, "translationY")?;
    let lines = state.policy.consume_touch(translation, Some(height));
    if lines == 0.0 {
        return Ok(());
    }
    if wheel {
        let Some(coords) = report_coords(state, &event)? else {
            return Ok(());
        };
        for _ in 0..line_count(lines)? {
            let mut event = CoreMouseEvent {
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
            report::trigger(state, &mut event)?;
        }
    } else {
        let application = state.host.mouse_application_cursor()?;
        let sequence = cursor_sequence(lines < 0.0, application);
        for _ in 0..line_count(lines)? {
            state
                .host
                .mouse_output(MouseOutput::UserData(sequence.to_vec()))?;
        }
    }
    Ok(())
}

fn line_count(lines: f64) -> Result<usize, JsValue> {
    // The source's i < Math.abs(NaN) loop has zero iterations. Infinity would
    // loop forever; reject that synthetic boundary rather than hanging the UI.
    if lines.is_nan() {
        return Ok(0);
    }
    if !lines.is_finite() || lines.abs() > usize::MAX as f64 {
        return Err(error("Gesture produces an unrepresentable line count"));
    }
    Ok(lines.abs() as usize)
}
