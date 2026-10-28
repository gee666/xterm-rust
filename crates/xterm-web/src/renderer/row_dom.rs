// Copyright (c) 2018, 2023 The xterm.js authors. All rights reserved. MIT license.
use super::row_types::RowSpan;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, HtmlElement};

/// Preserve source CSS assignment and DOM attribute order, not merely computed
/// color equality: upstream row assertions compare serialized span HTML.
pub fn create_spans(document: &Document, spans: &[RowSpan]) -> Result<Vec<HtmlElement>, JsValue> {
    spans
        .iter()
        .map(|span| {
            let element: HtmlElement = document.create_element("span")?.dyn_into()?;
            js_sys::Reflect::set(
                element.as_ref(),
                &JsValue::from_str("textContent"),
                &crate::utf16::to_js_string(&span.text),
            )?;
            for (property, value) in &span.styles {
                if property == "letter-spacing" {
                    continue;
                }
                if property == "background-color" || property == "color" {
                    let existing = element.get_attribute("style").unwrap_or_default();
                    element.set_attribute("style", &format!("{existing}{property}:{value};"))?;
                } else {
                    element.style().set_property(property, value)?;
                }
            }
            if !span.classes.is_empty() {
                element.set_class_name(&span.classes.join(" "));
            }
            if let Some((_, spacing)) = span
                .styles
                .iter()
                .find(|(property, _)| property == "letter-spacing")
            {
                element.style().set_property("letter-spacing", spacing)?;
            }
            Ok(element)
        })
        .collect()
}
