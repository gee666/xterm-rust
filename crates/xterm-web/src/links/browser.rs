// Copyright (c) 2022-2026 The xterm.js authors. All rights reserved. MIT license.
//! Browser URL and activation operations. Terminal URI storage remains raw UTF-16.
use crate::utf16::to_js_string;
use js_sys::{Function, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::Window;

pub fn is_http_url(uri: &[u16]) -> bool {
    // URL's constructor takes WebIDL USVString, which replaces lone surrogates.
    // This conversion is exact for that boundary, not a storage conversion.
    web_sys::Url::new(&String::from_utf16_lossy(uri))
        .is_ok_and(|url| matches!(url.protocol().as_str(), "http:" | "https:"))
}

/// Match the source default activation, including its confirmation and popup
/// warning. Custom link handlers bypass this function entirely.
pub fn default_activate(window: &Window, uri: &[u16]) -> Result<(), JsValue> {
    let mut prompt: Vec<u16> = "Do you want to navigate to ".encode_utf16().collect();
    prompt.extend_from_slice(uri);
    prompt.extend("?\n\nWARNING: This link could potentially be dangerous".encode_utf16());
    let confirm: Function =
        Reflect::get(window.as_ref(), &JsValue::from_str("confirm"))?.dyn_into()?;
    if !confirm
        .call1(window.as_ref(), &to_js_string(&prompt))?
        .is_truthy()
    {
        return Ok(());
    }
    let Some(new_window) = window.open()? else {
        web_sys::console::warn_1(&JsValue::from_str(
            "Opening link blocked as opener could not be cleared",
        ));
        return Ok(());
    };
    // Electron may throw when assigning opener. The source explicitly catches
    // that exception and still navigates; preserve this narrow compatibility case.
    let opener_result = Reflect::set(
        new_window.as_ref(),
        &JsValue::from_str("opener"),
        &JsValue::NULL,
    );
    match opener_result {
        Ok(_) | Err(_) => {}
    }
    let location = Reflect::get(new_window.as_ref(), &JsValue::from_str("location"))?;
    if !Reflect::set(&location, &JsValue::from_str("href"), &to_js_string(uri))? {
        return Err(JsValue::from_str("location.href assignment rejected"));
    }
    Ok(())
}
