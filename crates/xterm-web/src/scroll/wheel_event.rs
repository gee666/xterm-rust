// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! StandardWheelEvent normalization, retaining source field-read order and raw
//! arithmetic primitives until the consumer uses them.
use super::{
    environment::ScrollEnvironment,
    interop::{divide, get, method0, multiply, negate, nullish},
};
use wasm_bindgen::JsValue;
use web_sys::Event;

#[derive(Clone, Debug)]
pub struct StandardWheelEvent {
    pub browser_event: Option<Event>,
    pub delta_y: JsValue,
    pub delta_x: JsValue,
    pub target: JsValue,
}
impl StandardWheelEvent {
    pub fn new(
        event: Option<&Event>,
        delta_x: f64,
        delta_y: f64,
        environment: &ScrollEnvironment,
    ) -> Result<Self, JsValue> {
        let browser_event = event.filter(|event| !nullish(event.as_ref())).cloned();
        let mut result = Self {
            browser_event,
            delta_y: delta_y.into(),
            delta_x: delta_x.into(),
            target: JsValue::NULL,
        };
        if let Some(event) = event.filter(|event| {
            let raw: &JsValue = (*event).as_ref();
            raw.is_truthy()
        }) {
            let target = get(event.as_ref(), "target")?;
            result.target = if !nullish(&target) {
                target
            } else {
                let target = get(event.as_ref(), "targetNode")?;
                if !nullish(&target) {
                    target
                } else {
                    let target = get(event.as_ref(), "srcElement")?;
                    if nullish(&target) {
                        JsValue::NULL
                    } else {
                        target
                    }
                }
            };
        }
        // Source performs this navigator read even when the optional event is null.
        let factor_dpr = environment.factor_legacy_dpr()?;
        let Some(event) = event.filter(|event| {
            let raw: &JsValue = (*event).as_ref();
            raw.is_truthy()
        }) else {
            return Ok(result);
        };
        let raw = event.as_ref();
        let view = get(raw, "view")?;
        let dpr = if nullish(&view) {
            JsValue::UNDEFINED
        } else {
            get(&view, "devicePixelRatio")?
        };
        let dpr = if nullish(&dpr) { 1.0.into() } else { dpr };
        if !get(raw, "wheelDeltaY")?.is_undefined() {
            result.delta_y = legacy_delta(&get(raw, "wheelDeltaY")?, &dpr, factor_dpr)?;
        } else if axis_matches(raw, "VERTICAL_AXIS")? {
            result.delta_y = negative_ratio(&get(raw, "detail")?, 3.0)?;
        } else if get(raw, "type")? == JsValue::from_str("wheel") {
            result.delta_y = modern_delta(raw, "deltaY", environment)?;
        }
        if !get(raw, "wheelDeltaX")?.is_undefined() {
            if environment.platform.is_safari && environment.platform.is_windows {
                result.delta_x = negate(&divide(&get(raw, "wheelDeltaX")?, &120.0.into())?)?;
            } else {
                result.delta_x = legacy_delta(&get(raw, "wheelDeltaX")?, &dpr, factor_dpr)?;
            }
        } else if axis_matches(raw, "HORIZONTAL_AXIS")? {
            result.delta_x = negative_ratio(&get(raw, "detail")?, 3.0)?;
        } else if get(raw, "type")? == JsValue::from_str("wheel") {
            result.delta_x = modern_delta(raw, "deltaX", environment)?;
        }
        if result.delta_y == JsValue::from_f64(0.0)
            && result.delta_x == JsValue::from_f64(0.0)
            && get(raw, "wheelDelta")?.is_truthy()
        {
            result.delta_y = legacy_delta(&get(raw, "wheelDelta")?, &dpr, factor_dpr)?;
        }
        Ok(result)
    }
    pub fn prevent_default(&self) -> Result<(), JsValue> {
        if let Some(event) = self.browser_event.as_ref() {
            method0(event.as_ref(), "preventDefault")?;
        }
        Ok(())
    }
    pub fn stop_propagation(&self) -> Result<(), JsValue> {
        if let Some(event) = self.browser_event.as_ref() {
            method0(event.as_ref(), "stopPropagation")?;
        }
        Ok(())
    }
}
fn axis_matches(event: &JsValue, name: &str) -> Result<bool, JsValue> {
    if get(event, name)?.is_undefined() {
        return Ok(false);
    }
    Ok(get(event, "axis")? == get(event, name)?)
}
fn legacy_delta(value: &JsValue, dpr: &JsValue, factor: bool) -> Result<JsValue, JsValue> {
    let denominator = if factor {
        multiply(&120.0.into(), dpr)?
    } else {
        120.0.into()
    };
    divide(value, &denominator)
}
fn negative_ratio(value: &JsValue, denominator: f64) -> Result<JsValue, JsValue> {
    divide(&negate(value)?, &denominator.into())
}
fn modern_delta(
    event: &JsValue,
    axis: &str,
    environment: &ScrollEnvironment,
) -> Result<JsValue, JsValue> {
    if get(event, "deltaMode")? == get(event, "DOM_DELTA_LINE")? {
        if environment.platform.is_firefox && !environment.platform.is_mac {
            negative_ratio(&get(event, axis)?, 3.0)
        } else {
            negate(&get(event, axis)?)
        }
    } else {
        negative_ratio(&get(event, axis)?, 40.0)
    }
}
