// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Services shared with the actual terminal, not a second canvas/cache authority.
use super::{
    browser_cache::{AtlasRequest, AtlasServices, BrowserAtlasCache},
    cache::CacheError,
    config::{AtlasDimensions, AtlasOptions, FontWeight},
    texture_atlas::TextureAtlas,
};
use crate::{
    core_browser::CoreBrowserService,
    js_error::to_js_error,
    theme::{ColorSet, ThemeService},
};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use xterm_core::{headless::Terminal, options::OptionValue};

pub struct LayerServices {
    pub terminal: Rc<Terminal>,
    pub browser: Rc<CoreBrowserService>,
    pub theme: Rc<ThemeService>,
    /// Obtain this from the terminal's RuntimeContext::webgl_atlas_cache.
    pub cache: Rc<BrowserAtlasCache<Terminal>>,
}
impl LayerServices {
    pub fn options(&self) -> Result<AtlasOptions, JsValue> {
        let options = self.terminal.options();
        let number = |key| {
            options
                .get(key)
                .and_then(|v| v.to_number())
                .map_err(|e| to_js_error(&e))
        };
        let text = |key| {
            options
                .get(key)
                .and_then(|v| v.to_utf16())
                .map_err(|e| to_js_error(&e))
        };
        let flag = |key| {
            options
                .get(key)
                .map(|v| v.is_truthy())
                .map_err(|e| to_js_error(&e))
        };
        let weight = |key| -> Result<FontWeight, JsValue> {
            Ok(match options.get(key).map_err(|e| to_js_error(&e))? {
                OptionValue::Number(value) => FontWeight::Number(value),
                value => FontWeight::Text(value.to_utf16().map_err(|e| to_js_error(&e))?.into()),
            })
        };
        Ok(AtlasOptions {
            letter_spacing: number("letterSpacing")?,
            line_height: number("lineHeight")?,
            font_family: text("fontFamily")?.into(),
            font_size: number("fontSize")?,
            font_weight: weight("fontWeight")?,
            font_weight_bold: weight("fontWeightBold")?,
            allow_transparency: flag("allowTransparency")?,
            draw_bold_text_in_bright_colors: flag("drawBoldTextInBrightColors")?,
            minimum_contrast_ratio: number("minimumContrastRatio")?,
        })
    }
    pub fn acquire(
        &self,
        dimensions: AtlasDimensions,
        colors: &ColorSet,
    ) -> Result<Rc<TextureAtlas>, JsValue> {
        let options = self.options()?;
        self.cache
            .acquire(
                &self.terminal,
                AtlasRequest {
                    dimensions,
                    options: &options,
                    colors,
                    custom_glyphs: true,
                },
                || {
                    self.terminal
                        .with_state(|state| AtlasServices {
                            unicode: state.unicode.clone(),
                            log: state.logger.clone(),
                        })
                        .map_err(|e| to_js_error(&e))
                },
            )
            .map_err(|error| match error {
                CacheError::Config(error) => js_sys::TypeError::new(&error.to_string()).into(),
                CacheError::Create(error) | CacheError::Dispose(error) => error,
            })
    }
}
