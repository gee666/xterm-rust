use super::*;
impl RenderService {
    pub fn dispose(&self) -> Result<(), JsValue> {
        if self.disposed.replace(true) {
            return Ok(());
        }
        let renderer = self.renderer.borrow().clone();
        if let Some(renderer) = renderer {
            renderer.dispose()?;
        }
        self.renderer.borrow_mut().take();
        self.disconnect_observer();
        self.dimensions_change.dispose();
        self.viewport_change.dispose();
        self.render.dispose();
        self.refresh_request.dispose();
        self.paused_resize.clear();
        if let Some(debouncer) = self.debouncer.borrow().clone() {
            debouncer.dispose()?;
        }
        if let Some(synchronized) = self.synchronized.borrow().clone() {
            synchronized.dispose();
        }
        self.listeners.dispose();
        Ok(())
    }
}
impl Drop for RenderService {
    fn drop(&mut self) {
        if let Err(error) = self.dispose() {
            web_sys::console::error_1(&error);
        }
        // Native ownership still cancels host work if an earlier explicit
        // disposal callback threw, without repeating the renderer callback.
        self.disconnect_observer();
        self.paused_resize.clear();
        if let Some(debouncer) = self.debouncer.get_mut().take() {
            if let Err(error) = debouncer.dispose() {
                web_sys::console::error_1(&error);
            }
        }
        if let Some(synchronized) = self.synchronized.get_mut().take() {
            synchronized.dispose();
        }
    }
}
