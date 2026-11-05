//! Read-only inspection of the actual renderer's dimensions, not a metrics copy.
use crate::renderer::{service::RenderService, service_api::Renderer};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use wasm_bindgen::prelude::*;

type DimensionsView = (Weak<dyn Renderer>, JsValue);

#[wasm_bindgen(module = "/src/terminal/internal_render.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = dimensionsView)]
    fn dimensions_view(read: &js_sys::Function) -> Result<JsValue, JsValue>;
}

/// Internal diagnostic view. Raw writable service replacement remains unsupported.
#[wasm_bindgen]
pub struct InternalRenderServiceView {
    service: Rc<RenderService>,
    dimensions: RefCell<Option<DimensionsView>>,
}

impl InternalRenderServiceView {
    pub(super) fn new(service: Rc<RenderService>) -> Self {
        Self {
            service,
            dimensions: RefCell::new(None),
        }
    }
}

#[wasm_bindgen]
impl InternalRenderServiceView {
    #[wasm_bindgen(getter)]
    pub fn dimensions(&self) -> Result<JsValue, JsValue> {
        let renderer = self
            .service
            .current_renderer()
            .ok_or_else(|| js_sys::TypeError::new("RenderService has no renderer"))?;
        if let Some((previous, view)) = self.dimensions.borrow().as_ref() {
            if previous
                .upgrade()
                .is_some_and(|previous| Rc::ptr_eq(&previous, &renderer))
            {
                return Ok(view.clone());
            }
        }
        let owner = Rc::clone(&renderer);
        // Retained dimensions keep reading their original renderer, even after
        // replacement or disposal. Service disposal still releases host jobs.
        let read =
            Closure::wrap(
                Box::new(move || super::events::dimensions_value(&owner.dimensions()))
                    as Box<dyn Fn() -> Result<JsValue, JsValue>>,
            )
            .into_js_value();
        let view = dimensions_view(read.unchecked_ref())?;
        *self.dimensions.borrow_mut() = Some((Rc::downgrade(&renderer), view.clone()));
        Ok(view)
    }
}
