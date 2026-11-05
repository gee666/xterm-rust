// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use xterm_core::{
    log_service::{LogError, LogRecord, LogService, LogSink},
    options::OptionsService,
};
use xterm_web::webgl::custom_glyphs::CustomGlyphDrawParams;

struct StrictLog;
impl LogSink for StrictLog {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        Err(LogError::Sink(String::from_utf16_lossy(&record.message)))
    }
}
pub fn log() -> LogService {
    LogService::new(Rc::new(OptionsService::default()), Rc::new(StrictLog)).unwrap()
}
pub fn context() -> CanvasRenderingContext2d {
    let canvas: HtmlCanvasElement = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();
    canvas.set_width(48);
    canvas.set_height(48);
    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap()
}
pub fn params(p: &[f64; 8], variant: i32) -> CustomGlyphDrawParams<'static> {
    CustomGlyphDrawParams {
        x_offset: p[0],
        y_offset: p[1],
        device_cell_width: p[2],
        device_cell_height: p[3],
        device_char_width: p[4],
        device_char_height: p[5],
        font_size: p[6],
        device_pixel_ratio: p[7],
        background_color: Some("#112233"),
        variant_offset: variant,
    }
}
