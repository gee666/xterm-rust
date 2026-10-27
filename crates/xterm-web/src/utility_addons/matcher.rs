// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. WebLinks RegExp/WHATWG built-in adapter.
use crate::utf16::{from_js_string, to_js_string};
use js_sys::{Array, Function, JsString, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::Url;
use xterm_addons::web_links::{LinkError, LinkMatch, LinkMatcher, RegexSpec};

pub struct BrowserLinkMatcher;
fn error(value: JsValue) -> LinkError {
    LinkError(format!("{value:?}"))
}
impl LinkMatcher for BrowserLinkMatcher {
    fn matches(&self, text: &[u16], regex: &RegexSpec) -> Result<Vec<LinkMatch>, LinkError> {
        let flags = if regex.flags.contains('g') {
            regex.flags.clone()
        } else {
            format!("{}g", regex.flags)
        };
        let constructor = Reflect::get(&js_sys::global(), &"RegExp".into())
            .map_err(error)?
            .dyn_into::<Function>()
            .map_err(error)?;
        let args = Array::new();
        args.push(&regex.source.clone().into());
        args.push(&flags.into());
        let regex = Reflect::construct(&constructor, &args).map_err(error)?;
        let exec = Reflect::get(&regex, &"exec".into())
            .map_err(error)?
            .dyn_into::<Function>()
            .map_err(error)?;
        let text = to_js_string(text);
        let mut result = Vec::new();
        loop {
            let found = exec.call1(&regex, &text).map_err(error)?;
            if found.is_null() {
                break;
            }
            let array = found.dyn_into::<Array>().map_err(error)?;
            let matched = array.get(0).dyn_into::<JsString>().map_err(error)?;
            if matched.length() == 0 {
                return Err(LinkError("custom URL regex matched an empty string".into()));
            }
            if !is_url(&matched) {
                continue;
            }
            let index = Reflect::get(array.as_ref(), &"index".into())
                .map_err(error)?
                .as_f64()
                .ok_or_else(|| LinkError("RegExp match has no index".into()))?;
            result.push(LinkMatch {
                index: index as usize,
                text: from_js_string(&matched),
            });
        }
        Ok(result)
    }
}
fn is_url(text: &JsString) -> bool {
    // URL replaces unpaired surrogates as part of its USVString conversion.
    let raw = String::from_utf16_lossy(&from_js_string(text));
    let Ok(url) = Url::new(&raw) else {
        return false;
    };
    let username = url.username();
    let password = url.password();
    let authority = if !password.is_empty() && !username.is_empty() {
        format!(
            "{}//{}:{}@{}",
            url.protocol(),
            username,
            password,
            url.host()
        )
    } else if !username.is_empty() {
        format!("{}//{}@{}", url.protocol(), username, url.host())
    } else {
        format!("{}//{}", url.protocol(), url.host())
    };
    let lower = text.to_locale_lower_case(None);
    let prefix = JsString::from(authority).to_locale_lower_case(None);
    // Comparing UTF-16 preserves odd strings in custom RegExp matches.
    from_js_string(&lower).starts_with(&from_js_string(&prefix))
}
