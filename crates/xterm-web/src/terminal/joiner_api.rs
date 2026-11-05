//! Public joiner registrations feed the same service used by the active renderer.
use super::{browser_services::BrowserServices, Terminal};
use crate::{character_joiner::JoinerError, utf16::to_js_string};
use js_sys::{Array, Function, Reflect};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

fn ranges(value: JsValue) -> Result<Vec<[usize; 2]>, JoinerError> {
    if !Array::is_array(&value) {
        return Err(JoinerError("Character joiner must return an array".into()));
    }
    Array::from(&value)
        .iter()
        .map(|range| {
            let mut result = [0; 2];
            for (index, offset) in result.iter_mut().enumerate() {
                let value = Reflect::get(&range, &JsValue::from_f64(index as f64))
                    .map_err(|error| JoinerError(format!("{error:?}")))?;
                let number = value
                    .as_f64()
                    .filter(|number| number.is_finite() && *number >= 0.0 && number.fract() == 0.0)
                    .filter(|number| *number <= u32::MAX as f64)
                    .ok_or_else(|| JoinerError("Invalid character joiner offset".into()))?;
                *offset = number as usize;
            }
            Ok(result)
        })
        .collect()
}

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
        let id = services.joiners.register(Rc::new(move |text| {
            let result = handler
                .call1(&JsValue::UNDEFINED, &to_js_string(text))
                .map_err(|error| JoinerError(format!("{error:?}")))?;
            ranges(result)
        }));
        services.refresh(0.0, self.rows()? - 1.0)?;
        Ok(id)
    }

    #[wasm_bindgen(js_name = deregisterCharacterJoiner)]
    pub fn deregister_character_joiner(&self, id: f64) -> Result<(), JsValue> {
        let services = self.joiner_services()?;
        if services.joiners.deregister(id) {
            services.refresh(0.0, self.rows()? - 1.0)?;
        }
        Ok(())
    }
}
