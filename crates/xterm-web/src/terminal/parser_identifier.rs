// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Preserve JS accessor order, UTF-16 charCodeAt and operand coercions. A field
//! snapshot changes the source's validation precedence and CSI policy decision.
use wasm_bindgen::prelude::*;

const PREFIX_MIN: u32 = 0x3c;
const PREFIX_MAX: u32 = 0x3f;
const INTERMEDIATE_MIN: u32 = 0x20;
const INTERMEDIATE_MAX: u32 = 0x2f;
const CSI_DCS_FINAL_MIN: u32 = 0x40;
const ESC_APC_FINAL_MIN: u32 = 0x30;
const FINAL_MAX: u32 = 0x7e;
const MAX_PREFIX_LENGTH: u32 = 1;
const MAX_INTERMEDIATE_LENGTH: u32 = 2;

#[wasm_bindgen(module = "/src/terminal/parser_identifier_bridge.js")]
extern "C" {
    #[wasm_bindgen(catch,js_name=identifierProperty)]
    fn property(value: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch,js_name=identifierCodeAt)]
    fn code_at(value: &JsValue, index: u32) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch,js_name=identifierLess)]
    fn less(left: &JsValue, right: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch,js_name=identifierGreater)]
    fn greater(left: &JsValue, right: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch,js_name=identifierWord)]
    fn word(value: &JsValue) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch,js_name=identifierClearPrefix)]
    fn clear_prefix(value: &JsValue) -> Result<(), JsValue>;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Csi,
    Dcs,
    Esc,
    Apc,
}
pub(super) struct Identifier {
    pub packed: u32,
    pub source_guarded: bool,
}
fn out_of_range(value: &JsValue, min: u32, max: u32) -> Result<bool, JsValue> {
    Ok(less(value, &min.into())? || greater(value, &max.into())?)
}
fn shift_or(left: &JsValue, right: &JsValue) -> Result<JsValue, JsValue> {
    let shifted = word(left)?.wrapping_shl(8);
    let packed = shifted | word(right)?;
    Ok(i32::from_ne_bytes(packed.to_ne_bytes()).into())
}

pub(super) fn read_identifier(id: &JsValue, kind: Kind) -> Result<Identifier, JsValue> {
    if kind == Kind::Apc {
        clear_prefix(id)?;
    }
    // InputHandler checks these fields before EscapeSequenceParser._identifier.
    // A getter can change the final between these stages, so carry the decision.
    let source_guarded = kind == Kind::Csi
        && property(id, "final")? == JsValue::from_str("t")
        && !property(id, "prefix")?.is_truthy()
        && !property(id, "intermediates")?.is_truthy();
    let final_min = if kind == Kind::Csi || kind == Kind::Dcs {
        CSI_DCS_FINAL_MIN
    } else {
        ESC_APC_FINAL_MIN
    };
    let packed = pack_identifier(id, final_min)?;
    Ok(Identifier {
        packed,
        source_guarded,
    })
}

fn pack_identifier(id: &JsValue, final_min: u32) -> Result<u32, JsValue> {
    let mut result = JsValue::from_f64(0.0);
    if property(id, "prefix")?.is_truthy() {
        if greater(
            &property(&property(id, "prefix")?, "length")?,
            &MAX_PREFIX_LENGTH.into(),
        )? {
            return Err(js_sys::Error::new("only one byte as prefix supported").into());
        }
        result = code_at(&property(id, "prefix")?, 0)?;
        if out_of_range(&result, PREFIX_MIN, PREFIX_MAX)? {
            return Err(js_sys::Error::new("prefix must be in range 0x3c .. 0x3f").into());
        }
    }
    if property(id, "intermediates")?.is_truthy() {
        if greater(
            &property(&property(id, "intermediates")?, "length")?,
            &MAX_INTERMEDIATE_LENGTH.into(),
        )? {
            return Err(js_sys::Error::new("only two bytes as intermediates are supported").into());
        }
        let mut index = 0u32;
        while less(
            &index.into(),
            &property(&property(id, "intermediates")?, "length")?,
        )? {
            let intermediate = code_at(&property(id, "intermediates")?, index)?;
            if out_of_range(&intermediate, INTERMEDIATE_MIN, INTERMEDIATE_MAX)? {
                return Err(
                    js_sys::Error::new("intermediate must be in range 0x20 .. 0x2f").into(),
                );
            }
            result = shift_or(&result, &intermediate)?;
            // Every real JS string has fewer than u32::MAX UTF-16 units. A
            // synthetic length accessor outside that domain is an explicit error.
            index = index.checked_add(1).ok_or_else(|| {
                js_sys::RangeError::new("identifier index exceeds JavaScript string capacity")
            })?;
        }
    }
    if property(&property(id, "final")?, "length")? != JsValue::from_f64(1.0) {
        return Err(js_sys::Error::new("final must be a single byte").into());
    }
    let final_code = code_at(&property(id, "final")?, 0)?;
    if out_of_range(&final_code, final_min, FINAL_MAX)? {
        return Err(js_sys::Error::new(&format!(
            "final must be in range {final_min} .. {FINAL_MAX}"
        ))
        .into());
    }
    word(&shift_or(&result, &final_code)?)
}
