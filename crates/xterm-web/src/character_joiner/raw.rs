// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Raw browser ranges retain the first handler's array and all inner identities.
//! The native service uses the same traversal with infallible typed storage.
mod merge;
mod ops;
use super::{CharacterJoinerService, LineRef, RangeStorage};
use js_sys::Array;
use ops::*;
use wasm_bindgen::JsValue;

pub type BrowserJoinerService = CharacterJoinerService<RawRanges>;
pub use ops::report as report_error;
pub struct RawRanges(pub JsValue);
impl BrowserJoinerService {
    /// Receiver callbacks may mutate the record's id, including through getters.
    pub fn deregister_js(&self, id: f64) -> Result<bool, JsValue> {
        let mut index = 0;
        loop {
            let entry = self
                .handlers
                .borrow()
                .get(index)
                .map(|entry| (entry.id, entry.record.clone()));
            let Some((typed_id, record)) = entry else {
                return Ok(false);
            };
            let matches = if let Some(record) = record {
                get(&record, &"id".into())?
                    .as_f64()
                    .is_some_and(|value| value == id)
            } else {
                typed_id == id
            };
            if matches {
                let mut handlers = self.handlers.borrow_mut();
                if index < handlers.len() {
                    handlers.remove(index);
                }
                return Ok(true);
            }
            index += 1;
        }
    }
    /// Upstream calls a handler as a member of its joiner record, not as a free
    /// function. Keep that receiver and its handler property alive across calls.
    pub fn register_js(&self, handler: js_sys::Function) -> f64 {
        let record = record(self.next_id.get(), &handler);
        let receiver = record.clone();
        let id = self.register(std::rc::Rc::new(move |text| {
            call(&receiver, &crate::utf16::to_js_string(text)).map(RawRanges)
        }));
        // register does not call user code, so this is still the just-added entry.
        if let Some(entry) = self.handlers.borrow_mut().last_mut() {
            entry.record = Some(record);
        }
        id
    }
}
impl RawRanges {
    /// The existing typed row factory consumes cell offsets. Conversion has
    /// already mutated raw ranges; never coerce offsets a second time here.
    pub fn render_ranges(&self) -> Result<Vec<[usize; 2]>, JsValue> {
        let mut result = Vec::new();
        let mut index = 0.0;
        while less(&index.into(), &length(&self.0)?)? {
            let range = at(&self.0, index)?;
            let start = at(&range, 0.0)?;
            let Some(start) = start.as_f64().filter(|v| {
                v.is_finite() && *v >= 0.0 && v.fract() == 0.0 && *v <= u32::MAX as f64
            }) else {
                // Strict equality in the source row factory cannot match this
                // head range, so later ranges are not reached either.
                break;
            };
            let end = at(&range, 1.0)?
                .as_f64()
                .filter(|v| v.is_finite() && *v >= 0.0 && v.fract() == 0.0 && *v <= u32::MAX as f64)
                .ok_or_else(|| js_sys::RangeError::new("non-integral joined cell endpoint"))?;
            result.push([start as usize, end as usize]);
            index += 1.0;
        }
        Ok(result)
    }
}
impl RangeStorage for RawRanges {
    type HandlerError = JsValue;
    type ConversionError = JsValue;
    fn empty() -> Self {
        Self(Array::new().into())
    }
    fn missing_handler(text: &[u16]) -> Result<Self, JsValue> {
        call(&JsValue::UNDEFINED, &crate::utf16::to_js_string(text)).map(Self)
    }
    fn merge(&mut self, other: Self) -> Result<(), JsValue> {
        let mut index = 0.0;
        while less(&index.into(), &length(&other.0)?)? {
            merge::merge(&self.0, &at(&other.0, index)?)?;
            index += 1.0;
        }
        Ok(())
    }
    fn append(&mut self, other: Self) -> Result<(), JsValue> {
        let mut index = 0.0;
        while less(&index.into(), &length(&other.0)?)? {
            push(&self.0, &at(&other.0, index)?)?;
            index += 1.0;
        }
        Ok(())
    }
    fn convert(&mut self, line: &LineRef, start: usize) -> Result<(), JsValue> {
        let mut index = 0.0;
        let mut started = false;
        let mut string_index = 0;
        let mut range = at(&self.0, index)?;
        if !range.is_truthy() {
            return Ok(());
        }
        let trimmed = line.borrow().get_trimmed_length();
        for x in start..trimmed {
            // Release the line before any host getter or setter can reenter.
            let (width, len) = {
                let line = line.borrow();
                (
                    line.get_width(x),
                    line.get_string(x).map_or(1, |s| s.len().max(1)),
                )
            };
            if width == Some(0) {
                continue;
            }
            if !started && le(&at(&range, 0.0)?, &JsValue::from_f64(string_index as f64))? {
                put(&range, 0.0, &JsValue::from_f64(x as f64))?;
                started = true;
            }
            if le(&at(&range, 1.0)?, &JsValue::from_f64(string_index as f64))? {
                put(&range, 1.0, &JsValue::from_f64(x as f64))?;
                index += 1.0;
                range = at(&self.0, index)?;
                if !range.is_truthy() {
                    break;
                }
                started = le(&at(&range, 0.0)?, &JsValue::from_f64(string_index as f64))?;
                if started {
                    put(&range, 0.0, &JsValue::from_f64(x as f64))?;
                }
            }
            string_index += len;
        }
        if range.is_truthy() {
            put(&range, 1.0, &JsValue::from_f64(trimmed as f64))?;
        }
        Ok(())
    }
}
