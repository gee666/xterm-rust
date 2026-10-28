use super::{error, EngineInputHost};
use crate::input::{
    dom::mouse::{MouseHost, MouseReadHost},
    mouse_policy::MouseOutput,
};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::{HtmlElement, MouseEvent, WheelEvent};
use xterm_core::{
    lifecycle::DisposableHandle,
    mouse_state::{MouseEventTypes, MouseStateService},
};

impl MouseReadHost for EngineInputHost {
    fn mouse_require_alt(&self) -> Result<bool, JsValue> {
        self.option_bool("mouseEventsRequireAlt")
    }
    fn mouse_has_scrollback(&self) -> Result<bool, JsValue> {
        let buffer = self.buffer.buffer();
        let result = buffer.try_borrow().map_err(error)?.has_scrollback();
        Ok(result)
    }
    fn mouse_application_cursor(&self) -> Result<bool, JsValue> {
        Ok(self
            .core
            .dec_private_modes
            .try_borrow()
            .map_err(error)?
            .application_cursor_keys)
    }
    fn mouse_cols(&self) -> Result<f64, JsValue> {
        Ok(self.buffer.cols() as f64)
    }
    fn mouse_rows(&self) -> Result<f64, JsValue> {
        Ok(self.buffer.rows() as f64)
    }
    fn mouse_scroll_sensitivity(&self) -> Result<JsValue, JsValue> {
        crate::terminal::options::into_js(&self.options.get("scrollSensitivity").map_err(error)?)
    }
    fn mouse_fast_scroll_sensitivity(&self) -> Result<JsValue, JsValue> {
        crate::terminal::options::into_js(
            &self.options.get("fastScrollSensitivity").map_err(error)?,
        )
    }
    fn mouse_device_cell_height(&self) -> Result<Option<f64>, JsValue> {
        Ok(Some(self.services.render_dimensions()?.device.cell.height))
    }
    fn mouse_dpr(&self) -> Result<Option<f64>, JsValue> {
        Ok(Some(self.services.dpr()?))
    }
    fn mouse_valid_char_size(&self) -> Result<bool, JsValue> {
        self.services.valid_char_size()
    }
    fn mouse_canvas_size(&self) -> Result<[f64; 2], JsValue> {
        let dimensions = self.services.render_dimensions()?;
        Ok([dimensions.css.canvas.width, dimensions.css.canvas.height])
    }
    fn mouse_cell_size(&self) -> Result<[f64; 2], JsValue> {
        let dimensions = self.services.render_dimensions()?;
        Ok([dimensions.css.cell.width, dimensions.css.cell.height])
    }
    fn mouse_css_cell_height(&self) -> Result<Option<f64>, JsValue> {
        Ok(Some(self.services.render_dimensions()?.css.cell.height))
    }
}

impl MouseHost for EngineInputHost {
    fn mouse_option_changes(&self) -> xterm_core::event::Event<&'static str> {
        self.options.on_option_change()
    }
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
