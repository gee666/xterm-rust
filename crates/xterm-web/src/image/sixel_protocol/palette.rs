// Copyright (c) 2020, 2023 The xterm.js authors. MIT license.
//! Argument evaluation around decoder init/release, outside native borrows.
use super::{RawPalette, State};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

impl State {
    pub(super) fn palette_limit(&self) -> Result<f64, JsValue> {
        match &self.palette {
            Some(palette) => {
                let value = (palette.current)();
                // Decoder.init defaults undefined to its constructor option.
                let value = if value.is_undefined() {
                    &palette.initial
                } else {
                    &value
                };
                super::super::addon::options::number(value)
            }
            None => Ok(self.options.get().sixel_palette_limit),
        }
    }
    fn complete_release(&self, palette: &RawPalette) -> Result<(), JsValue> {
        let limit = super::super::addon::options::number(&palette.initial)?;
        self.handler.borrow_mut().finish_release(limit);
        Ok(())
    }
    pub(super) fn release(&self) -> Result<(), JsValue> {
        match &self.palette {
            Some(palette) => {
                let ready = self.handler.borrow_mut().begin_release();
                if ready {
                    self.complete_release(palette)?;
                }
            }
            None => self.handler.borrow_mut().release(),
        }
        Ok(())
    }
    pub(super) fn finish_pending_release(&self) -> Result<(), JsValue> {
        let pending = self.handler.borrow_mut().take_pending_release();
        if pending {
            if let Some(palette) = &self.palette {
                self.complete_release(palette)?;
            }
        }
        Ok(())
    }
    pub(super) fn reset_palette(&self) -> Result<(), JsValue> {
        let ready = self.handler.borrow().is_ready();
        if ready {
            self.release()?;
            self.handler.borrow_mut().clear_palette();
            let limit = self.palette_limit()?;
            self.handler.borrow_mut().finish_reset(limit);
        }
        Ok(())
    }
}

#[wasm_bindgen(inline_js = "
export function imageSixelReleaseWarning(error) {
  return `SIXEL: error while decoding image - ${error}`;
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = imageSixelReleaseWarning)]
    pub(super) fn release_warning(error: &JsValue) -> Result<js_sys::JsString, JsValue>;
}
