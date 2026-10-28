//! Shared renderer replacement contract corresponding to upstream IRenderer.
use crate::api::RenderDimensions;
use wasm_bindgen::JsValue;
use web_sys::Window;
use xterm_core::event::Event;

/// The live CoreBrowserService view shared by renderers and RenderService.
/// A trait also permits controlled observer/clock tests without a second host.
pub trait BrowserService {
    fn window(&self) -> Window;
    fn dpr(&self) -> Result<f64, JsValue>;
    fn is_focused(&self) -> Result<bool, JsValue>;
    fn on_dpr_change(&self) -> Event<f64>;
    fn on_window_change(&self) -> Event<Window>;
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RedrawRequest {
    pub start: f64,
    pub end: f64,
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
    fn render_rows(&self, start: f64, end: f64) -> Result<(), JsValue>;
    fn visibility(&self) -> Option<&dyn VisibilityRenderer> {
        None
    }
    fn texture_atlas(&self) -> Option<&dyn TextureAtlasRenderer> {
        None
    }
}
