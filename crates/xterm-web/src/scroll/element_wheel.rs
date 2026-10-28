// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
use super::*;
use crate::terminal::options::into_js;

impl SmoothScrollableElement {
    pub(super) fn clear_wheel_listeners(&self) -> Result<(), JsValue> {
        for mut listener in std::mem::take(&mut *self.wheel_listeners.borrow_mut()) {
            listener.dispose()?;
        }
        Ok(())
    }
    pub(super) fn set_listening_to_wheel(self: &Rc<Self>) -> Result<(), JsValue> {
        let should = self.options.borrow().handle_mouse_wheel.clone();
        let listening = !self.wheel_listeners.borrow().is_empty();
        if V::Bool(listening) == should {
            return Ok(());
        }
        self.clear_wheel_listeners()?;
        if should.is_truthy() {
            let target =
                self.listen_on.borrow().clone().ok_or_else(|| {
                    js_sys::TypeError::new("scroll listener node is not initialized")
                })?;
            let weak = Rc::downgrade(self);
            let listener = DomListener::new_reentrant_fallible(
                &target,
                "wheel",
                false,
                Some(false),
                move |event| {
                    if let Some(this) = weak.upgrade() {
                        let wheel =
                            StandardWheelEvent::new(Some(&event), 0.0, 0.0, &this.environment)?;
                        this.handle_mouse_wheel(&wheel)?;
                    }
                    Ok(())
                },
            )?;
            self.wheel_listeners.borrow_mut().push(listener);
        }
        Ok(())
    }
    pub(super) fn handle_mouse_wheel(&self, event: &StandardWheelEvent) -> Result<(), JsValue> {
        if let Some(raw) = event.browser_event.as_ref() {
            if interop::get(raw.as_ref(), "defaultPrevented")?.is_truthy() {
                return Ok(());
            }
        }
        self.environment.accept_wheel(event)?;
        let mut did_scroll = false;
        if event.delta_y.is_truthy() || event.delta_x.is_truthy() {
            let sensitivity = self.options.borrow().mouse_wheel_scroll_sensitivity.clone();
            let mut delta_y =
                interop::number(&interop::multiply(&event.delta_y, &into_js(&sensitivity)?)?)?;
            let sensitivity = self.options.borrow().mouse_wheel_scroll_sensitivity.clone();
            let mut delta_x =
                interop::number(&interop::multiply(&event.delta_x, &into_js(&sensitivity)?)?)?;
            if self.options.borrow().scroll_predominant_axis.is_truthy() {
                if self.options.borrow().scroll_y_to_x.is_truthy() && delta_x + delta_y == 0.0 {
                    delta_x = 0.0;
                    delta_y = 0.0;
                } else if delta_y.abs() >= delta_x.abs() {
                    delta_x = 0.0;
                } else {
                    delta_y = 0.0;
                }
            }
            if self.options.borrow().flip_axes.is_truthy() {
                std::mem::swap(&mut delta_y, &mut delta_x);
            }
            let shift = if !self.environment.platform.is_mac {
                match event.browser_event.as_ref().filter(|event| {
                    let raw: &JsValue = event.as_ref();
                    raw.is_truthy()
                }) {
                    Some(raw) => interop::get(raw.as_ref(), "shiftKey")?.is_truthy(),
                    None => false,
                }
            } else {
                false
            };
            if (self.options.borrow().scroll_y_to_x.is_truthy() || shift) && !js_truthy(delta_x) {
                delta_x = delta_y;
                delta_y = 0.0;
            }
            if let Some(raw) = event.browser_event.as_ref().filter(|event| {
                let raw: &JsValue = event.as_ref();
                raw.is_truthy()
            }) {
                if interop::get(raw.as_ref(), "altKey")?.is_truthy() {
                    let fast = self.options.borrow().fast_scroll_sensitivity.clone();
                    delta_x =
                        interop::number(&interop::multiply(&delta_x.into(), &into_js(&fast)?)?)?;
                    let fast = self.options.borrow().fast_scroll_sensitivity.clone();
                    delta_y =
                        interop::number(&interop::multiply(&delta_y.into(), &into_js(&fast)?)?)?;
                }
            }
            let future = self.model.future_scroll_position();
            let mut desired = PositionUpdate::default();
            if js_truthy(delta_y) {
                let pixels = SCROLL_WHEEL_SENSITIVITY * delta_y;
                self.vertical()?
                    .write_scroll_position(&mut desired, future.top - away_from_zero(pixels));
            }
            if js_truthy(delta_x) {
                let pixels = SCROLL_WHEEL_SENSITIVITY * delta_x;
                self.horizontal()?
                    .write_scroll_position(&mut desired, future.left - away_from_zero(pixels));
            }
            let desired = self.model.validate_scroll_position(desired);
            if future.left != desired.left || future.top != desired.top {
                let smooth = self.options.borrow().mouse_wheel_smooth_scroll.is_truthy()
                    && self.environment.is_physical_mouse_wheel();
                let update = PositionUpdate {
                    left: Some(desired.left),
                    top: Some(desired.top),
                };
                if smooth {
                    self.model.set_scroll_position_smooth(update, false)
                } else {
                    self.model.set_scroll_position_now(update)
                }
                .map_err(scroll_error)?;
                did_scroll = true;
            }
        }
        let mut consume = did_scroll;
        if !consume && self.options.borrow().always_consume_mouse_wheel.is_truthy() {
            consume = true;
        }
        if !consume
            && self
                .options
                .borrow()
                .consume_mouse_wheel_if_scrollbar_is_needed
                .is_truthy()
            && (self.vertical()?.is_needed() || self.horizontal()?.is_needed())
        {
            consume = true;
        }
        if consume {
            event.prevent_default()?;
            event.stop_propagation()?;
        }
        Ok(())
    }
}
fn js_truthy(value: f64) -> bool {
    value != 0.0 && !value.is_nan()
}
fn away_from_zero(value: f64) -> f64 {
    if value < 0.0 {
        value.floor()
    } else {
        value.ceil()
    }
}
