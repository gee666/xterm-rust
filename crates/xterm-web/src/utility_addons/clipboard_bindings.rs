// Copyright (c) 2023 The xterm.js authors. All rights reserved. MIT license.
//! Raw JS values stay at the clipboard provider/codec FFI boundary. Rust owns
//! OSC parsing, base64, reply formatting and asynchronous parser continuation.
use crate::{
    terminal::Terminal,
    utf16::{from_js_string, to_js_string},
};
use js_sys::{Function, JsString, Reflect, Uint8Array};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use xterm_addons::clipboard::{parse_request, query_reply, ClipboardRequest};
use xterm_core::{core_service::CoreService, lifecycle::DisposableHandle, parser::osc::OscHandler};

// These are language/host calls only. Direct method syntax keeps the browser's
// TypeError, getter timing, this receiver and original thrown-value identity.
#[wasm_bindgen(inline_js = r#"
export function clipboardRead(provider, selection) {
  return provider === undefined ? navigator.clipboard.readText() : provider.readText(selection);
}
export function clipboardWrite(provider, selection, text) {
  return provider === undefined ? navigator.clipboard.writeText(text) : provider.writeText(selection, text);
}
export function clipboardEncode(codec, value) { return codec.encodeText(value); }
export function clipboardDecode(codec, value) { return codec.decodeText(value); }
export function clipboardString(value) { return `${value}`; }
export function clipboardEncoderInput(value) { return value === undefined ? '' : `${value}`; }
export function clipboardFromBase64(value) { return Uint8Array.fromBase64(value); }
export function clipboardAtob(value) { return atob(value); }
export function clipboardPromise(value) { return value instanceof Promise; }
export function clipboardReadThen(promise, invoke) { return promise.then(data => invoke(data)); }
export function clipboardWriteThen(promise) { return promise.then(() => true); }
"#)]
extern "C" {
    #[wasm_bindgen(catch,js_name=clipboardRead)]
    fn read(provider: &JsValue, selection: &JsString) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch,js_name=clipboardWrite)]
    fn write(provider: &JsValue, selection: &JsString, text: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch,js_name=clipboardEncode)]
    fn encode(codec: &JsValue, value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch,js_name=clipboardDecode)]
    fn decode(codec: &JsValue, value: &JsString) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch,js_name=clipboardString)]
    fn abstract_string(value: &JsValue) -> Result<JsString, JsValue>;
    #[wasm_bindgen(catch,js_name=clipboardEncoderInput)]
    pub fn encoder_input(value: &JsValue) -> Result<JsString, JsValue>;
    #[wasm_bindgen(catch,js_name=clipboardFromBase64)]
    fn from_base64(value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch,js_name=clipboardAtob)]
    fn atob(value: &JsValue) -> Result<JsString, JsValue>;
    #[wasm_bindgen(catch,js_name=clipboardPromise)]
    fn is_promise(value: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch,js_name=clipboardReadThen)]
    fn read_then(value: &JsValue, invoke: &Function) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch,js_name=clipboardWriteThen)]
    fn write_then(value: &JsValue) -> Result<JsValue, JsValue>;
}
pub fn decode_default(value: &JsValue) -> Result<Vec<u16>, JsValue> {
    // The source checks this property outside its inner try/catch, then reads it
    // again for invocation. Keep host capability/getter errors at those points.
    let constructor = Reflect::get(&js_sys::global(), &"Uint8Array".into())?;
    let modern = Reflect::get(&constructor, &"fromBase64".into())?;
    let bytes = if modern.is_undefined() {
        match atob(value) {
            Ok(text) => from_js_string(&text)
                .into_iter()
                .map(|unit| unit as u8)
                .collect(),
            Err(_) => return Ok(Vec::new()),
        }
    } else {
        match from_base64(value).and_then(|value| value.dyn_into::<Uint8Array>()) {
            Ok(bytes) => bytes.to_vec(),
            Err(_) => return Ok(Vec::new()),
        }
    };
    let text = String::from_utf8_lossy(&bytes);
    Ok(text
        .strip_prefix('\u{feff}')
        .unwrap_or(&text)
        .encode_utf16()
        .collect())
}
#[wasm_bindgen(js_name=BrowserClipboardProvider)]
#[derive(Default)]
pub struct BrowserClipboardProvider;
#[wasm_bindgen]
impl BrowserClipboardProvider {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }
    #[wasm_bindgen(js_name=readText)]
    pub fn read_text_js(&self, _selection: JsValue) -> Result<JsValue, JsValue> {
        // This provider intentionally redirects every selection to navigator.
        // Do not inspect or coerce the ignored selection argument.
        read(&JsValue::UNDEFINED, &JsString::from(""))
    }
    #[wasm_bindgen(js_name=writeText)]
    pub fn write_text_js(&self, _selection: JsValue, text: JsValue) -> Result<JsValue, JsValue> {
        write(&JsValue::UNDEFINED, &JsString::from(""), &text)
    }
}
struct State {
    codec: JsValue,
    provider: JsValue,
    target: RefCell<Option<Rc<CoreService>>>,
}
impl State {
    fn reply(&self, selection: &[u16], data: JsValue) -> Result<bool, JsValue> {
        let encoded = encode(&self.codec, &data)?;
        // JS selects the optional-call receiver before interpolating b64. A
        // custom encoded value may activate another terminal in its toString.
        let target = self
            .target
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::Error::new("Clipboard addon is not activated"))?;
        let encoded = abstract_string(&encoded)?;
        let reply = query_reply(selection, &from_js_string(&encoded));
        target
            .trigger_data_event(&reply, false)
            .map_err(|error| crate::js_error::to_js_error(&error))?;
        Ok(true)
    }
    fn handle(self: &Rc<Self>, data: &[u16]) -> Result<JsValue, JsValue> {
        match parse_request(data) {
            ClipboardRequest::Ignored => Ok(JsValue::TRUE),
            ClipboardRequest::Read(selection) => {
                let value = read(&self.provider, &to_js_string(selection))?;
                if is_promise(&value)? {
                    let state = self.clone();
                    let selection = selection.to_vec();
                    let invoke =
                        Closure::<dyn Fn(JsValue) -> Result<bool, JsValue>>::new(move |data| {
                            state.reply(&selection, data)
                        })
                        .into_js_value();
                    read_then(&value, invoke.unchecked_ref())
                } else {
                    self.reply(selection, value).map(JsValue::from_bool)
                }
            }
            ClipboardRequest::Write { selection, payload } => {
                // Decode failures alone clear, provider failures propagate.
                let text = decode(&self.codec, &to_js_string(payload))
                    .unwrap_or_else(|_| JsString::from("").into());
                let value = write(&self.provider, &to_js_string(selection), &text)?;
                if is_promise(&value)? {
                    write_then(&value)
                } else {
                    Ok(JsValue::TRUE)
                }
            }
        }
    }
}
#[wasm_bindgen(js_name=ClipboardAddon)]
pub struct ClipboardAddon {
    state: Rc<State>,
    registration: RefCell<Option<DisposableHandle>>,
}
#[wasm_bindgen]
impl ClipboardAddon {
    #[wasm_bindgen(constructor)]
    pub fn new(codec: JsValue, provider: JsValue) -> ClipboardAddon {
        let codec = if codec.is_undefined() {
            super::bindings::Base64::new().into()
        } else {
            codec
        };
        let provider = if provider.is_undefined() {
            BrowserClipboardProvider::new().into()
        } else {
            provider
        };
        Self {
            state: Rc::new(State {
                codec,
                provider,
                target: RefCell::default(),
            }),
            registration: RefCell::default(),
        }
    }
    pub fn activate(&self, terminal: &Terminal) -> Result<(), JsValue> {
        let native = terminal.shared_native();
        let target = native
            .with_state(|state| state.core.clone())
            .map_err(|error| crate::js_error::to_js_error(&error))?;
        *self.state.target.borrow_mut() = Some(target);
        let (osc, _) = native
            .addon_registrars()
            .map_err(|error| crate::js_error::to_js_error(&error))?;
        let state = self.state.clone();
        let registration = osc
            .register_handler(
                f64::from(xterm_addons::clipboard::OSC_CLIPBOARD),
                Rc::new(RefCell::new(OscHandler::new(move |data| {
                    crate::terminal::parser_api::result(state.handle(&data))
                }))),
            )
            .ok_or_else(|| js_sys::Error::new("Clipboard parser registry was dropped"))?;
        *self.registration.borrow_mut() = Some(registration);
        Ok(())
    }
    pub fn dispose(&self) {
        let registration = self.registration.borrow().clone();
        if let Some(registration) = registration {
            registration.dispose();
        }
    }
}
impl Drop for ClipboardAddon {
    fn drop(&mut self) {
        self.dispose();
    }
}
