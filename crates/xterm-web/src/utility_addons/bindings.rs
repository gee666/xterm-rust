//! Callable addon exports over the Rust-backed browser Terminal.
use super::{
    attach_terminal::BrowserAttachTerminal, clipboard::BrowserClipboardProvider,
    socket::BrowserSocket,
};
use crate::{
    terminal::Terminal,
    utf16::{from_js_string, to_js_string},
};
use js_sys::{Function, JsString, Promise, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use xterm_addons::{attach, clipboard, native_utility::NativeUtilityTerminal};
use xterm_core::lifecycle::Disposable;

#[wasm_bindgen(js_name=AttachAddon)]
pub struct AttachAddon {
    addon: attach::AttachAddon,
    terminals: RefCell<Vec<Rc<BrowserAttachTerminal>>>,
}
#[wasm_bindgen]
impl AttachAddon {
    #[wasm_bindgen(constructor)]
    pub fn new(socket: web_sys::WebSocket, options: JsValue) -> Result<AttachAddon, JsValue> {
        let bidirectional = if options.is_null() || options.is_undefined() {
            None
        } else {
            Some(Reflect::get(&options, &"bidirectional".into())?.as_bool() != Some(false))
        };
        let addon = attach::AttachAddon::new(Rc::new(BrowserSocket(socket)), bidirectional)
            .map_err(|error| crate::js_error::to_js_error(&error))?;
        Ok(Self {
            addon,
            terminals: RefCell::default(),
        })
    }
    pub fn activate(&self, terminal: &Terminal) -> Result<(), JsValue> {
        let native = NativeUtilityTerminal::from_headless(terminal.shared_native())
            .map_err(|e| js_sys::Error::new(&e.to_string()))?;
        let host = Rc::new(BrowserAttachTerminal {
            native: Rc::new(native),
            writer: terminal.queued_write_handle(),
        });
        self.terminals.borrow_mut().push(host.clone());
        self.addon
            .activate(host)
            .map_err(|error| crate::js_error::to_js_error(&error))
    }
    pub fn dispose(&self) {
        self.addon.dispose();
    }
}

fn clipboard_error(error: JsValue) -> clipboard::ClipboardError {
    clipboard::ClipboardError::host(crate::js_error::JsCallbackError(error))
}
fn call(
    object: &JsValue,
    method: &str,
    args: &[JsValue],
) -> Result<JsValue, clipboard::ClipboardError> {
    let method = Reflect::get(object, &method.into())
        .and_then(|v| v.dyn_into::<Function>())
        .map_err(clipboard_error)?;
    let array = js_sys::Array::new();
    for arg in args {
        array.push(arg);
    }
    method.apply(object, &array).map_err(clipboard_error)
}
fn string(value: JsValue) -> Result<Vec<u16>, clipboard::ClipboardError> {
    value
        .dyn_into::<JsString>()
        .map(|value| from_js_string(&value))
        .map_err(clipboard_error)
}
struct JsProvider(JsValue);
impl clipboard::ClipboardProvider for JsProvider {
    fn read_text(&self, selection: &[u16]) -> clipboard::Completion<Vec<u16>> {
        use clipboard::Completion;
        match call(&self.0, "readText", &[to_js_string(selection).into()]) {
            Err(error) => Completion::Ready(Err(error)),
            Ok(value) if value.is_instance_of::<Promise>() => {
                Completion::Pending(Box::pin(async move {
                    string(
                        JsFuture::from(value.unchecked_into::<Promise>())
                            .await
                            .map_err(clipboard_error)?,
                    )
                }))
            }
            Ok(value) => Completion::Ready(string(value)),
        }
    }
    fn write_text(&self, selection: &[u16], text: &[u16]) -> clipboard::Completion<()> {
        use clipboard::Completion;
        match call(
            &self.0,
            "writeText",
            &[to_js_string(selection).into(), to_js_string(text).into()],
        ) {
            Err(error) => Completion::Ready(Err(error)),
            Ok(value) if value.is_instance_of::<Promise>() => {
                Completion::Pending(Box::pin(async move {
                    JsFuture::from(value.unchecked_into::<Promise>())
                        .await
                        .map_err(clipboard_error)?;
                    Ok(())
                }))
            }
            Ok(_) => Completion::Ready(Ok(())),
        }
    }
}
struct JsCodec(JsValue);
impl clipboard::Base64Codec for JsCodec {
    fn encode_text(&self, text: &[u16]) -> Result<Vec<u16>, clipboard::ClipboardError> {
        string(call(&self.0, "encodeText", &[to_js_string(text).into()])?)
    }
    fn decode_text(&self, text: &[u16]) -> Result<Vec<u16>, clipboard::ClipboardError> {
        string(call(&self.0, "decodeText", &[to_js_string(text).into()])?)
    }
}

#[wasm_bindgen(js_name=ClipboardAddon)]
pub struct ClipboardAddon {
    addon: clipboard::ClipboardAddon,
    terminals: RefCell<Vec<Rc<NativeUtilityTerminal>>>,
}
#[wasm_bindgen]
impl ClipboardAddon {
    #[wasm_bindgen(constructor)]
    pub fn new(base64: JsValue, provider: JsValue) -> Result<ClipboardAddon, JsValue> {
        let codec = if base64.is_undefined() {
            None
        } else {
            Some(Rc::new(JsCodec(base64)) as Rc<dyn clipboard::Base64Codec>)
        };
        let provider: Rc<dyn clipboard::ClipboardProvider> = if provider.is_undefined() {
            let window = web_sys::window()
                .ok_or_else(|| js_sys::Error::new("no window for clipboard provider"))?;
            Rc::new(BrowserClipboardProvider::new(
                window.navigator().clipboard(),
            ))
        } else {
            Rc::new(JsProvider(provider))
        };
        Ok(Self {
            addon: clipboard::ClipboardAddon::new(provider, codec),
            terminals: RefCell::default(),
        })
    }
    pub fn activate(&self, terminal: &Terminal) -> Result<(), JsValue> {
        let host = Rc::new(
            NativeUtilityTerminal::from_headless(terminal.shared_native())
                .map_err(|e| js_sys::Error::new(&e.to_string()))?,
        );
        self.addon.activate(host.clone());
        self.terminals.borrow_mut().push(host);
        Ok(())
    }
    pub fn dispose(&self) {
        self.addon.dispose();
    }
}
#[wasm_bindgen(js_name=Base64)]
pub struct Base64;
#[wasm_bindgen]
impl Base64 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Base64 {
        Self
    }
    #[wasm_bindgen(js_name=encodeText)]
    pub fn encode_text(&self, text: JsString) -> JsString {
        JsString::from(xterm_addons::base64::encode_text(&from_js_string(&text)))
    }
    #[wasm_bindgen(js_name=decodeText)]
    pub fn decode_text(&self, text: JsString) -> JsString {
        to_js_string(&xterm_addons::base64::decode_text(&from_js_string(&text)))
    }
}
impl Default for Base64 {
    fn default() -> Self {
        Self::new()
    }
}
