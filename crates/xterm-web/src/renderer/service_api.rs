//! Shared renderer replacement contract corresponding to upstream IRenderer.
use crate::api::RenderDimensions;
use wasm_bindgen::JsValue;
use xterm_core::event::Event;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedrawRequest {
    pub start: usize,
    pub end: usize,
    pub sync: bool,
}
/// Optional upstream methods are capabilities, not successful no-op methods.
pub trait VisibilityRenderer {
    fn handle_viewport_visibility_change(&self, visible: bool) -> Result<(), JsValue>;
}
pub trait TextureAtlasRenderer {
    fn clear_texture_atlas(&self) -> Result<(), JsValue>;
}
pub trait Renderer {
    fn dimensions(&self) -> RenderDimensions;
    fn on_request_redraw(&self) -> Event<RedrawRequest>;
    fn dispose(&self) -> Result<(), JsValue>;
    fn handle_device_pixel_ratio_change(&self) -> Result<(), JsValue>;
    fn handle_resize(&self, cols: usize, rows: usize) -> Result<(), JsValue>;
    fn handle_char_size_changed(&self) -> Result<(), JsValue>;
    fn handle_blur(&self) -> Result<(), JsValue>;
    fn handle_focus(&self) -> Result<(), JsValue>;
    fn handle_selection_changed(
        &self,
        start: Option<[i64; 2]>,
        end: Option<[i64; 2]>,
        column: bool,
    ) -> Result<(), JsValue>;
    fn handle_cursor_move(&self) -> Result<(), JsValue>;
    fn clear(&self) -> Result<(), JsValue>;
    fn render_rows(&self, start: usize, end: usize) -> Result<(), JsValue>;
    fn visibility(&self) -> Option<&dyn VisibilityRenderer> {
        None
    }
    fn texture_atlas(&self) -> Option<&dyn TextureAtlasRenderer> {
        None
    }
}
