//! JavaScript public parser callbacks over the Rust terminal engine.
use crate::{
    js_error::{callback_error, to_js_error},
    utf16::to_js_string,
};
use js_sys::{Array, Function, Promise, Reflect};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use xterm_core::{
    headless::Terminal,
    lifecycle::DisposableHandle,
    parser::{
        escape::FunctionIdentifier,
        handler::{DispatchError, HandlerResult},
        params::ParamValue,
    },
};

#[wasm_bindgen(js_name = ParserRegistration)]
pub struct ApiDisposable {
    handle: DisposableHandle,
}
impl ApiDisposable {
    pub(crate) fn new(handle: DisposableHandle) -> Self {
        Self { handle }
    }
}
#[wasm_bindgen]
impl ApiDisposable {
    pub fn dispose(&self) {
        self.handle.dispose();
    }
}

struct Identifier {
    prefix: Option<String>,
    intermediates: Option<String>,
    final_byte: String,
}
impl Identifier {
    fn read(value: &JsValue) -> Result<Self, JsValue> {
        fn property(value: &JsValue, name: &str) -> Result<Option<String>, JsValue> {
            let field = Reflect::get(value, &name.into())?;
            if field.is_null() || field.is_undefined() {
                return Ok(None);
            }
            field
                .as_string()
                .map(Some)
                .ok_or_else(|| js_sys::TypeError::new("identifier fields must be strings").into())
        }
        Ok(Self {
            prefix: property(value, "prefix")?,
            intermediates: property(value, "intermediates")?,
            final_byte: property(value, "final")?.unwrap_or_default(),
        })
    }
    fn borrowed(&self) -> FunctionIdentifier<'_> {
        FunctionIdentifier {
            prefix: self.prefix.as_deref(),
            intermediates: self.intermediates.as_deref(),
            final_byte: &self.final_byte,
        }
    }
}
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
pub(crate) fn result(value: Result<JsValue, JsValue>) -> HandlerResult {
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
#[wasm_bindgen]
impl WebParserApi {
    #[wasm_bindgen(js_name = registerCsiHandler)]
    pub fn register_csi_handler(
        &self,
        id: JsValue,
        callback: Function,
    ) -> Result<ApiDisposable, JsValue> {
        let id = Identifier::read(&id)?;
        let handle = self
            .terminal
            .with_parser(|api| {
                api.register_csi_handler(id.borrowed(), move |params| {
                    result(callback.call1(&JsValue::UNDEFINED, &params_array(params)))
                })
            })
            .map_err(|error| to_js_error(&error))?
            .map_err(|error| to_js_error(&error))?;
        Ok(ApiDisposable::new(handle))
    }
    #[wasm_bindgen(js_name = addCsiHandler)]
    pub fn add_csi_handler(
        &self,
        id: JsValue,
        callback: Function,
    ) -> Result<ApiDisposable, JsValue> {
        self.register_csi_handler(id, callback)
    }
    #[wasm_bindgen(js_name = registerDcsHandler)]
    pub fn register_dcs_handler(
        &self,
        id: JsValue,
        callback: Function,
    ) -> Result<ApiDisposable, JsValue> {
        let id = Identifier::read(&id)?;
        let handle = self
            .terminal
            .with_parser(|api| {
                api.register_dcs_handler(id.borrowed(), move |data, params| {
                    result(callback.call2(
                        &JsValue::UNDEFINED,
                        &to_js_string(&data),
                        &params_array(params),
                    ))
                })
            })
            .map_err(|error| to_js_error(&error))?
            .map_err(|error| to_js_error(&error))?;
        Ok(ApiDisposable::new(handle))
    }
    #[wasm_bindgen(js_name = addDcsHandler)]
    pub fn add_dcs_handler(
        &self,
        id: JsValue,
        callback: Function,
    ) -> Result<ApiDisposable, JsValue> {
        self.register_dcs_handler(id, callback)
    }
    #[wasm_bindgen(js_name = registerEscHandler)]
    pub fn register_esc_handler(
        &self,
        id: JsValue,
        callback: Function,
    ) -> Result<ApiDisposable, JsValue> {
        let id = Identifier::read(&id)?;
        let handle = self
            .terminal
            .with_parser(|api| {
                api.register_esc_handler(id.borrowed(), move || {
                    result(callback.call0(&JsValue::UNDEFINED))
                })
            })
            .map_err(|error| to_js_error(&error))?
            .map_err(|error| to_js_error(&error))?;
        Ok(ApiDisposable::new(handle))
    }
    #[wasm_bindgen(js_name = addEscHandler)]
    pub fn add_esc_handler(
        &self,
        id: JsValue,
        callback: Function,
    ) -> Result<ApiDisposable, JsValue> {
        self.register_esc_handler(id, callback)
    }
    #[wasm_bindgen(js_name = registerOscHandler)]
    pub fn register_osc_handler(
        &self,
        id: f64,
        callback: Function,
    ) -> Result<ApiDisposable, JsValue> {
        let handle = self
            .terminal
            .with_parser(|api| {
                api.register_osc_handler(id, move |data| {
                    result(callback.call1(&JsValue::UNDEFINED, &to_js_string(&data)))
                })
            })
            .map_err(|error| to_js_error(&error))?
            .map_err(|error| to_js_error(&error))?;
        Ok(ApiDisposable::new(handle))
    }
    #[wasm_bindgen(js_name = addOscHandler)]
    pub fn add_osc_handler(&self, id: f64, callback: Function) -> Result<ApiDisposable, JsValue> {
        self.register_osc_handler(id, callback)
    }
    #[wasm_bindgen(js_name = registerApcHandler)]
    pub fn register_apc_handler(
        &self,
        id: JsValue,
        callback: Function,
    ) -> Result<ApiDisposable, JsValue> {
        let id = Identifier::read(&id)?;
        let handle = self
            .terminal
            .with_parser(|api| {
                api.register_apc_handler(id.borrowed(), move |data| {
                    result(callback.call1(&JsValue::UNDEFINED, &to_js_string(&data)))
                })
            })
            .map_err(|error| to_js_error(&error))?
            .map_err(|error| to_js_error(&error))?;
        Ok(ApiDisposable::new(handle))
    }
}
