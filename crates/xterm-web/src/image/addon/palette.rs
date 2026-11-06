// Copyright (c) 2020, 2023 The xterm.js authors. MIT license.
//! Raw palette values belong to the addon; decoder calls own their conversion.
use super::super::js;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use xterm_addons::image::policy::{ImageControlAction, ImageControlEffect, ImageParameter};

const GRAPHICS_COLORS: f64 = 1.0;
const GRAPHICS_READ: f64 = 1.0;
const GRAPHICS_SET_DEFAULT: f64 = 2.0;
const GRAPHICS_SET: f64 = 3.0;
const MAX_SIXEL_PALETTE: f64 = 4096.0;

pub(super) struct Palette {
    current: RefCell<JsValue>,
    default: JsValue,
}
impl Palette {
    pub fn read(current: &JsValue, defaults: &JsValue) -> Result<Self, JsValue> {
        Ok(Self {
            current: RefCell::new(js::get(current, "sixelPaletteLimit")?),
            default: js::get(defaults, "sixelPaletteLimit")?,
        })
    }
    pub fn get(&self) -> JsValue {
        self.current.borrow().clone()
    }
    pub fn reset(&self) {
        self.current.replace(self.default.clone());
    }
    pub fn set(&self, value: f64) {
        self.current.replace(value.into());
    }
}

impl super::State {
    pub(super) fn palette_control(
        &self,
        params: &[ImageParameter],
    ) -> Option<Result<bool, JsValue>> {
        if params.len() < 2 || params[0] != ImageParameter::Number(GRAPHICS_COLORS) {
            return None;
        }
        let reset = match params[1] {
            ImageParameter::Number(GRAPHICS_READ) => false,
            ImageParameter::Number(GRAPHICS_SET_DEFAULT) => {
                self.palette.reset();
                true
            }
            ImageParameter::Number(GRAPHICS_SET) => {
                if let Some(ImageParameter::Number(value)) = params.get(2) {
                    if *value <= MAX_SIXEL_PALETTE {
                        self.palette.set(*value);
                        return Some(self.palette_report(false));
                    }
                }
                return None;
            }
            _ => return None,
        };
        Some(self.palette_report(reset))
    }
    fn palette_report(&self, reset: bool) -> Result<bool, JsValue> {
        // Capture before string conversion. Reentry may replace the live value,
        // but the in-flight reply still describes its evaluated operand.
        let value = self.palette.get();
        let data = reply(&value)?;
        let mut effects = vec![ImageControlEffect::Reply(crate::utf16::from_js_string(
            &data,
        ))];
        if reset {
            effects.push(ImageControlEffect::ResetHandlers);
        }
        self.execute(ImageControlAction {
            handled: true,
            effects,
        })
    }
}

#[wasm_bindgen(inline_js = "
export function imagePaletteInitial(value) {
  if (value > 4096) {
    throw new Error('DecoderOptions.paletteLimit must not exceed 4096');
  }
  // The decoder's WASM i32 argument converts again, then clamps in C++.
  return Math.min((+value) >>> 0, 4096);
}
export function imagePaletteReply(value) {
  return `\\x1b[?1;0;${value}S`;
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = imagePaletteInitial)]
    pub(super) fn initial(value: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = imagePaletteReply)]
    pub(super) fn reply(value: &JsValue) -> Result<js_sys::JsString, JsValue>;
}
