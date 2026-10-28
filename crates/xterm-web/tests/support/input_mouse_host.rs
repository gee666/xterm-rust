//! Mouse collaborators for clipboard/IME/keyboard component tests. Full mouse
//! browser behavior is tested by wasm_mouse, not by these fixed snapshots.
use super::*;
use xterm_web::input::{
    dom::mouse::{MouseHost, MouseReadHost},
    mouse_policy::MouseOutput,
};

impl MouseReadHost for Host {
    fn mouse_require_alt(&self) -> Result<bool, JsValue> {
        Ok(false)
    }
    fn mouse_has_scrollback(&self) -> Result<bool, JsValue> {
        Ok(true)
    }
    fn mouse_application_cursor(&self) -> Result<bool, JsValue> {
        Ok(false)
    }
    fn mouse_cols(&self) -> Result<f64, JsValue> {
        Ok(80.0)
    }
    fn mouse_rows(&self) -> Result<f64, JsValue> {
        Ok(24.0)
    }
    fn mouse_scroll_sensitivity(&self) -> Result<JsValue, JsValue> {
        Ok(1.0.into())
    }
    fn mouse_fast_scroll_sensitivity(&self) -> Result<JsValue, JsValue> {
        Ok(5.0.into())
    }
    fn mouse_device_cell_height(&self) -> Result<Option<f64>, JsValue> {
        Ok(Some(18.0))
    }
    fn mouse_dpr(&self) -> Result<Option<f64>, JsValue> {
        Ok(Some(1.0))
    }
    fn mouse_valid_char_size(&self) -> Result<bool, JsValue> {
        Ok(true)
    }
    fn mouse_canvas_size(&self) -> Result<[f64; 2], JsValue> {
        Ok([720.0, 432.0])
    }
    fn mouse_cell_size(&self) -> Result<[f64; 2], JsValue> {
        Ok([9.0, 18.0])
    }
    fn mouse_css_cell_height(&self) -> Result<Option<f64>, JsValue> {
        Ok(Some(18.0))
    }
}

impl MouseHost for Host {
    fn mouse_option_changes(&self) -> xterm_core::event::Event<&'static str> {
        self.option_changes.event()
    }
    fn mouse_protocol_log(
        &self,
        _: xterm_core::mouse_state::MouseEventTypes,
    ) -> Result<(), JsValue> {
        Ok(())
    }
    fn mouse_service(&self) -> Result<Rc<xterm_core::mouse_state::MouseStateService>, JsValue> {
        Ok(self.mouse.clone())
    }
    fn allow_custom_wheel(&self, _: &web_sys::WheelEvent) -> Result<bool, JsValue> {
        Ok(true)
    }
    fn mouse_focus(&self) -> Result<(), JsValue> {
        Ok(())
    }
    fn should_force_selection(&self, _: &MouseEvent) -> Result<bool, JsValue> {
        Ok(false)
    }
    fn set_mouse_selection_enabled(&self, _: bool) -> Result<(), JsValue> {
        Ok(())
    }
    fn mouse_output(&self, output: MouseOutput) -> Result<(), JsValue> {
        match output {
            MouseOutput::Binary(data) => {
                self.data.borrow_mut().push((data, false));
                Ok(())
            }
            MouseOutput::UserData(data) => InputHost::data(self, &data, true),
        }
    }
    fn mouse_touch_scroll(&self, _: f64) -> Result<(), JsValue> {
        Ok(())
    }
    fn mouse_gesture_target(
        &self,
        _: &HtmlElement,
    ) -> Result<xterm_core::lifecycle::DisposableHandle, JsValue> {
        Ok(xterm_core::lifecycle::to_disposable(|| {}))
    }
}
