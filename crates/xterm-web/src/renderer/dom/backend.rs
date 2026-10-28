use super::*;
use crate::renderer::service_api::{LinkUnderlineRenderer, Renderer, VisibilityRenderer};
use xterm_core::event::Event;
impl Renderer for DomRenderer {
    fn dimensions(&self) -> RenderDimensions {
        DomRenderer::dimensions(self)
    }
    fn on_request_redraw(&self) -> Event<RedrawRequest> {
        DomRenderer::on_request_redraw(self)
    }
    fn dispose(&self) -> Result<(), JsValue> {
        DomRenderer::dispose(self);
        Ok(())
    }
    fn handle_device_pixel_ratio_change(&self) -> Result<(), JsValue> {
        DomRenderer::handle_device_pixel_ratio_change(self)
    }
    fn handle_resize(&self, cols: usize, rows: usize) -> Result<(), JsValue> {
        DomRenderer::handle_resize(self, cols, rows)
    }
    fn handle_char_size_changed(&self) -> Result<(), JsValue> {
        DomRenderer::handle_char_size_changed(self)
    }
    fn handle_blur(&self) -> Result<(), JsValue> {
        DomRenderer::handle_blur(self)
    }
    fn handle_focus(&self) -> Result<(), JsValue> {
        DomRenderer::handle_focus(self)
    }
    fn handle_selection_changed(
        &self,
        start: Option<[i64; 2]>,
        end: Option<[i64; 2]>,
        column: bool,
    ) -> Result<(), JsValue> {
        DomRenderer::handle_selection_changed(self, start, end, column)
    }
    fn handle_cursor_move(&self) -> Result<(), JsValue> {
        DomRenderer::handle_cursor_move(self)
    }
    fn clear(&self) -> Result<(), JsValue> {
        DomRenderer::clear(self)
    }
    fn render_rows(&self, start: f64, end: f64) -> Result<(), JsValue> {
        DomRenderer::render_rows_js(self, start, end)
    }
    fn visibility(&self) -> Option<&dyn VisibilityRenderer> {
        Some(self)
    }
    fn link_underline(&self) -> Option<&dyn LinkUnderlineRenderer> {
        Some(self)
    }
}
impl LinkUnderlineRenderer for DomRenderer {
    fn handle_link_underline(
        &self,
        enabled: bool,
        event: crate::links::UnderlineEvent,
    ) -> Result<(), JsValue> {
        DomRenderer::handle_link_underline(self, enabled, event)
    }
}
impl VisibilityRenderer for DomRenderer {
    fn handle_viewport_visibility_change(&self, visible: bool) -> Result<(), JsValue> {
        DomRenderer::handle_viewport_visibility_change(self, visible)
    }
}
