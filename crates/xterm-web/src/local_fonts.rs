// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Local-font discovery, independent of GSUB parsing. A browser terminal service
//! owns this shared catalog; clones share cached metadata without global state.

use crate::utf16::to_js_string;
use js_sys::{Array, ArrayBuffer, Function, Object, Promise, Reflect, Uint8Array};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use xterm_addons::ligatures::parse::{parse_families, ParseError};

#[derive(Clone, Default)]
pub struct LocalFonts {
    cache: Rc<RefCell<Option<JsValue>>>,
}

pub struct LocalFont {
    pub family: Vec<u16>,
    pub bytes: Vec<u8>,
}

fn get(object: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    Reflect::get(object, &JsValue::from_str(key))
}

fn call(object: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    let function = get(object, name)?
        .dyn_into::<Function>()
        .map_err(|_| js_sys::TypeError::new(&format!("{name} is not a function")))?;
    function.call0(object)
}

async fn await_value(value: JsValue) -> Result<JsValue, JsValue> {
    JsFuture::from(Promise::resolve(&value)).await
}

fn has_own(object: &JsValue, key: &JsValue) -> Result<bool, JsValue> {
    Ok(get(object, "hasOwnProperty")?
        .dyn_into::<Function>()?
        .call1(object, key)?
        .is_truthy())
}

async fn permission(navigator: &JsValue) -> Result<(), JsValue> {
    let permissions = get(navigator, "permissions")?;
    let request = get(&permissions, "request")?;
    if request.is_undefined() || request.is_null() {
        return Ok(());
    }
    let descriptor = Object::new();
    Reflect::set(&descriptor, &"name".into(), &"local-fonts".into())?;
    let status = await_value(
        request
            .dyn_into::<Function>()?
            .call1(&permissions, &descriptor)?,
    )
    .await?;
    if status.is_truthy() && get(&status, "state")? != JsValue::from_str("granted") {
        return Err(js_sys::Error::new("Permission to access local fonts not granted.").into());
    }
    Ok(())
}

fn log_query_error(error: &JsValue) -> Result<(), JsValue> {
    if error.is_instance_of::<js_sys::Error>() {
        web_sys::console::error_2(&get(error, "name")?, &get(error, "message")?);
    }
    Ok(())
}

async fn group_query(owner: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    let metadata = await_value(call(owner, name)?).await?;
    let iterator = js_sys::try_iter(&metadata)?
        .ok_or_else(|| js_sys::TypeError::new("Font metadata is not iterable"))?;
    let grouped = Object::new();
    for entry in iterator {
        let entry = entry?;
        let family = get(&entry, "family")?;
        if !has_own(&grouped, &family)? {
            Reflect::set(&grouped, &family, &Array::new())?;
        }
        let family_entries = Reflect::get(&grouped, &family)?.dyn_into::<Array>()?;
        family_entries.push(&entry);
    }
    Ok(grouped.into())
}

impl LocalFonts {
    async fn catalog(&self, global: &JsValue) -> Result<JsValue, JsValue> {
        if let Some(cached) = self.cache.borrow().clone() {
            return Ok(cached);
        }
        let navigator = get(global, "navigator")?;
        let legacy =
            !navigator.is_undefined() && Reflect::has(&navigator, &JsValue::from_str("fonts"))?;
        let grouped = if legacy {
            if let Err(error) = permission(&navigator).await {
                // The source ignores non-Error throws and TypeErrors because
                // older browser permission implementations reject this name.
                if error.is_instance_of::<js_sys::Error>()
                    && get(&error, "name")? != JsValue::from_str("TypeError")
                {
                    return Err(error);
                }
            }
            async {
                let fonts = get(&navigator, "fonts")?;
                group_query(&fonts, "query").await
            }
            .await
        } else {
            let window = get(global, "window")?;
            if !window.is_undefined()
                && Reflect::has(&window, &JsValue::from_str("queryLocalFonts"))?
            {
                group_query(&window, "queryLocalFonts").await
            } else {
                Ok(Object::new().into())
            }
        };
        let grouped = match grouped {
            Ok(value) => value,
            Err(error) => {
                log_query_error(&error)?;
                Object::new().into()
            }
        };
        // As in the source, permission failures are not cached, but query
        // failures cache an empty catalog. Concurrent first calls may both query.
        *self.cache.borrow_mut() = Some(grouped.clone());
        Ok(grouped)
    }

    pub async fn load(&self, family: &[u16]) -> Result<Option<LocalFont>, JsValue> {
        self.load_from(&js_sys::global(), family).await
    }

    /// An explicit environment also makes legacy/current browser branches
    /// testable without replacing a browser's actual permission objects.
    pub async fn load_from(
        &self,
        global: &JsValue,
        family: &[u16],
    ) -> Result<Option<LocalFont>, JsValue> {
        let fonts = self.catalog(global).await?;
        let families = parse_families(family).map_err(|error| -> JsValue {
            match error {
                ParseError::UnterminatedString => js_sys::Error::new(&error.to_string()).into(),
                ParseError::InvalidCodePoint(_) => {
                    js_sys::RangeError::new(&error.to_string()).into()
                }
            }
        })?;
        for family in families {
            if is_generic(&family) {
                return Ok(None);
            }
            let key = to_js_string(&family);
            if has_own(&fonts, &key)? {
                let entries = Reflect::get(&fonts, &key)?.dyn_into::<Array>()?;
                if entries.length() == 0 {
                    continue;
                }
                let font = entries.get(0);
                if Reflect::has(&font, &JsValue::from_str("blob"))? {
                    let blob = await_value(call(&font, "blob")?).await?;
                    let buffer = await_value(call(&blob, "arrayBuffer")?)
                        .await?
                        .dyn_into::<ArrayBuffer>()?;
                    return Ok(Some(LocalFont {
                        family,
                        bytes: Uint8Array::new(&buffer).to_vec(),
                    }));
                }
                return Ok(None);
            }
        }
        Ok(None)
    }
}

fn is_generic(family: &[u16]) -> bool {
    const GENERIC: &[&str] = &[
        "serif",
        "sans-serif",
        "cursive",
        "fantasy",
        "monospace",
        "system-ui",
        "emoji",
        "math",
        "fangsong",
    ];
    GENERIC
        .iter()
        .any(|value| value.encode_utf16().eq(family.iter().copied()))
}
