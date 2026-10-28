use super::{error, EngineInputHost};
use crate::input::{
    dom::mouse::{MouseHost, MouseSnapshot},
    mouse_policy::{MouseOutput, MouseRouting},
    mouse_wheel::WheelOptions,
};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::{HtmlElement, MouseEvent, WheelEvent};
use xterm_core::{
    lifecycle::DisposableHandle,
    mouse_state::{MouseEventTypes, MouseStateService},
};

impl MouseHost for EngineInputHost {
    fn mouse_service(&self) -> Result<Rc<MouseStateService>, JsValue> {
        Ok(self.mouse.clone())
    }
    fn mouse_protocol_log(
        &self,
        events: xterm_core::mouse_state::MouseEventTypes,
    ) -> Result<(), JsValue> {
        use xterm_core::{
            log_service::{LogLevel, LogParameter, LogValue},
            options::OptionValue,
        };
        let logger = self
            .terminal
            .with_state(|state| state.logger.clone())
            .map_err(error)?;
        if events.0 == 0 {
            logger
                .log(
                    LogLevel::Debug,
                    &"Unbinding from mouse events."
                        .encode_utf16()
                        .collect::<Vec<_>>(),
                    Vec::new(),
                )
                .map_err(error)?;
        } else if self.options.get("logLevel").map_err(error)? == OptionValue::from("debug") {
            let explanation = OptionValue::property_map([
                (
                    "down",
                    OptionValue::Bool(events.0 & MouseEventTypes::DOWN.0 != 0),
                ),
                (
                    "up",
                    OptionValue::Bool(events.0 & MouseEventTypes::UP.0 != 0),
                ),
                (
                    "drag",
                    OptionValue::Bool(events.0 & MouseEventTypes::DRAG.0 != 0),
                ),
                (
                    "move",
                    OptionValue::Bool(events.0 & MouseEventTypes::MOVE.0 != 0),
                ),
                (
                    "wheel",
                    OptionValue::Bool(events.0 & MouseEventTypes::WHEEL.0 != 0),
                ),
            ]);
            logger
                .log(
                    LogLevel::Debug,
                    &"Binding to mouse events:"
                        .encode_utf16()
                        .collect::<Vec<_>>(),
                    vec![LogParameter::Value(LogValue::Value(explanation))],
                )
                .map_err(error)?;
        }
        Ok(())
    }

    fn mouse_snapshot(&self) -> Result<MouseSnapshot, JsValue> {
        let metrics = self.services.metrics()?;
        let buffer = self.buffer.buffer();
        let has_scrollback = buffer.try_borrow().map_err(error)?.has_scrollback();
        Ok(MouseSnapshot {
            routing: MouseRouting {
                cols: self.buffer.cols() as f64,
                rows: self.buffer.rows() as f64,
                require_alt: self.option_bool("mouseEventsRequireAlt")?,
                device_cell_height: Some(metrics.device_cell_height),
                dpr: Some(metrics.dpr),
                wheel_options: WheelOptions {
                    rows: self.buffer.rows() as f64,
                    scroll_sensitivity: self.option_number("scrollSensitivity")?,
                    fast_scroll_sensitivity: self.option_number("fastScrollSensitivity")?,
                },
            },
            valid_char_size: metrics.valid,
            css_cell_width: metrics.cell_width,
            css_cell_height: metrics.cell_height,
            css_canvas_width: metrics.canvas_width,
            css_canvas_height: metrics.canvas_height,
            has_scrollback,
            application_cursor: self
                .core
                .dec_private_modes
                .try_borrow()
                .map_err(error)?
                .application_cursor_keys,
        })
    }
    fn allow_custom_wheel(&self, event: &WheelEvent) -> Result<bool, JsValue> {
        self.services.custom_wheel(event)
    }
    fn mouse_focus(&self) -> Result<(), JsValue> {
        let options = web_sys::FocusOptions::new();
        options.set_prevent_scroll(true);
        self.dom.textarea.focus_with_options(&options)
    }
    fn should_force_selection(&self, event: &MouseEvent) -> Result<bool, JsValue> {
        self.services.force_selection(event)
    }
    fn set_mouse_selection_enabled(&self, enabled: bool) -> Result<(), JsValue> {
        self.services.set_selection_enabled(enabled)
    }
    fn mouse_output(&self, output: MouseOutput) -> Result<(), JsValue> {
        match output {
            MouseOutput::UserData(data) => self.terminal.input(&data, true).map_err(error),
            MouseOutput::Binary(data) => self.terminal.input_binary(&data).map_err(error),
        }
    }
    fn mouse_touch_scroll(&self, translation_y: f64) -> Result<(), JsValue> {
        self.services.touch_scroll(translation_y)
    }
    fn mouse_gesture_target(&self, screen: &HtmlElement) -> Result<DisposableHandle, JsValue> {
        self.services.gesture_target(screen)
    }
}
