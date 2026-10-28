//! Browser-specific event payloads; callable subscriptions share one adapter.
use js_sys::JsString;
use wasm_bindgen::prelude::*;
pub use xterm_wasm_support::events::{callable, empty, text};

#[wasm_bindgen(inline_js = "
export function pairEvent(ak, av, bk, bv) { return { [ak]: av, [bk]: bv }; }
export function keyEvent(key, domEvent) { return { key, domEvent }; }
export function dimensionsEvent(cw, ch, sw, sh, dw, dh, tw, th, w, h, left, top) {
  return {
    css: { canvas: { width: cw, height: ch }, cell: { width: sw, height: sh } },
    device: { canvas: { width: dw, height: dh }, cell: { width: tw, height: th },
      char: { width: w, height: h, left, top } }
  };
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = pairEvent)]
    fn pair_event(ak: &str, av: f64, bk: &str, bv: f64) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = keyEvent)]
    pub fn key_event(key: &JsString, event: &web_sys::KeyboardEvent) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = dimensionsEvent)]
    fn dimensions_event(
        cw: f64,
        ch: f64,
        sw: f64,
        sh: f64,
        dw: f64,
        dh: f64,
        tw: f64,
        th: f64,
        w: f64,
        h: f64,
        left: f64,
        top: f64,
    ) -> Result<JsValue, JsValue>;
}
pub fn pair(first: (&str, usize), second: (&str, usize)) -> Result<JsValue, JsValue> {
    pair_numbers((first.0, first.1 as f64), (second.0, second.1 as f64))
}
pub fn pair_numbers(first: (&str, f64), second: (&str, f64)) -> Result<JsValue, JsValue> {
    pair_event(first.0, first.1, second.0, second.1)
}
pub fn dimensions_value(value: &crate::api::RenderDimensions) -> Result<JsValue, JsValue> {
    let css = value.css;
    let device = value.device;
    dimensions_event(
        css.canvas.width,
        css.canvas.height,
        css.cell.width,
        css.cell.height,
        device.canvas.width,
        device.canvas.height,
        device.cell.width,
        device.cell.height,
        device.character.width,
        device.character.height,
        device.character.left,
        device.character.top,
    )
}
