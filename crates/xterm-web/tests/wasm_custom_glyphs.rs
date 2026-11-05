// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
#![cfg(target_arch = "wasm32")]
use serde_json::Value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::CanvasRenderingContext2d;
use xterm_core::log_service::LogService;
use xterm_web::webgl::custom_glyphs::try_draw_custom_glyph;
#[path = "support/custom_glyph_canvas.rs"]
mod canvas;
use canvas::{context, params};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(inline_js = "
export function sourceCustomGlyph(ctx, chars, p, background, variant) {
  return globalThis.CustomGlyphSource.tryDrawCustomGlyph(ctx,String.fromCharCode(...chars),...p,{error(message){throw new Error(message);}},background,variant);
}
export function adoptCustomGlyphCanvases(a,b) {
  const frame=document.createElement('iframe');document.body.appendChild(frame);
  const doc=frame.contentDocument;doc.body.append(a.canvas,b.canvas);
  doc.createElement=()=>{throw new Error('Must not use adopted canvas ownerDocument');};
  return frame;
}
export function customGlyphState(ctx) { return JSON.stringify([typeof ctx.fillStyle === 'string' ? ctx.fillStyle : Object.prototype.toString.call(ctx.fillStyle),ctx.strokeStyle,ctx.lineWidth,ctx.lineCap]); }
")]
extern "C" {
    #[wasm_bindgen(catch,js_name = sourceCustomGlyph)]
    fn source(
        ctx: &CanvasRenderingContext2d,
        chars: &[u16],
        params: &js_sys::Array,
        background: &str,
        variant: i32,
    ) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch,js_name = adoptCustomGlyphCanvases)]
    fn adopt(
        a: &CanvasRenderingContext2d,
        b: &CanvasRenderingContext2d,
    ) -> Result<web_sys::HtmlElement, JsValue>;
    #[wasm_bindgen(js_name = customGlyphState)]
    fn state(ctx: &CanvasRenderingContext2d) -> String;
}
fn compare(chars: &[u16], p: &[f64; 8], variant: i32, color: &str, log: &LogService) {
    let original = context();
    let rust = context();
    original.set_fill_style_str(color);
    rust.set_fill_style_str(color);
    let values = js_sys::Array::new();
    for &value in p {
        values.push(&value.into());
    }
    let expected = source(&original, chars, &values, "#112233", variant).unwrap();
    let actual = try_draw_custom_glyph(&rust, chars, params(p, variant), log).unwrap();
    assert_eq!(actual, expected, "chars{chars:?}");
    assert_eq!(state(&rust), state(&original), "chars{chars:?} state");
    let actual = rust.get_image_data(0.0, 0.0, 48.0, 48.0).unwrap().data().0;
    let expected = original
        .get_image_data(0.0, 0.0, 48.0, 48.0)
        .unwrap()
        .data()
        .0;
    if actual != expected {
        let difference = actual
            .iter()
            .zip(&expected)
            .enumerate()
            .find(|(_, (a, b))| a != b)
            .unwrap();
        panic!(
            "chars{chars:?} p{p:?} variant{variant} color{color}: pixel byte{} actual{} expected{}",
            difference.0,
            (difference.1).0,
            (difference.1).1
        );
    }
}
#[wasm_bindgen_test]
fn all_glyphs_match_source_pixels_and_context_state() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/custom-glyphs.json")).unwrap();
    let log = canvas::log();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2355);
    for case in cases {
        let chars: Vec<_> = case["chars"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u16)
            .collect();
        let mut p = [0.0; 8];
        for (i, v) in p.iter_mut().enumerate() {
            *v = case["p"][i].as_f64().unwrap();
        }
        compare(&chars, &p, 3, "#e06090", &log);
    }
}
#[wasm_bindgen_test]
fn patterns_match_all_variants_and_alpha() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/custom-glyphs.json")).unwrap();
    let log = canvas::log();
    for cp in fixture["patternCodepoints"].as_array().unwrap() {
        let c = char::from_u32(cp.as_u64().unwrap() as u32).unwrap();
        let chars: Vec<_> = c.to_string().encode_utf16().collect();
        for variant in 0..4 {
            for color in ["#e06090", "#e0609080", "rgba(31,71,119,0.5)"] {
                compare(
                    &chars,
                    &[2.5, 3.25, 9.0, 17.0, 7.0, 13.0, 15.0, 1.25],
                    variant,
                    color,
                    &log,
                );
            }
        }
    }
}
#[wasm_bindgen_test]
fn pattern_cache_preserves_definition_and_color_identity() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/custom-glyphs.json")).unwrap();
    let log = canvas::log();
    let a = context();
    let b = context();
    let p = [0.0, 0.0, 9.0, 17.0, 7.0, 13.0, 15.0, 1.25];
    let values = js_sys::Array::new();
    for &v in &p {
        values.push(&v.into());
    }
    let mut source_styles = Vec::new();
    let mut rust_styles = Vec::new();
    for color in ["#4aa1c9", "#4aa1c9", "#4aa1c980"] {
        for cp in fixture["patternCodepoints"].as_array().unwrap() {
            let chars: Vec<_> = char::from_u32(cp.as_u64().unwrap() as u32)
                .unwrap()
                .to_string()
                .encode_utf16()
                .collect();
            a.set_fill_style_str(color);
            b.set_fill_style_str(color);
            source(&a, &chars, &values, "#112233", 0).unwrap();
            try_draw_custom_glyph(&b, &chars, params(&p, 0), &log).unwrap();
            source_styles.push(a.fill_style());
            rust_styles.push(b.fill_style());
        }
    }
    for i in 0..source_styles.len() {
        for j in 0..source_styles.len() {
            assert_eq!(
                rust_styles[i] == rust_styles[j],
                source_styles[i] == source_styles[j],
                "pattern identity {i}/{j}"
            );
        }
    }
}

#[wasm_bindgen_test]
fn uncached_pattern_uses_main_realm_after_canvas_adoption() {
    let log = canvas::log();
    let a = context();
    let b = context();
    let frame = adopt(&a, &b).unwrap();
    a.set_fill_style_str("#19c3f1");
    b.set_fill_style_str("#19c3f1");
    let p = [0.0, 0.0, 9.0, 17.0, 7.0, 13.0, 15.0, 1.25];
    let values = js_sys::Array::new();
    for &v in &p {
        values.push(&v.into());
    }
    let expected = source(&a, &[0x2591], &values, "#112233", 2);
    let actual = try_draw_custom_glyph(&b, &[0x2591], params(&p, 2), &log);
    frame.remove();
    assert_eq!(actual.unwrap(), expected.unwrap());
    assert_eq!(
        b.get_image_data(0.0, 0.0, 48.0, 48.0).unwrap().data().0,
        a.get_image_data(0.0, 0.0, 48.0, 48.0).unwrap().data().0
    );
}

#[wasm_bindgen_test]
fn nonstring_pattern_fill_throws_like_source() {
    let log = canvas::log();
    let a = context();
    let b = context();
    for ctx in [&a, &b] {
        let gradient = ctx.create_linear_gradient(0.0, 0.0, 5.0, 5.0);
        ctx.set_fill_style_canvas_gradient(&gradient);
    }
    let p = [0.0, 0.0, 8.0, 16.0, 8.0, 16.0, 15.0, 1.0];
    let values = js_sys::Array::new();
    for &v in &p {
        values.push(&v.into());
    }
    let expected = source(&a, &[0x2591], &values, "#112233", 0).unwrap_err();
    let actual = try_draw_custom_glyph(&b, &[0x2591], params(&p, 0), &log).unwrap_err();
    assert_eq!(
        js_sys::Reflect::get(&actual, &"message".into()).unwrap(),
        js_sys::Reflect::get(&expected, &"message".into()).unwrap()
    );
}
