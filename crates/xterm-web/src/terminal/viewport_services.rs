//! Central browser routing. Viewport requests bypass this routing when applying
//! their native line displacement, so smooth-scroll callbacks cannot recurse.
use super::browser_services::BrowserServices;
use crate::{
    js_error::{callback_error, to_js_error},
    viewport::Viewport,
};
use std::rc::Rc;
use wasm_bindgen::JsValue;

impl BrowserServices {
    fn current_viewport(&self) -> Result<Rc<Viewport>, JsValue> {
        self.viewport
            .get()
            .cloned()
            .ok_or_else(|| js_sys::Error::new("Viewport is not initialized").into())
    }

    /// Called once after the actual Viewport factory succeeds during open.
    pub(crate) fn connect_viewport(self: &Rc<Self>, viewport: Rc<Viewport>) -> Result<(), JsValue> {
        self.viewport
            .set(Rc::clone(&viewport))
            .map_err(|_| js_sys::Error::new("Viewport is already initialized"))?;
        let weak = Rc::downgrade(self);
        let subscription = viewport
            .on_request_scroll_lines()
            .try_subscribe(move |difference| {
                if let Some(services) = weak.upgrade() {
                    services
                        .apply_viewport_scroll(*difference)
                        .map_err(callback_error)?;
                }
                Ok(())
            });
        self.listeners.borrow_mut().push(subscription);
        Ok(())
    }

    pub fn queue_viewport_sync(&self) -> Result<(), JsValue> {
        self.current_viewport()?.queue_sync(None)
    }
    pub fn handle_touch_scroll(&self, translation_y: f64) -> Result<(), JsValue> {
        self.current_viewport()?.handle_touch_scroll(translation_y)
    }
    pub fn scroll_lines(&self, amount: i64, _suppress_scroll_event: bool) -> Result<(), JsValue> {
        // Browser displacements are JS Numbers. Source ignores suppression once
        // a Viewport exists; the native fallback before open owns that flag.
        self.scroll_lines_number(amount as f64)
    }
    pub fn scroll_lines_number(&self, amount: f64) -> Result<(), JsValue> {
        self.current_viewport()?.scroll_lines(amount)?;
        self.refresh_all_rows()
    }
    pub fn scroll_to_line_number(&self, line: f64) -> Result<(), JsValue> {
        let active = self
            .native
            .with_state(|state| state.active_buffer())
            .map_err(|error| to_js_error(&error))?;
        let display = active
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .ydisp;
        let difference = line - display as f64;
        if difference != 0.0 {
            self.scroll_lines_number(difference)?;
        }
        Ok(())
    }
    pub fn scroll_to_bottom_number(&self, disable_smooth: bool) -> Result<(), JsValue> {
        let active = self
            .native
            .with_state(|state| state.active_buffer())
            .map_err(|error| to_js_error(&error))?;
        let (base, display) = {
            let buffer = active.try_borrow().map_err(|error| to_js_error(&error))?;
            (buffer.ybase, buffer.ydisp)
        };
        if disable_smooth {
            self.current_viewport()?.scroll_to_line(base as f64, true)
        } else {
            self.scroll_lines_number(base as f64 - display as f64)
        }
    }
    fn apply_viewport_scroll(&self, difference: f64) -> Result<(), JsValue> {
        // Source scrollbar requests are rounded line differences. Saturating
        // only beyond i64 keeps their sign and the service's top/bottom behavior
        // because the supported buffer range is bounded below i64::MAX.
        if difference.is_nan() || (difference.is_finite() && difference.fract() != 0.0) {
            return Err(js_sys::RangeError::new(
                "Viewport produced a nonintegral line displacement",
            )
            .into());
        }
        let amount = if difference <= i64::MIN as f64 {
            i64::MIN
        } else if difference >= -(i64::MIN as f64) {
            i64::MAX
        } else {
            difference as i64
        };
        self.native
            .scroll_lines(amount)
            .map_err(|error| to_js_error(&error))?;
        self.refresh_all_rows()
    }
    fn refresh_all_rows(&self) -> Result<(), JsValue> {
        let rows = self.native.rows().map_err(|error| to_js_error(&error))?;
        self.refresh(0.0, (rows - 1) as f64)
    }
}
