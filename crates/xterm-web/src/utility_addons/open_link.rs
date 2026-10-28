// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Default WebLinksAddon activation behavior.
use crate::utf16::to_js_string;
use wasm_bindgen::{JsCast, JsValue};
use xterm_addons::web_links::LinkError;

pub fn open_link(uri: &[u16]) -> Result<(), LinkError> {
    let window =
        web_sys::window().ok_or_else(|| LinkError::Message("no window to open link".into()))?;
    let opened = window.open().map_err(error)?;
    if let Some(opened) = opened {
        // Upstream explicitly tolerates Electron throwing when clearing opener.
        let _electron_opener_result =
            js_sys::Reflect::set(opened.as_ref(), &"opener".into(), &JsValue::NULL);
        let location = js_sys::Reflect::get(opened.as_ref(), &"location".into()).map_err(error)?;
        let assigned =
            js_sys::Reflect::set(&location, &"href".into(), &to_js_string(uri)).map_err(error)?;
        if !assigned {
            return Err(LinkError::Message(
                "link window location rejected navigation".into(),
            ));
        }
    } else {
        let console = js_sys::Reflect::get(window.as_ref(), &"console".into()).map_err(error)?;
        let warn = js_sys::Reflect::get(&console, &"warn".into())
            .map_err(error)?
            .dyn_into::<js_sys::Function>()
            .map_err(error)?;
        warn.call1(
            &console,
            &"Opening link blocked as opener could not be cleared".into(),
        )
        .map_err(error)?;
    }
    Ok(())
}
fn error(value: JsValue) -> LinkError {
    LinkError::host(crate::js_error::JsCallbackError(value))
}
