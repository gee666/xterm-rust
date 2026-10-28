// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Arrow DOM construction keeps source default-hint + 'px' conversion separate
//! from vertical scrollbar updates, whose template strings use String hint.
use super::ArrowOptions;
use crate::{
    scroll::{
        fast_dom_node::{assign_style, begin_style},
        options::{concat_pixels, number},
    },
    terminal::options::into_js,
};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::HtmlElement;

#[wasm_bindgen(
    inline_js = "export function arrowClass(node, value) { 'use strict'; node.className=value; }"
)]
extern "C" {
    #[wasm_bindgen(catch,js_name=arrowClass)]
    fn class_name(node: &HtmlElement, value: &JsValue) -> Result<(), JsValue>;
}
fn style(node: &HtmlElement, key: &str, value: &JsValue) -> Result<(), JsValue> {
    let target = begin_style(node)?;
    assign_style(&target, key, value)
}
fn pixels(
    node: &HtmlElement,
    key: &str,
    value: &xterm_core::options::OptionValue,
) -> Result<(), JsValue> {
    let target = begin_style(node)?;
    let value = concat_pixels(value)?;
    assign_style(&target, key, value.as_ref())
}
fn present(value: &Option<xterm_core::options::OptionValue>) -> bool {
    !matches!(
        value,
        None | Some(xterm_core::options::OptionValue::Undefined)
    )
}
pub(super) fn nodes(options: &ArrowOptions) -> Result<(HtmlElement, HtmlElement), JsValue> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| js_sys::Error::new("scrollbar arrow requires a document"))?;
    let background = document.create_element("div")?.dyn_into::<HtmlElement>()?;
    class_name(&background, &"xterm-arrow-background".into())?;
    style(&background, "position", &"absolute".into())?;
    pixels(&background, "width", &options.bg_width)?;
    pixels(&background, "height", &options.bg_height)?;
    for (key, value) in [
        ("top", &options.top),
        ("left", &options.left),
        ("bottom", &options.bottom),
        ("right", &options.right),
    ] {
        if present(value) {
            style(&background, key, &"0px".into())?;
        }
    }
    let node = document.create_element("div")?.dyn_into::<HtmlElement>()?;
    class_name(&node, &into_js(&options.class_name)?)?;
    style(&node, "position", &"absolute".into())?;
    let width = number(&options.bg_width)?;
    let height = number(&options.bg_height)?;
    let size = crate::scroll::math::min(width, height);
    pixels(&node, "width", &size.into())?;
    pixels(&node, "height", &size.into())?;
    for (key, value) in [
        ("top", &options.top),
        ("left", &options.left),
        ("bottom", &options.bottom),
        ("right", &options.right),
    ] {
        if present(value) {
            if let Some(value) = value {
                pixels(&node, key, value)?;
            }
        }
    }
    Ok((background, node))
}
