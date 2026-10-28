//! Mouse collaborators for clipboard/IME/keyboard component tests. Full mouse
//! browser behavior is tested by wasm_mouse, not by these fixed snapshots.
use super::*;
use xterm_web::input::{
    dom::mouse::{MouseHost, MouseSnapshot},
    mouse_policy::{MouseOutput, MouseRouting},
    mouse_wheel::WheelOptions,
};

impl MouseHost for Host {
    fn mouse_protocol_log(
        &self,
        _: xterm_core::mouse_state::MouseEventTypes,
    ) -> Result<(), JsValue> {
        Ok(())
    }
    fn mouse_service(&self) -> Result<Rc<xterm_core::mouse_state::MouseStateService>, JsValue> {
        Ok(self.mouse.clone())
    }
    fn mouse_snapshot(&self) -> Result<MouseSnapshot, JsValue> {
        Ok(MouseSnapshot {
            routing: MouseRouting {
                cols: 80.0,
                rows: 24.0,
                require_alt: false,
                device_cell_height: Some(18.0),
                dpr: Some(1.0),
                wheel_options: WheelOptions::default(),
            },
            valid_char_size: true,
            css_cell_width: 9.0,
            css_cell_height: 18.0,
            css_canvas_width: 720.0,
            css_canvas_height: 432.0,
            has_scrollback: true,
            application_cursor: false,
        })
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
