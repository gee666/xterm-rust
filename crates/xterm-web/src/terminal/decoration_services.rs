//! Decoration overlays share the live renderer's metrics and refresh queue.
use super::browser_services::BrowserServices;
use crate::{
    js_error::to_js_error,
    renderer::buffer_decoration::{BufferDecorationHost, BufferDecorationRenderer},
};
use std::rc::Rc;
use wasm_bindgen::JsValue;

impl BrowserServices {
    pub(super) fn install_decorations(self: &Rc<Self>) -> Result<(), JsValue> {
        let buffers = self
            .native
            .with_state(|state| Rc::clone(&state.buffers))
            .map_err(|error| to_js_error(&error))?;
        // Weak capabilities avoid a cycle through the RenderService's queued
        // callback, which temporarily owns the decoration renderer itself.
        let metrics = Rc::downgrade(&self.render_service);
        let queue = Rc::downgrade(&self.render_service);
        let renderer = BufferDecorationRenderer::new(BufferDecorationHost {
            document: self.core_browser.main_document(),
            screen: self.dom.screen.clone(),
            buffers,
            decorations: Rc::clone(&self.decorations),
            dimensions: Rc::new(move || {
                metrics
                    .upgrade()
                    .ok_or_else(|| js_sys::Error::new("Decoration render service disposed"))?
                    .dimensions()
            }),
            add_refresh_callback: Rc::new(move |callback| {
                queue
                    .upgrade()
                    .ok_or_else(|| js_sys::Error::new("Decoration render service disposed"))?
                    .add_refresh_callback(move |time| callback(time))
            }),
            viewport_changed: self.render_service.on_rendered_viewport_change(),
            dimensions_changed: self.render_service.on_dimensions_change(),
            dpr_changed: self.core_browser.on_dpr_change(),
        })?;
        *self.decoration_renderer.borrow_mut() = Some(renderer);
        Ok(())
    }
}
