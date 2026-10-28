// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Extended getters read the shared JS attribute object lazily. Reading all
//! fields up front would change accessor side effects and error precedence.
use super::WebBufferCell;
use js_sys::{Function, Reflect};
use wasm_bindgen::{prelude::*, JsCast};
use xterm_core::buffer::constants::{attributes, bg_flags, fg_flags};

#[wasm_bindgen(inline_js = "export function cellUint32(value) { return value >>> 0; }")]
extern "C" {
    #[wasm_bindgen(catch, js_name = cellUint32)]
    pub(super) fn cell_uint32(value: &JsValue) -> Result<u32, JsValue>;
}
impl WebBufferCell {
    fn fg_bits(&self) -> u32 {
        self.data.borrow().fg.unwrap_or(0)
    }
    fn bg_bits(&self) -> u32 {
        self.data.borrow().bg.unwrap_or(0)
    }
    fn has_extended(&self) -> bool {
        self.bg_bits() & bg_flags::HAS_EXTENDED != 0
    }
    fn extended_property(&self, name: &str) -> Result<JsValue, JsValue> {
        let value = self.extended_js.borrow().clone();
        Reflect::get(&value, &name.into())
    }
    fn extended_word(&self, name: &str) -> Result<u32, JsValue> {
        cell_uint32(&self.extended_property(name)?)
    }
    fn custom_underline_color(&self) -> Result<bool, JsValue> {
        Ok(self.has_extended() && self.extended_word("underlineColor")? != u32::MAX)
    }
}
fn other_method(other: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    let value = Reflect::get(other, &name.into())?;
    let function = value.dyn_into::<Function>().map_err(|_| {
        JsValue::from(js_sys::TypeError::new(&format!(
            "other.{name} is not a function"
        )))
    })?;
    function.call0(other)
}
#[wasm_bindgen]
impl WebBufferCell {
    #[wasm_bindgen(js_name = isUnderline)]
    pub fn is_underline(&self) -> Result<u32, JsValue> {
        if self.has_extended()
            && self.extended_property("underlineStyle")? != JsValue::from_f64(0.0)
        {
            return Ok(1);
        }
        Ok(self.fg_bits() & fg_flags::UNDERLINE)
    }
    #[wasm_bindgen(js_name = getUnderlineStyle)]
    pub fn get_underline_style(&self) -> Result<JsValue, JsValue> {
        if self.fg_bits() & fg_flags::UNDERLINE == 0 {
            return Ok(0.into());
        }
        if self.has_extended() {
            self.extended_property("underlineStyle")
        } else {
            Ok(1.into())
        }
    }
    #[wasm_bindgen(js_name = getUnderlineColor)]
    pub fn get_underline_color(&self) -> Result<i32, JsValue> {
        if self.custom_underline_color()? {
            match self.extended_word("underlineColor")? & attributes::CM_MASK {
                attributes::CM_P16 | attributes::CM_P256 => {
                    return Ok(
                        (self.extended_word("underlineColor")? & attributes::PCOLOR_MASK) as i32,
                    )
                }
                attributes::CM_RGB => {
                    return Ok(
                        (self.extended_word("underlineColor")? & attributes::RGB_MASK) as i32,
                    )
                }
                _ => {}
            }
        }
        Ok(self.get_fg_color())
    }
    #[wasm_bindgen(js_name = getUnderlineColorMode)]
    pub fn get_underline_color_mode(&self) -> Result<u32, JsValue> {
        if self.custom_underline_color()? {
            Ok(self.extended_word("underlineColor")? & attributes::CM_MASK)
        } else {
            Ok(self.get_fg_color_mode())
        }
    }
    #[wasm_bindgen(js_name = isUnderlineColorRGB)]
    pub fn is_underline_color_rgb(&self) -> Result<bool, JsValue> {
        if self.custom_underline_color()? {
            Ok(self.extended_word("underlineColor")? & attributes::CM_MASK == attributes::CM_RGB)
        } else {
            Ok(self.is_fg_rgb())
        }
    }
    #[wasm_bindgen(js_name = isUnderlineColorPalette)]
    pub fn is_underline_color_palette(&self) -> Result<bool, JsValue> {
        if self.custom_underline_color()? {
            Ok(
                self.extended_word("underlineColor")? & attributes::CM_MASK == attributes::CM_P16
                    || self.extended_word("underlineColor")? & attributes::CM_MASK
                        == attributes::CM_P256,
            )
        } else {
            Ok(self.is_fg_palette())
        }
    }
    #[wasm_bindgen(js_name = isUnderlineColorDefault)]
    pub fn is_underline_color_default(&self) -> Result<bool, JsValue> {
        if self.custom_underline_color()? {
            Ok(self.extended_word("underlineColor")? & attributes::CM_MASK == 0)
        } else {
            Ok(self.is_fg_default())
        }
    }
    #[wasm_bindgen(js_name = attributesEquals)]
    pub fn attributes_equals(&self, other: JsValue) -> Result<bool, JsValue> {
        if JsValue::from(self.get_fg_color_mode()) != other_method(&other, "getFgColorMode")?
            || JsValue::from(self.get_fg_color()) != other_method(&other, "getFgColor")?
            || JsValue::from(self.get_bg_color_mode()) != other_method(&other, "getBgColorMode")?
            || JsValue::from(self.get_bg_color()) != other_method(&other, "getBgColor")?
            || JsValue::from(self.is_inverse()) != other_method(&other, "isInverse")?
            || JsValue::from(self.is_bold()) != other_method(&other, "isBold")?
        {
            return Ok(false);
        }
        if JsValue::from(self.is_underline()?) != other_method(&other, "isUnderline")? {
            return Ok(false);
        }
        if self.is_underline()? != 0 {
            if self.get_underline_style()? != other_method(&other, "getUnderlineStyle")? {
                return Ok(false);
            }
            let own_default = self.is_underline_color_default()?;
            let other_default = other_method(&other, "isUnderlineColorDefault")?;
            if !(own_default && other_default.is_truthy()) {
                if JsValue::from_bool(own_default) != other_default {
                    return Ok(false);
                }
                if JsValue::from(self.get_underline_color()?)
                    != other_method(&other, "getUnderlineColor")?
                {
                    return Ok(false);
                }
                if JsValue::from(self.get_underline_color_mode()?)
                    != other_method(&other, "getUnderlineColorMode")?
                {
                    return Ok(false);
                }
            }
        }
        Ok(
            JsValue::from(self.is_overline()) == other_method(&other, "isOverline")?
                && JsValue::from(self.is_blink()) == other_method(&other, "isBlink")?
                && JsValue::from(self.is_invisible()) == other_method(&other, "isInvisible")?
                && JsValue::from(self.is_italic()) == other_method(&other, "isItalic")?
                && JsValue::from(self.is_dim()) == other_method(&other, "isDim")?
                && JsValue::from(self.is_strikethrough())
                    == other_method(&other, "isStrikethrough")?,
        )
    }
}
