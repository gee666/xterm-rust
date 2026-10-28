use super::*;
use xterm_core::event::Event;
impl DomRenderer {
    pub(super) fn initialize_blink(self: &Rc<Self>) -> Result<(), JsValue> {
        let weak = Rc::downgrade(self);
        let text = TextBlinkController::new(
            self.window.clone(),
            self.options.borrow().blink_interval,
            move || {
                if let Some(renderer) = weak.upgrade() {
                    let rows = renderer
                        .terminal
                        .rows()
                        .map_err(|e| crate::js_error::to_js_error(&e))?;
                    renderer
                        .redraw
                        .fire(&RedrawRequest {
                            start: 0,
                            end: rows - 1,
                            sync: false,
                        })
                        .map_err(|e| crate::js_error::to_js_error(&e))?;
                }
                Ok(())
            },
        );
        let cursor = CursorBlinkController::new(
            self.window.clone(),
            self.row_container.clone(),
            self.focused.get(),
        )?;
        *self.text_blink.borrow_mut() = Some(text);
        *self.cursor_blink.borrow_mut() = Some(cursor);
        let weak = Rc::downgrade(self);
        let listener = crate::dom_listener::DomListener::new_reentrant_fallible(
            self.document.as_ref(),
            "mousedown",
            false,
            None,
            move |_| {
                if let Some(renderer) = weak.upgrade() {
                    renderer.handle_cursor_move()?;
                }
                Ok(())
            },
        )?;
        self.dom_listeners.borrow_mut().push(listener);
        Ok(())
    }
    pub fn on_request_redraw(&self) -> Event<RedrawRequest> {
        self.redraw.event()
    }
    pub fn handle_cursor_move(&self) -> Result<(), JsValue> {
        let blink = self
            .cursor_blink
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::Error::new("DOM renderer has been disposed"))?;
        blink.restart()
    }
    pub fn handle_viewport_visibility_change(&self, visible: bool) -> Result<(), JsValue> {
        let blink = self
            .text_blink
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::Error::new("DOM renderer has been disposed"))?;
        blink.set_visible(visible)
    }
    pub(super) fn update_text_blink(&self) -> Result<(), JsValue> {
        let needs = self.row_blink.borrow().iter().any(|value| *value);
        let blink = self.text_blink.borrow().clone();
        if let Some(blink) = blink {
            blink.set_needs_blink(needs)?;
        }
        Ok(())
    }
    pub(super) fn update_blink_duration(&self) -> Result<(), JsValue> {
        let duration = self.options.borrow().blink_interval;
        let blink = self.text_blink.borrow().clone();
        if let Some(blink) = blink {
            blink.set_duration(duration)?;
        }
        Ok(())
    }
    pub(super) fn is_text_blink_on(&self) -> bool {
        self.text_blink
            .borrow()
            .as_ref()
            .is_none_or(TextBlinkController::is_blink_on)
    }
}
