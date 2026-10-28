// Copyright (c) 2024 The xterm.js authors. All rights reserved. Licensed under MIT.
use super::{error, Viewport};
use crate::{theme::ColorRef, utf16::to_js_string};
use wasm_bindgen::JsValue;

fn background(viewport: &Viewport) -> ColorRef {
    viewport
        .host
        .services
        .theme
        .colors()
        .borrow()
        .background
        .clone()
}
fn set(target: &JsValue, key: &str, value: JsValue) -> Result<(), JsValue> {
    if js_sys::Reflect::set(target, &key.into(), &value)? {
        Ok(())
    } else {
        Err(error("Cannot set viewport DOM property"))
    }
}
pub(super) fn apply_background(viewport: &Viewport) -> Result<(), JsValue> {
    let root_style = viewport.root.style();
    set(
        root_style.as_ref(),
        "backgroundColor",
        to_js_string(&background(viewport).css_units()).into(),
    )?;
    let node = viewport.host.element()?.get_dom_node()?;
    // Read the current color again after the first overridable DOM setter.
    set(
        node.style().as_ref(),
        "backgroundColor",
        to_js_string(&background(viewport).css_units()).into(),
    )
}
fn append(output: &mut Vec<u16>, text: &str) {
    output.extend(text.encode_utf16());
}
pub(super) fn apply_rules(viewport: &Viewport) -> Result<(), JsValue> {
    let mut output = Vec::new();
    append(
        &mut output,
        ".xterm .xterm-scrollable-element > .xterm-scrollbar > .xterm-slider {\n  background: ",
    );
    let color = viewport
        .host
        .services
        .theme
        .colors()
        .borrow()
        .scrollbar_slider_background
        .clone();
    output.extend(color.css_units());
    append(&mut output, ";\n}\n.xterm .xterm-scrollable-element > .xterm-scrollbar > .xterm-slider:hover {\n  background: ");
    let color = viewport
        .host
        .services
        .theme
        .colors()
        .borrow()
        .scrollbar_slider_hover_background
        .clone();
    output.extend(color.css_units());
    append(&mut output, ";\n}\n.xterm .xterm-scrollable-element > .xterm-scrollbar > .xterm-slider.xterm-active {\n  background: ");
    let color = viewport
        .host
        .services
        .theme
        .colors()
        .borrow()
        .scrollbar_slider_active_background
        .clone();
    output.extend(color.css_units());
    append(&mut output, ";\n}");
    let style = viewport
        .style
        .borrow()
        .clone()
        .ok_or_else(|| error("Viewport style is not initialized"))?;
    set(style.as_ref(), "textContent", to_js_string(&output).into())
}
