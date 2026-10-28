// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. WebLinks RegExp/WHATWG built-in adapter.
use crate::utf16::{from_js_string, to_js_string};
use js_sys::{Array, Function, JsString, Reflect};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::Url;
use xterm_addons::web_links::{LinkError, LinkMatch, LinkMatcher, PreparedMatcher, RegexSpec};

#[wasm_bindgen(
    inline_js = "export function prepareLinkRegExp(regex) { const flags = regex.flags.includes('g') ? regex.flags : `${regex.flags}g`; return new RegExp(regex.source, flags); }"
)]
extern "C" {
    // Native language object access/coercion remains at this host boundary.
    #[wasm_bindgen(catch,js_name=prepareLinkRegExp)]
    fn prepare_object(regex: &JsValue) -> Result<JsValue, JsValue>;
}
pub struct BrowserLinkMatcher;
pub struct ObjectLinkMatcher(JsValue);
impl ObjectLinkMatcher {
    pub fn new(regex: JsValue) -> Self {
        Self(regex)
    }
}
struct PreparedBrowserMatcher(JsValue);
fn error(value: JsValue) -> LinkError {
    LinkError::host(crate::js_error::JsCallbackError(value))
}
impl LinkMatcher for BrowserLinkMatcher {
    fn matches(&self, text: &[u16], regex: &RegexSpec) -> Result<Vec<LinkMatch>, LinkError> {
        self.prepare(regex)?.matches(text)
    }
    fn prepare(&self, regex: &RegexSpec) -> Result<Box<dyn PreparedMatcher + '_>, LinkError> {
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
        Ok(Box::new(PreparedBrowserMatcher(
            Reflect::construct(&constructor, &args).map_err(error)?,
        )))
    }
}
impl LinkMatcher for ObjectLinkMatcher {
    fn matches(&self, text: &[u16], regex: &RegexSpec) -> Result<Vec<LinkMatch>, LinkError> {
        self.prepare(regex)?.matches(text)
    }
    fn prepare(&self, _: &RegexSpec) -> Result<Box<dyn PreparedMatcher + '_>, LinkError> {
        Ok(Box::new(PreparedBrowserMatcher(
            prepare_object(&self.0).map_err(error)?,
        )))
    }
}
impl PreparedMatcher for PreparedBrowserMatcher {
    fn matches(&self, text: &[u16]) -> Result<Vec<LinkMatch>, LinkError> {
        let regex = &self.0;
        let exec = Reflect::get(regex, &"exec".into())
            .map_err(error)?
            .dyn_into::<Function>()
            .map_err(error)?;
        let text = to_js_string(text);
        let mut result = Vec::new();
        let mut previous_end = 0.0;
        loop {
            let found = exec.call1(regex, &text).map_err(error)?;
            if found.is_null() {
                break;
            }
            let array = found.dyn_into::<Array>().map_err(error)?;
            let matched = array.get(0).dyn_into::<JsString>().map_err(error)?;
            if matched.length() == 0 {
                return Err(LinkError::Message(
                    "custom URL regex matched an empty string".into(),
                ));
            }
            let next_end = Reflect::get(regex, &"lastIndex".into())
                .map_err(error)?
                .as_f64()
                .ok_or_else(|| LinkError::Message("URL RegExp has no numeric lastIndex".into()))?;
            if !next_end.is_finite()
                || next_end.fract() != 0.0
                || next_end <= previous_end
                || next_end > f64::from(text.length())
            {
                return Err(LinkError::Message(
                    "URL RegExp did not advance within the input".into(),
                ));
            }
            previous_end = next_end;
            if !is_url(&matched) {
                continue;
            }
            let index = Reflect::get(array.as_ref(), &"index".into())
                .map_err(error)?
                .as_f64()
                .ok_or_else(|| LinkError::Message("RegExp match has no index".into()))?;
            result.push(LinkMatch {
                index: index as usize,
                text: from_js_string(&matched),
            });
        }
        Ok(result)
    }
}
fn is_url(text: &JsString) -> bool {
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
    from_js_string(&lower).starts_with(&from_js_string(&prefix))
}
