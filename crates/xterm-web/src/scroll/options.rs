// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Browser option resolution. The source copies non-undefined values without
//! validation; keep them raw until each consuming arithmetic/string expression.
use super::ScrollbarVisibility;
use crate::terminal::options::from_js;
use wasm_bindgen::prelude::*;
use xterm_core::options::OptionValue as V;

use super::interop::{get as scrollOptionGet, set as scrollOptionSet};

#[derive(Clone, Debug)]
pub struct ResolvedOptions {
    pub lazy_render: V,
    pub class_name: V,
    pub use_shadows: V,
    pub handle_mouse_wheel: V,
    pub flip_axes: V,
    pub scroll_y_to_x: V,
    pub consume_mouse_wheel_if_scrollbar_is_needed: V,
    pub always_consume_mouse_wheel: V,
    pub mouse_wheel_scroll_sensitivity: V,
    pub fast_scroll_sensitivity: V,
    pub scroll_predominant_axis: V,
    pub mouse_wheel_smooth_scroll: V,
    pub listen_on_dom_node: JsValue,
    pub horizontal: ScrollbarVisibility,
    pub horizontal_scrollbar_size: V,
    pub horizontal_slider_size: V,
    pub horizontal_has_arrows: V,
    pub vertical: ScrollbarVisibility,
    pub vertical_scrollbar_size: V,
    pub vertical_slider_size: V,
    pub vertical_has_arrows: V,
    pub scroll_by_page: V,
}

/// Preserve the source's typeof check followed by a second read. The second
/// getter can return undefined, mutate another property or throw.
pub fn read_change(options: &JsValue, key: &str) -> Result<Option<V>, JsValue> {
    if scrollOptionGet(options, key)?.is_undefined() {
        return Ok(None);
    }
    from_js(&scrollOptionGet(options, key)?).map(Some)
}
pub fn number(value: &V) -> Result<f64, JsValue> {
    value
        .to_number()
        .map_err(|error| crate::terminal::options::error_to_js(&error))
}

/// Arrow geometry's value + 'px' uses default-hint concatenation. This is not
/// FastDomNode::numberAsPixels, which leaves all non-number values unchanged.
pub fn concat_pixels(value: &V) -> Result<js_sys::JsString, JsValue> {
    let mut text = value
        .concat_utf16()
        .map_err(|error| crate::terminal::options::error_to_js(&error))?;
    text.extend([u16::from(b'p'), u16::from(b'x')]);
    Ok(crate::utf16::to_js_string(&text))
}

fn pick(options: &JsValue, key: &str, default: V) -> Result<V, JsValue> {
    Ok(read_change(options, key)?.unwrap_or(default))
}
fn node_option(options: &JsValue) -> Result<JsValue, JsValue> {
    if scrollOptionGet(options, "listenOnDomNode")?.is_undefined() {
        Ok(JsValue::NULL)
    } else {
        scrollOptionGet(options, "listenOnDomNode")
    }
}

pub fn normalize_input(options: &JsValue, owns_scrollable: bool) -> Result<JsValue, JsValue> {
    let options = if options.is_null() || options.is_undefined() {
        js_sys::Object::new().into()
    } else {
        options.clone()
    };
    if owns_scrollable {
        // Source mutates the caller's options. A setter/frozen object/primitive
        // failure must propagate before any subsequent option getter is read.
        scrollOptionSet(&options, "mouseWheelSmoothScroll", &JsValue::FALSE)?;
    }
    Ok(options)
}

impl ResolvedOptions {
    /// Call after normalize_input, because optional Scrollable ownership changes
    /// the input object's mouseWheelSmoothScroll property before resolution.
    pub fn resolve(options: &JsValue, is_mac: bool) -> Result<Self, JsValue> {
        let mut result = Self {
            lazy_render: pick(options, "lazyRender", false.into())?,
            class_name: pick(options, "className", "".into())?,
            use_shadows: pick(options, "useShadows", true.into())?,
            handle_mouse_wheel: pick(options, "handleMouseWheel", true.into())?,
            flip_axes: pick(options, "flipAxes", false.into())?,
            consume_mouse_wheel_if_scrollbar_is_needed: pick(
                options,
                "consumeMouseWheelIfScrollbarIsNeeded",
                false.into(),
            )?,
            always_consume_mouse_wheel: pick(options, "alwaysConsumeMouseWheel", false.into())?,
            scroll_y_to_x: pick(options, "scrollYToX", false.into())?,
            mouse_wheel_scroll_sensitivity: pick(
                options,
                "mouseWheelScrollSensitivity",
                1.0.into(),
            )?,
            fast_scroll_sensitivity: pick(options, "fastScrollSensitivity", 5.0.into())?,
            scroll_predominant_axis: pick(options, "scrollPredominantAxis", true.into())?,
            mouse_wheel_smooth_scroll: pick(options, "mouseWheelSmoothScroll", true.into())?,
            listen_on_dom_node: node_option(options)?,
            horizontal: ScrollbarVisibility::from_value(pick(options, "horizontal", 1.0.into())?),
            horizontal_scrollbar_size: pick(options, "horizontalScrollbarSize", 10.0.into())?,
            horizontal_slider_size: pick(options, "horizontalSliderSize", 0.0.into())?,
            horizontal_has_arrows: pick(options, "horizontalHasArrows", false.into())?,
            vertical: ScrollbarVisibility::from_value(pick(options, "vertical", 1.0.into())?),
            vertical_scrollbar_size: pick(options, "verticalScrollbarSize", 10.0.into())?,
            vertical_has_arrows: pick(options, "verticalHasArrows", false.into())?,
            vertical_slider_size: pick(options, "verticalSliderSize", 0.0.into())?,
            scroll_by_page: pick(options, "scrollByPage", false.into())?,
        };
        result.horizontal_slider_size = pick(
            options,
            "horizontalSliderSize",
            result.horizontal_scrollbar_size.clone(),
        )?;
        result.vertical_slider_size = pick(
            options,
            "verticalSliderSize",
            result.vertical_scrollbar_size.clone(),
        )?;
        if is_mac {
            result.class_name = result
                .class_name
                .js_add(&V::from(" xterm-mac"))
                .map_err(|error| crate::js_error::to_js_error(&error))?;
        }
        Ok(result)
    }
}
