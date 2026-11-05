// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Raw browser state values and live update-object entry points. Numeric and
//! raw callers share state and animation algorithms, not synchronized caches.
use super::{
    interop,
    scrollable::{ScrollError, Scrollable},
    state::{DimensionUpdate, Position, PositionUpdate, ScrollState},
    value::ScrollValue,
};
use crate::links::debounce::FrameScheduler;
use wasm_bindgen::prelude::*;

// Only language operators cross this boundary. Clamping, updates and events
// remain in the shared Rust algorithm, including their observable ordering.
#[wasm_bindgen(inline_js = "
export function scrollRawInt32(value) { return value | 0; }
export function scrollRawAdd(a, b) { return a + b; }
export function scrollRawLess(a, b) { return a < b; }
export function scrollRawGreater(a, b) { return a > b; }
export function scrollRawAbs(value) { return Math.abs(value); }
export function scrollRawPower(value, exponent) { return Math.pow(value, exponent); }
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = scrollRawInt32)]
    fn int32(value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = scrollRawAdd)]
    fn add(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = scrollRawLess)]
    fn less(a: &JsValue, b: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = scrollRawGreater)]
    fn greater(a: &JsValue, b: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = scrollRawAbs)]
    fn absolute(value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = scrollRawPower)]
    fn power(value: &JsValue, exponent: &JsValue) -> Result<JsValue, JsValue>;
}
impl ScrollValue for JsValue {
    type Error = JsValue;
    fn number(value: f64) -> Self {
        value.into()
    }
    fn is_undefined(&self) -> bool {
        JsValue::is_undefined(self)
    }
    fn int32(&self) -> Result<Self, JsValue> {
        int32(self)
    }
    fn add(&self, other: &Self) -> Result<Self, JsValue> {
        add(self, other)
    }
    fn subtract(&self, other: &Self) -> Result<Self, JsValue> {
        interop::subtract(self, other)
    }
    fn less(&self, other: &Self) -> Result<bool, JsValue> {
        less(self, other)
    }
    fn greater(&self, other: &Self) -> Result<bool, JsValue> {
        greater(self, other)
    }
    fn multiply(&self, other: &Self) -> Result<Self, JsValue> {
        interop::multiply(self, other)
    }
    fn divide(&self, other: &Self) -> Result<Self, JsValue> {
        interop::divide(self, other)
    }
    fn absolute(&self) -> Result<Self, JsValue> {
        absolute(self)
    }
    fn power(&self, exponent: &Self) -> Result<Self, JsValue> {
        power(self, exponent)
    }
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

/// Read once for the undefined test and again for the selected expression.
/// Inherited properties, changed getter results and thrown values are observable.
fn defined(update: &JsValue, key: &str) -> Result<Option<JsValue>, JsValue> {
    if interop::get(update, key)?.is_undefined() {
        Ok(None)
    } else {
        interop::get(update, key).map(Some)
    }
}

fn dimensions(update: &JsValue) -> Result<DimensionUpdate<JsValue>, JsValue> {
    Ok(DimensionUpdate {
        width: defined(update, "width")?,
        scroll_width: defined(update, "scrollWidth")?,
        height: defined(update, "height")?,
        scroll_height: defined(update, "scrollHeight")?,
    })
}
fn position(update: &JsValue) -> Result<PositionUpdate<JsValue>, JsValue> {
    Ok(PositionUpdate {
        left: defined(update, "scrollLeft")?,
        top: defined(update, "scrollTop")?,
    })
}
impl ScrollState<JsValue> {
    pub fn try_with_raw_dimensions(&self, update: &JsValue, raw: bool) -> Result<Self, JsValue> {
        self.try_with_dimensions(dimensions(update)?, raw)
    }
    pub fn try_with_raw_position(&self, update: &JsValue) -> Result<Self, JsValue> {
        self.try_with_position(position(update)?)
    }
}
impl<S: FrameScheduler> Scrollable<S, JsValue> {
    pub fn try_validate_raw_scroll_position(
        &self,
        update: &JsValue,
    ) -> Result<Position<JsValue>, JsValue> {
        self.try_validate_scroll_position_resolving(|| position(update))
    }
    pub fn set_raw_scroll_dimensions(
        &self,
        update: &JsValue,
        raw: bool,
    ) -> Result<(), ScrollError<S::Error, JsValue>> {
        self.set_scroll_dimensions_resolving(|| dimensions(update), raw)
    }
    pub fn set_raw_scroll_position_now(
        &self,
        update: &JsValue,
    ) -> Result<(), ScrollError<S::Error, JsValue>> {
        self.set_scroll_position_now_resolving(|| position(update))
    }
    pub fn set_raw_scroll_position_smooth(
        &self,
        update: &JsValue,
        reuse: bool,
    ) -> Result<(), ScrollError<S::Error, JsValue>> {
        self.set_scroll_position_smooth_resolving(
            || defined(update, "scrollLeft"),
            || defined(update, "scrollTop"),
            reuse,
        )
    }
}
