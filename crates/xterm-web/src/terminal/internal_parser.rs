//! Streaming internal parser adapters. Payload decoding and dispatch stay in Rust.
use crate::js_error::{callback_error, to_js_error};
use js_sys::{Array, Function, Int32Array, Reflect, Uint32Array};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use xterm_core::{
    parser::{
        dcs::DcsHandlerTrait,
        handler::{DispatchError, HandlerResult},
        osc::OscHandlerTrait,
        params::{ParamValue, Params},
    },
    public_api::parser::ParserRegistrations,
};
use xterm_wasm_support::public_api::parser_api::{read_identifier, result, Kind};

struct Stream(JsValue);
impl Stream {
    fn call(&self, name: &str, arguments: &Array) -> Result<JsValue, JsValue> {
        Reflect::get(&self.0, &name.into())?
            .dyn_into::<Function>()
            .map_err(|_| js_sys::TypeError::new(&format!("handler.{name} is not a function")))?
            .apply(&self.0, arguments)
    }
    fn empty(&self, name: &str) -> Result<(), DispatchError> {
        self.call(name, &Array::new())
            .map(|_| ())
            .map_err(|e| DispatchError::Callback(callback_error(e)))
    }
    fn data(&self, data: &[u32]) -> Result<(), DispatchError> {
        let args = Array::new();
        args.push(&Uint32Array::from(data));
        args.push(&0.into());
        args.push(&(data.len() as f64).into());
        self.call("put", &args)
            .map(|_| ())
            .map_err(|e| DispatchError::Callback(callback_error(e)))
    }
    fn finish(&self, name: &str, success: bool) -> HandlerResult {
        result(self.call(name, &Array::of1(&success.into())))
    }
}
impl DcsHandlerTrait for Stream {
    fn hook(&mut self, params: &Params) -> Result<(), DispatchError> {
        let params: JsValue = InternalParams(params.clone()).into();
        self.call("hook", &Array::of1(&params))
            .map(|_| ())
            .map_err(|e| DispatchError::Callback(callback_error(e)))
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        self.data(data)
    }
    fn unhook(&mut self, success: bool) -> HandlerResult {
        self.finish("unhook", success)
    }
}
impl OscHandlerTrait for Stream {
    fn start(&mut self) -> Result<(), DispatchError> {
        self.empty("start")
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        self.data(data)
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        self.finish("end", success)
    }
}
/// Params are owned callback snapshots, avoiding a buffer borrow across JS code.
#[wasm_bindgen]
pub struct InternalParams(Params);
#[wasm_bindgen]
impl InternalParams {
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        self.0.len()
    }
    #[wasm_bindgen(getter)]
    pub fn params(&self) -> Int32Array {
        Int32Array::from(self.0.values())
    }
    #[wasm_bindgen(js_name = hasSubParams)]
    pub fn has_sub(&self, index: usize) -> bool {
        self.0.has_sub_params(index)
    }
    #[wasm_bindgen(js_name = getSubParams)]
    pub fn sub(&self, index: usize) -> JsValue {
        self.0
            .get_sub_params(index)
            .map_or(JsValue::NULL, |p| Int32Array::from(p).into())
    }
    #[wasm_bindgen(js_name = toArray)]
    pub fn to_array(&self) -> Array {
        self.0
            .to_array()
            .into_iter()
            .map(|p| -> JsValue {
                match p {
                    ParamValue::Param(v) => v.into(),
                    ParamValue::SubParams(v) => {
                        v.into_iter().map(JsValue::from).collect::<Array>().into()
                    }
                }
            })
            .collect()
    }
}

#[wasm_bindgen(js_name = EscapeSequenceParser)]
pub struct InternalParser {
    registrations: ParserRegistrations,
}
impl InternalParser {
    pub(super) fn new(registrations: ParserRegistrations) -> Self {
        Self { registrations }
    }
}
#[wasm_bindgen(js_class = EscapeSequenceParser)]
impl InternalParser {
    #[wasm_bindgen(js_name = registerDcsHandler)]
    pub fn dcs(&self, id: JsValue, handler: JsValue) -> Result<JsValue, JsValue> {
        let id = read_identifier(&id, Kind::Dcs)?.packed;
        let handle = self
            .registrations
            .dcs_registrar()
            .register_handler(id as i32, Rc::new(RefCell::new(Stream(handler))))
            .ok_or_else(|| js_sys::Error::new("Parser has been dropped"))?;
        Ok(xterm_wasm_support::events::disposable(handle))
    }
    #[wasm_bindgen(js_name = registerOscHandler)]
    pub fn osc(&self, id: f64, handler: JsValue) -> Result<JsValue, JsValue> {
        let handle = self
            .registrations
            .osc_registrar()
            .register_handler(id, Rc::new(RefCell::new(Stream(handler))))
            .ok_or_else(|| js_sys::Error::new("Parser has been dropped"))?;
        Ok(xterm_wasm_support::events::disposable(handle))
    }
    #[wasm_bindgen(js_name = registerApcHandler)]
    pub fn apc(&self, id: JsValue, handler: JsValue) -> Result<JsValue, JsValue> {
        let id = read_identifier(&id, Kind::Apc)?.packed;
        let handle = self
            .registrations
            .apc_registrar()
            .register_handler(id, Rc::new(RefCell::new(Stream(handler))))
            .ok_or_else(|| js_sys::Error::new("Parser has been dropped"))?;
        Ok(xterm_wasm_support::events::disposable(handle))
    }
    #[wasm_bindgen(js_name = registerCsiHandler)]
    pub fn csi(&self, id: JsValue, callback: Function) -> Result<JsValue, JsValue> {
        // Direct parser registration has no InputHandler window-policy pre-read.
        let id = read_identifier(&id, Kind::Dcs)?.packed;
        let mut api =
            xterm_core::public_api::parser::ParserApi::from_registrations(&self.registrations);
        let handle = api
            .register_csi_packed(id, false, move |values| {
                let params = match Params::from_array(&values) {
                    Ok(params) => params,
                    Err(error) => {
                        return HandlerResult::Failed(DispatchError::Callback(callback_error(
                            to_js_error(&error),
                        )))
                    }
                };
                let params: JsValue = InternalParams(params).into();
                result(callback.call1(&JsValue::UNDEFINED, &params))
            })
            .map_err(|e| to_js_error(&e))?;
        Ok(xterm_wasm_support::events::disposable(handle))
    }
}
