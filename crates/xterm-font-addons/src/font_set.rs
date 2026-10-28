// Copyright (c) 2024 The xterm.js authors. All rights reserved. MIT license.
//! Real FontFaceSet loading. Selection remains ordered, including partial adds
//! before a later unknown family rejects the request.

use crate::utf16::from_js_string;
use js_sys::{Array, JsString, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Document, FontFace, FontFaceSet};
use xterm_addons::web_fonts::{unquote, FontDescriptor};

pub fn font_property(face: &FontFace, name: &str) -> Result<Vec<u16>, JsValue> {
    let value = Reflect::get(face, &JsValue::from_str(name))?.dyn_into::<JsString>()?;
    Ok(from_js_string(&value))
}

fn descriptor(face: &FontFace) -> Result<FontDescriptor, JsValue> {
    Ok(FontDescriptor {
        family: font_property(face, "family")?,
        stretch: font_property(face, "stretch")?,
        style: font_property(face, "style")?,
        unicode_range: font_property(face, "unicodeRange")?,
        weight: font_property(face, "weight")?,
    }
    .normalized())
}

pub fn registered_faces(set: &FontFaceSet) -> Result<Vec<FontFace>, JsValue> {
    Array::from(set.as_ref())
        .iter()
        .map(|value| value.dyn_into::<FontFace>())
        .collect()
}

pub async fn wait_ready(document: &Document) -> Result<(), JsValue> {
    JsFuture::from(document.fonts().ready()?).await?;
    Ok(())
}

/// Caller has already awaited document.fonts.ready, as relayout must enumerate
/// and select without an additional asynchronous gap.
pub async fn load_selected(
    document: &Document,
    requests: Option<&Array>,
) -> Result<Array, JsValue> {
    let set = document.fonts();
    let mut faces = registered_faces(&set)?;
    let mut selected = Vec::new();
    if let Some(requests) = requests.filter(|items| items.length() != 0) {
        let mut hashes = faces
            .iter()
            .map(descriptor)
            .collect::<Result<Vec<_>, _>>()?;
        for request in requests.iter() {
            if let Some(face) = request.dyn_ref::<FontFace>() {
                let hash = descriptor(face)?;
                if let Some(index) = hashes.iter().position(|value| value == &hash) {
                    selected.push(faces[index].clone());
                } else {
                    set.add(face)?;
                    faces.push(face.clone());
                    hashes.push(hash);
                    selected.push(face.clone());
                }
            } else {
                let family = request
                    .dyn_ref::<JsString>()
                    .ok_or_else(|| js_sys::TypeError::new("Expected font family or FontFace"))?;
                let name = from_js_string(family);
                let mut matches = Vec::new();
                for face in &faces {
                    if unquote(&font_property(face, "family")?) == name {
                        matches.push(face.clone());
                    }
                }
                if matches.is_empty() {
                    let message = JsString::from("font family \"")
                        .concat(family)
                        .concat(&JsString::from("\" not registered in document.fonts"));
                    return Err(message.into());
                }
                selected.extend(matches);
            }
        }
    } else {
        selected = faces;
    }
    let promises = Array::new();
    // Start every load before awaiting any one of them, like Promise.all.
    for face in selected {
        promises.push(face.load()?.as_ref());
    }
    let loaded = JsFuture::from(Promise::all(&promises)).await?;
    Ok(Array::from(&loaded))
}

pub async fn load_fonts(document: &Document, requests: Option<&Array>) -> Result<Array, JsValue> {
    wait_ready(document).await?;
    load_selected(document, requests).await
}
