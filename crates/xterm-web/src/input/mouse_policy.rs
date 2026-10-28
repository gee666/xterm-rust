// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
//! Browser mouse report routing. DOM listener ownership remains in the adapter.
use super::mouse_coords::ReportCoords;
use super::mouse_wheel::{WheelInput, WheelOptions, WheelState};
use std::cell::RefCell;
use std::fmt;
use xterm_core::mouse_state::{MouseAction, MouseButton, MouseEvent, MouseStateService};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerEventKind {
    Move,
    Up,
    Down,
    Wheel,
    Other,
}
impl PointerEventKind {
    pub fn from_names(event_type: &str, override_type: Option<&str>) -> Self {
        let name = override_type
            .filter(|name| !name.is_empty())
            .unwrap_or(event_type);
        match name {
            "mousemove" => Self::Move,
            "mouseup" => Self::Up,
            "mousedown" => Self::Down,
            "wheel" => Self::Wheel,
            _ => Self::Other,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerInput {
    pub kind: PointerEventKind,
    pub buttons: Option<u16>,
    pub button: Option<i16>,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub delta_y: f64,
    pub delta_mode: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MouseRouting {
    pub cols: f64,
    pub rows: f64,
    pub require_alt: bool,
    pub device_cell_height: Option<f64>,
    pub dpr: Option<f64>,
    pub wheel_options: WheelOptions,
}

pub struct MouseInput<'a, W> {
    pub pointer: PointerInput,
    pub coords: Option<ReportCoords>,
    pub wheel_payload: Option<&'a W>,
}

/// Default encoding goes through onBinary; all other encodings through onData
/// with wasUserInput=true, retaining the terminal's distinct stdin policies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MouseOutput {
    Binary(Vec<u16>),
    UserData(Vec<u16>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum MouseRouteError<E> {
    MissingWheelPayload,
    Output(E),
}
impl<E: fmt::Display> fmt::Display for MouseRouteError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingWheelPayload => {
                f.write_str("wheel routing requires the original wheel event")
            }
            Self::Output(error) => error.fmt(f),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for MouseRouteError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Output(error) => Some(error),
            Self::MissingWheelPayload => None,
        }
    }
}

/// State borrows never cross custom restrictions, encoders or output callbacks.
/// Reentrant reporting observes the prior last event; the outer call commits its
/// own event after output, matching upstream's callback ordering.
#[derive(Default)]
pub struct MousePolicy {
    last_event: RefCell<Option<MouseEvent>>,
    wheel: RefCell<WheelState>,
}

impl MousePolicy {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn reset(&self) {
        *self.last_event.borrow_mut() = None;
        self.wheel.borrow_mut().reset();
    }
    pub fn wheel_state(&self) -> WheelState {
        *self.wheel.borrow()
    }
    pub fn touch_start(&self) {
        self.wheel.borrow_mut().touch_start();
    }
    pub fn consume_touch(&self, translation_y: f64, cell_height: Option<f64>) -> f64 {
        self.wheel
            .borrow_mut()
            .consume_touch(translation_y, cell_height)
    }
    pub fn consume_wheel_pixels(&self, amount: f64) -> f64 {
        self.wheel.borrow_mut().accumulate_pixels(amount)
    }
    pub fn consume_wheel(&self, input: WheelInput, routing: MouseRouting) -> f64 {
        self.wheel.borrow_mut().consume(
            input,
            routing.device_cell_height,
            routing.dpr,
            routing.wheel_options,
        )
    }

    pub fn route<W, E>(
        &self,
        service: &MouseStateService<W>,
        input: MouseInput<'_, W>,
        routing: MouseRouting,
        mut emit: impl FnMut(MouseOutput) -> Result<(), E>,
    ) -> Result<bool, MouseRouteError<E>> {
        let Some(pos) = input.coords else {
            return Ok(false);
        };
        let raw = input.pointer;
        let (button, action) = match raw.kind {
            PointerEventKind::Move => {
                let button = if let Some(buttons) = raw.buttons {
                    if buttons & 1 != 0 {
                        MouseButton::Left
                    } else if buttons & 4 != 0 {
                        MouseButton::Middle
                    } else if buttons & 2 != 0 {
                        MouseButton::Right
                    } else {
                        MouseButton::None
                    }
                } else {
                    raw_button(raw.button)
                };
                (button, MouseAction::Move)
            }
            PointerEventKind::Up => (raw_button(raw.button), MouseAction::Up),
            PointerEventKind::Down => (raw_button(raw.button), MouseAction::Down),
            PointerEventKind::Wheel => {
                let payload = input
                    .wheel_payload
                    .ok_or(MouseRouteError::MissingWheelPayload)?;
                if !service.allow_custom_wheel_event(payload) || raw.delta_y == 0.0 {
                    return Ok(false);
                }
                let lines = self.consume_wheel(
                    WheelInput {
                        delta_y: raw.delta_y,
                        delta_mode: raw.delta_mode,
                        ctrl: raw.ctrl,
                        alt: raw.alt,
                        shift: raw.shift,
                    },
                    routing,
                );
                if lines == 0.0 {
                    return Ok(false);
                }
                (
                    MouseButton::Wheel,
                    if raw.delta_y < 0.0 {
                        MouseAction::Up
                    } else {
                        MouseAction::Down
                    },
                )
            }
            PointerEventKind::Other => return Ok(false),
        };
        let alt_gate = button != MouseButton::Wheel
            && routing.require_alt
            && service.are_mouse_events_active();
        if alt_gate && !raw.alt {
            return Ok(false);
        }
        let mut event = MouseEvent {
            col: pos.col,
            row: pos.row,
            x: pos.x,
            y: pos.y,
            button,
            action,
            ctrl: Some(raw.ctrl),
            alt: Some(raw.alt && !alt_gate),
            shift: Some(raw.shift),
        };
        self.trigger(service, &mut event, routing.cols, routing.rows, |output| {
            emit(output).map_err(MouseRouteError::Output)
        })
    }

    pub fn trigger<W, E>(
        &self,
        service: &MouseStateService<W>,
        event: &mut MouseEvent,
        cols: f64,
        rows: f64,
        mut emit: impl FnMut(MouseOutput) -> Result<(), E>,
    ) -> Result<bool, E> {
        if event.col < 0.0 || event.col >= cols || event.row < 0.0 || event.row >= rows {
            return Ok(false);
        }
        if (event.button == MouseButton::Wheel && event.action == MouseAction::Move)
            || (event.button == MouseButton::None && event.action != MouseAction::Move)
            || (event.button != MouseButton::Wheel
                && matches!(event.action, MouseAction::Left | MouseAction::Right))
        {
            return Ok(false);
        }
        event.col += 1.0;
        event.row += 1.0;
        let last = self.last_event.borrow().clone();
        if event.action == MouseAction::Move
            && last
                .as_ref()
                .is_some_and(|last| equal_events(last, event, service.is_pixel_encoding()))
        {
            return Ok(false);
        }
        if !service.restrict_mouse_event(event) {
            return Ok(false);
        }
        let report = service.encode_mouse_event(event);
        if !report.is_empty() {
            emit(if service.is_default_encoding() {
                MouseOutput::Binary(report)
            } else {
                MouseOutput::UserData(report)
            })?;
        }
        *self.last_event.borrow_mut() = Some(event.clone());
        Ok(true)
    }
}

fn raw_button(button: Option<i16>) -> MouseButton {
    match button {
        Some(0) => MouseButton::Left,
        Some(1) => MouseButton::Middle,
        Some(2) => MouseButton::Right,
        Some(button) if button < 0 => MouseButton::Raw(i32::from(button)),
        _ => MouseButton::None,
    }
}

fn equal_events(first: &MouseEvent, second: &MouseEvent, pixels: bool) -> bool {
    let same_position = if pixels {
        first.x == second.x && first.y == second.y
    } else {
        first.col == second.col && first.row == second.row
    };
    same_position
        && first.button == second.button
        && first.action == second.action
        && first.ctrl == second.ctrl
        && first.alt == second.alt
        && first.shift == second.shift
}
