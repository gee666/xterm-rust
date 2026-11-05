//! Public joiners retain raw arrays until the renderer consumes their ranges.
use super::{browser_services::BrowserServices, Terminal};
use js_sys::Function;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

impl Terminal {
    fn joiner_services(&self) -> Result<Rc<BrowserServices>, JsValue> {
        self.shared
            .services
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::Error::new("Terminal must be opened first").into())
    }
}

#[wasm_bindgen]
impl Terminal {
    #[wasm_bindgen(js_name = registerCharacterJoiner)]
    pub fn register_character_joiner(&self, handler: Function) -> Result<f64, JsValue> {
        let services = self.joiner_services()?;
        let id = services.joiners.register_js(handler);
        services.refresh(0.0, self.rows()? - 1.0)?;
        Ok(id)
    }
    #[wasm_bindgen(js_name = deregisterCharacterJoiner)]
    pub fn deregister_character_joiner(&self, id: f64) -> Result<(), JsValue> {
        let services = self.joiner_services()?;
        if services.joiners.deregister_js(id)? {
            services.refresh(0.0, self.rows()? - 1.0)?;
        }
        Ok(())
    }
}
