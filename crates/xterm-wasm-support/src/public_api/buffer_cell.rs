// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Cell snapshots retain undefined typed-array fields from numeric JS reads.
use crate::utf16::{from_js_string, to_js_string};
#[path = "buffer_cell_extended.rs"]
mod extended_getters;
#[path = "buffer_extended.rs"]
mod extended_values;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use xterm_core::{
    buffer::{CellData, ExtendedAttrs},
    public_api::line_numbers::{NumericCell, NumericCellError},
};

fn numeric_error(error: NumericCellError) -> JsValue {
    js_sys::TypeError::new(&error.to_string()).into()
}

#[wasm_bindgen(js_name = BufferCell)]
pub struct WebBufferCell {
    data: RefCell<NumericCell>,
    extended_js: RefCell<JsValue>,
}
impl WebBufferCell {
    pub(crate) fn coerce_word(value: &JsValue) -> Result<u32, JsValue> {
        extended_getters::cell_uint32(value)
    }
    pub(crate) fn new(data: CellData) -> Self {
        Self::numeric(data.into())
    }
    pub(crate) fn extended_value(value: Option<ExtendedAttrs>) -> JsValue {
        value
            .map(|value| extended_values::WebExtendedAttrs::new(value).into())
            .unwrap_or(JsValue::UNDEFINED)
    }
    pub(crate) fn numeric(mut data: NumericCell) -> Self {
        let extended = Self::extended_value(data.extended.take());
        // Browser extended getters use the JS object. Keep only an empty scalar
        // metadata cache here, so target reuse cannot retain an old line payload.
        data.extended = Some(ExtendedAttrs::default());
        Self {
            data: RefCell::new(data),
            extended_js: RefCell::new(extended),
        }
    }
}
#[wasm_bindgen(js_class = BufferCell)]
impl WebBufferCell {
    #[wasm_bindgen(js_name = getChars)]
    pub fn get_chars(&self) -> JsValue {
        let chars = self.data.borrow().chars();
        chars.map_or(JsValue::UNDEFINED, |value| to_js_string(&value).into())
    }
    #[wasm_bindgen(js_name = getCode)]
    pub fn get_code(&self) -> Result<f64, JsValue> {
        let code = self.data.borrow().code();
        code.map(|value| value.map_or(f64::NAN, f64::from))
            .map_err(numeric_error)
    }
    #[wasm_bindgen(js_name = getWidth)]
    pub fn get_width(&self) -> i32 {
        self.data.borrow().metadata().get_width()
    }
    #[wasm_bindgen(js_name = getFgColorMode)]
    pub fn get_fg_color_mode(&self) -> u32 {
        self.data.borrow().metadata().get_fg_color_mode()
    }
    #[wasm_bindgen(js_name = getBgColorMode)]
    pub fn get_bg_color_mode(&self) -> u32 {
        self.data.borrow().metadata().get_bg_color_mode()
    }
    #[wasm_bindgen(js_name = getFgColor)]
    pub fn get_fg_color(&self) -> i32 {
        self.data.borrow().metadata().get_fg_color()
    }
    #[wasm_bindgen(js_name = getBgColor)]
    pub fn get_bg_color(&self) -> i32 {
        self.data.borrow().metadata().get_bg_color()
    }
    #[wasm_bindgen(js_name = isBold)]
    pub fn is_bold(&self) -> u32 {
        self.data.borrow().metadata().is_bold()
    }
    #[wasm_bindgen(js_name = isItalic)]
    pub fn is_italic(&self) -> u32 {
        self.data.borrow().metadata().is_italic()
    }
    #[wasm_bindgen(js_name = isDim)]
    pub fn is_dim(&self) -> u32 {
        self.data.borrow().metadata().is_dim()
    }
    #[wasm_bindgen(js_name = isBlink)]
    pub fn is_blink(&self) -> u32 {
        self.data.borrow().metadata().is_blink()
    }
    #[wasm_bindgen(js_name = isInverse)]
    pub fn is_inverse(&self) -> u32 {
        self.data.borrow().metadata().is_inverse()
    }
    #[wasm_bindgen(js_name = isInvisible)]
    pub fn is_invisible(&self) -> u32 {
        self.data.borrow().metadata().is_invisible()
    }
    #[wasm_bindgen(js_name = isStrikethrough)]
    pub fn is_strikethrough(&self) -> i32 {
        i32::from_ne_bytes(
            self.data
                .borrow()
                .metadata()
                .is_strikethrough()
                .to_ne_bytes(),
        )
    }
    #[wasm_bindgen(js_name = isOverline)]
    pub fn is_overline(&self) -> u32 {
        self.data.borrow().metadata().is_overline()
    }
    #[wasm_bindgen(js_name = isFgRGB)]
    pub fn is_fg_rgb(&self) -> bool {
        self.data.borrow().metadata().is_fg_rgb()
    }
    #[wasm_bindgen(js_name = isFgPalette)]
    pub fn is_fg_palette(&self) -> bool {
        self.data.borrow().metadata().is_fg_palette()
    }
    #[wasm_bindgen(js_name = isFgDefault)]
    pub fn is_fg_default(&self) -> bool {
        self.data.borrow().metadata().is_fg_default()
    }
    #[wasm_bindgen(js_name = isBgRGB)]
    pub fn is_bg_rgb(&self) -> bool {
        self.data.borrow().metadata().is_bg_rgb()
    }
    #[wasm_bindgen(js_name = isBgPalette)]
    pub fn is_bg_palette(&self) -> bool {
        self.data.borrow().metadata().is_bg_palette()
    }
    #[wasm_bindgen(js_name = isBgDefault)]
    pub fn is_bg_default(&self) -> bool {
        self.data.borrow().metadata().is_bg_default()
    }
    #[wasm_bindgen(js_name = isAttributeDefault)]
    pub fn is_attribute_default(&self) -> bool {
        self.data.borrow().is_attribute_default()
    }
    #[wasm_bindgen(getter)]
    pub fn content(&self) -> Option<u32> {
        self.data.borrow().content
    }
    #[wasm_bindgen(setter)]
    pub fn set_content(&self, value: Option<u32>) {
        self.data.borrow_mut().content = value;
    }
    #[wasm_bindgen(getter)]
    pub fn fg(&self) -> Option<u32> {
        self.data.borrow().fg
    }
    #[wasm_bindgen(setter)]
    pub fn set_fg(&self, value: Option<u32>) {
        self.data.borrow_mut().fg = value;
    }
    #[wasm_bindgen(getter)]
    pub fn bg(&self) -> Option<u32> {
        self.data.borrow().bg
    }
    #[wasm_bindgen(setter)]
    pub fn set_bg(&self, value: Option<u32>) {
        self.data.borrow_mut().bg = value;
    }
    #[wasm_bindgen(getter, js_name = combinedData)]
    pub fn combined_data(&self) -> Option<js_sys::JsString> {
        let chars = self.data.borrow().combined_data.clone();
        chars.as_ref().map(|value| to_js_string(value))
    }
    #[wasm_bindgen(setter, js_name = combinedData)]
    pub fn set_combined_data(&self, value: Option<js_sys::JsString>) {
        let value = value.as_ref().map(from_js_string);
        self.data.borrow_mut().combined_data = value;
    }
    #[wasm_bindgen(getter)]
    pub fn extended(&self) -> JsValue {
        self.extended_js.borrow().clone()
    }
    #[wasm_bindgen(setter)]
    pub fn set_extended(&self, value: JsValue) {
        *self.extended_js.borrow_mut() = value;
    }
}
