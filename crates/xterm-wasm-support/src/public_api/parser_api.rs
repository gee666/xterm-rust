//! JavaScript public parser callbacks over the Rust terminal engine.
use crate::{
    js_error::{callback_error, to_js_error},
    utf16::to_js_string,
};
#[path = "parser_identifier.rs"]
mod identifier;
use identifier::{read_identifier, Kind};
use js_sys::{Array, Function, Promise};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use xterm_core::{
    headless::Terminal,
    parser::{
        handler::{DispatchError, HandlerResult},
        params::ParamValue,
    },
};

fn params_array(params: Vec<ParamValue>) -> Array {
    params
        .into_iter()
        .map(|param| match param {
            ParamValue::Param(value) => JsValue::from(value),
            ParamValue::SubParams(values) => values
                .into_iter()
                .map(JsValue::from)
                .collect::<Array>()
                .into(),
        })
        .collect()
}
pub fn result(value: Result<JsValue, JsValue>) -> HandlerResult {
    match value {
        Err(error) => HandlerResult::Failed(DispatchError::Callback(callback_error(error))),
        Ok(value) if value.is_instance_of::<Promise>() => {
            let future = JsFuture::from(value.unchecked_into::<Promise>());
            HandlerResult::PendingFallible(Box::pin(async move {
                future
                    .await
                    .map(|value| value.as_bool() == Some(true))
                    .map_err(|error| DispatchError::Callback(callback_error(error)))
            }))
        }
        Ok(value) => HandlerResult::Ready(value.as_bool() == Some(true)),
    }
}

#[wasm_bindgen(js_name = ParserApi)]
pub struct WebParserApi {
    terminal: Rc<Terminal>,
}
impl WebParserApi {
    pub fn new(terminal: Rc<Terminal>) -> Self {
        Self { terminal }
    }
}
#[wasm_bindgen(js_class = ParserApi)]
impl WebParserApi {
    #[wasm_bindgen(js_name = registerCsiHandler)]
    pub fn register_csi_handler(
        &self,
        id: JsValue,
        callback: Function,
    ) -> Result<JsValue, JsValue> {
        let id = read_identifier(&id, Kind::Csi)?;
        let handle = self
            .terminal
            .with_parser(|api| {
                api.register_csi_packed(id.packed, id.source_guarded, move |params| {
                    result(callback.call1(&JsValue::UNDEFINED, &params_array(params)))
                })
            })
            .map_err(|error| to_js_error(&error))?
            .map_err(|error| to_js_error(&error))?;
        Ok(super::events::disposable(handle))
    }
    #[wasm_bindgen(js_name = addCsiHandler)]
    pub fn add_csi_handler(&self, id: JsValue, callback: Function) -> Result<JsValue, JsValue> {
        self.register_csi_handler(id, callback)
    }
    #[wasm_bindgen(js_name = registerDcsHandler)]
    pub fn register_dcs_handler(
        &self,
        id: JsValue,
        callback: Function,
    ) -> Result<JsValue, JsValue> {
        let id = read_identifier(&id, Kind::Dcs)?;
        let handle = self
            .terminal
            .with_parser(|api| {
                api.register_dcs_packed(id.packed, move |data, params| {
                    result(callback.call2(
                        &JsValue::UNDEFINED,
                        &to_js_string(&data),
                        &params_array(params),
                    ))
                })
            })
            .map_err(|error| to_js_error(&error))?
            .map_err(|error| to_js_error(&error))?;
        Ok(super::events::disposable(handle))
    }
    #[wasm_bindgen(js_name = addDcsHandler)]
    pub fn add_dcs_handler(&self, id: JsValue, callback: Function) -> Result<JsValue, JsValue> {
        self.register_dcs_handler(id, callback)
    }
    #[wasm_bindgen(js_name = registerEscHandler)]
    pub fn register_esc_handler(
        &self,
        id: JsValue,
        callback: Function,
    ) -> Result<JsValue, JsValue> {
        let id = read_identifier(&id, Kind::Esc)?;
        let handle = self
            .terminal
            .with_parser(|api| {
                api.register_esc_packed(id.packed, move || {
                    result(callback.call0(&JsValue::UNDEFINED))
                })
            })
            .map_err(|error| to_js_error(&error))?
            .map_err(|error| to_js_error(&error))?;
        Ok(super::events::disposable(handle))
    }
    #[wasm_bindgen(js_name = addEscHandler)]
    pub fn add_esc_handler(&self, id: JsValue, callback: Function) -> Result<JsValue, JsValue> {
        self.register_esc_handler(id, callback)
    }
    #[wasm_bindgen(js_name = registerOscHandler)]
    pub fn register_osc_handler(&self, id: f64, callback: Function) -> Result<JsValue, JsValue> {
        let handle = self
            .terminal
            .with_parser(|api| {
                api.register_osc_handler(id, move |data| {
                    result(callback.call1(&JsValue::UNDEFINED, &to_js_string(&data)))
                })
            })
            .map_err(|error| to_js_error(&error))?
            .map_err(|error| to_js_error(&error))?;
        Ok(super::events::disposable(handle))
    }
    #[wasm_bindgen(js_name = addOscHandler)]
    pub fn add_osc_handler(&self, id: f64, callback: Function) -> Result<JsValue, JsValue> {
        self.register_osc_handler(id, callback)
    }
    #[wasm_bindgen(js_name = registerApcHandler)]
    pub fn register_apc_handler(
        &self,
        id: JsValue,
        callback: Function,
    ) -> Result<JsValue, JsValue> {
        let id = read_identifier(&id, Kind::Apc)?;
        let handle = self
            .terminal
            .with_parser(|api| {
                api.register_apc_packed(id.packed, move |data| {
                    result(callback.call1(&JsValue::UNDEFINED, &to_js_string(&data)))
                })
            })
            .map_err(|error| to_js_error(&error))?
            .map_err(|error| to_js_error(&error))?;
        Ok(super::events::disposable(handle))
    }
}
