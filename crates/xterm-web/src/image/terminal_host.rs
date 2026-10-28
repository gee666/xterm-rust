use super::{js, CanvasDimensions, ImageCanvasHost};
use crate::terminal::Shared;
use std::rc::Weak;
use wasm_bindgen::JsValue;

/// Canvas ownership follows the terminal's current document, never a cached
/// global document. A weak reference keeps image subscriptions cycle-free.
pub(super) struct TerminalCanvasHost(pub Weak<Shared>);
impl ImageCanvasHost for TerminalCanvasHost {
    fn document(&self) -> Result<JsValue, JsValue> {
        let services = self
            .0
            .upgrade()
            .and_then(|shared| shared.services.borrow().clone());
        Ok(services
            .and_then(|services| services.core_browser.window().document())
            .map(JsValue::from)
            .unwrap_or(JsValue::UNDEFINED))
    }
    fn screen_element(&self) -> Result<JsValue, JsValue> {
        let dom = self
            .0
            .upgrade()
            .and_then(|shared| shared.dom.borrow().clone());
        Ok(dom
            .map(|dom| dom.screen.clone().into())
            .unwrap_or(JsValue::UNDEFINED))
    }
    fn dimensions(&self) -> Result<Option<CanvasDimensions>, JsValue> {
        let services = self
            .0
            .upgrade()
            .and_then(|shared| shared.services.borrow().clone());
        services
            .map(|services| {
                let d = services.render_service.dimensions()?;
                Ok(CanvasDimensions {
                    cell_width: d.css.cell.width,
                    cell_height: d.css.cell.height,
                    canvas_width: d.css.canvas.width,
                    canvas_height: d.css.canvas.height,
                })
            })
            .transpose()
    }
    fn screen_width(&self) -> Result<f64, JsValue> {
        // Source's placeholder width uses the module-global screen, not the
        // terminal's owner window. Preserve that distinction after adoption.
        js::number(&js::get(&js_sys::global().into(), "screen")?, "width")
    }
}
