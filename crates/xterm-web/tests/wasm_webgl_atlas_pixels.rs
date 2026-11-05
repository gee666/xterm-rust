// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
#![cfg(target_arch = "wasm32")]
#[path = "support/atlas_pixel_fixture.rs"]
mod fixture;
use serde_json::Value;
use wasm_bindgen::{prelude::wasm_bindgen, Clamped, JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use xterm_web::{
    api::Size,
    webgl::{
        atlas_canvas::{prepare_canvas, prepare_image, PixelOptions},
        atlas_pixels::GlyphScan,
    },
};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(
    inline_js = "export function injectReadError(ctx, error) { ctx.getImageData = () => { throw error; }; }"
)]
extern "C" {
    #[wasm_bindgen(js_name = injectReadError)]
    fn inject_read_error(context: &CanvasRenderingContext2d, error: &JsValue);
}

#[wasm_bindgen_test]
fn actual_image_data_matches_original_pixel_and_bounds_cases() {
    let input: Value =
        serde_json::from_str(include_str!("fixtures/webgl-atlas-pixels.json")).unwrap();
    assert_eq!(input["cases"].as_array().unwrap().len(), 527);
    for c in input["cases"].as_array().unwrap() {
        let (w, h) = (
            c["width"].as_u64().unwrap() as u32,
            c["height"].as_u64().unwrap() as u32,
        );
        if w == 0 || h == 0 {
            assert!(ImageData::new_with_u8_clamped_array_and_sh(Clamped(&[]), w, h).is_err());
            continue; // Pure pixel scans allow empty data; the browser constructor rejects it.
        }
        let data = fixture::pixels(&c["pixels"]);
        let image = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&data), w, h).unwrap();
        for allow_transparency in [false, true] {
            let options = PixelOptions {
                background_rgba: c["bg"].as_u64().unwrap() as u32,
                foreground_rgba: c["fg"].as_u64().unwrap() as u32,
                allow_transparency,
                clear_threshold: c["threshold"].as_bool().unwrap(),
            };
            let result = prepare_image(&image, fixture::scan(&c["scan"]), options).unwrap();
            let empty = if allow_transparency {
                c["beforeTransparent"].as_bool().unwrap()
            } else {
                c["empty"].as_bool().unwrap()
            };
            assert_eq!(
                result.is_none(),
                empty,
                "{} empty/transparency={allow_transparency}",
                c["name"]
            );
            if let Some(bitmap) = result {
                let expected = if allow_transparency {
                    &c["pixels"]
                } else {
                    &c["cleared"]
                };
                assert_eq!(bitmap.image.data().0, fixture::pixels(expected));
                fixture::assert_bounds(
                    bitmap.bounds,
                    if allow_transparency {
                        &c["beforeBounds"]
                    } else {
                        &c["afterBounds"]
                    },
                    c["name"].as_str().unwrap(),
                );
                assert_eq!((bitmap.image.width(), bitmap.image.height()), (w, h));
            }
        }
    }
}
fn canvas() -> (HtmlCanvasElement, CanvasRenderingContext2d) {
    let canvas: HtmlCanvasElement = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();
    canvas.set_width(8);
    canvas.set_height(6);
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();
    (canvas, context)
}
fn scan() -> GlyphScan {
    GlyphScan {
        allowed_width: 6,
        padding: 2,
        restricted: false,
        custom: false,
        cell: Size {
            width: 6.0,
            height: 6.0,
        },
        character: Size {
            width: 6.0,
            height: 6.0,
        },
        line_height: 1.0,
    }
}
fn options() -> PixelOptions {
    PixelOptions {
        background_rgba: 0x000000ff,
        foreground_rgba: 0xffffffff,
        allow_transparency: false,
        clear_threshold: false,
    }
}
#[wasm_bindgen_test]
fn reads_real_canvas_and_returns_filtered_crop_without_mutating_canvas() {
    let (_canvas, ctx) = canvas();
    ctx.set_fill_style_str("black");
    ctx.fill_rect(0.0, 0.0, 8.0, 6.0);
    ctx.set_fill_style_str("white");
    ctx.fill_rect(2.0, 1.0, 3.0, 2.0);
    let bitmap = prepare_canvas(&ctx, scan(), options()).unwrap().unwrap();
    assert_eq!(
        (
            bitmap.bounds.left,
            bitmap.bounds.top,
            bitmap.bounds.right,
            bitmap.bounds.bottom
        ),
        (2, 1, 4, 2)
    );
    assert_eq!(bitmap.bounds.size, [3.0, 2.0]);
    assert_eq!(bitmap.bounds.offset, [0.0, 1.0]);
    assert_eq!(&bitmap.image.data().0[..4], &[0, 0, 0, 0]);
    assert_eq!(
        &ctx.get_image_data(0.0, 0.0, 1.0, 1.0).unwrap().data().0,
        &[0, 0, 0, 255]
    );
    ctx.clear_rect(0.0, 0.0, 8.0, 6.0);
    let mut options = options();
    options.allow_transparency = true;
    assert!(prepare_canvas(&ctx, scan(), options).unwrap().is_none());
}
#[wasm_bindgen_test]
fn preserves_canvas_read_errors_and_validates_nonempty_scan() {
    let (canvas, ctx) = canvas();
    ctx.set_fill_style_str("white");
    ctx.fill_rect(0.0, 0.0, 8.0, 6.0);
    let mut invalid = scan();
    invalid.allowed_width = 9;
    let error = prepare_canvas(&ctx, invalid, options()).err().unwrap();
    assert!(error.is_instance_of::<js_sys::Error>());
    canvas.set_width(0);
    let error = prepare_canvas(&ctx, scan(), options()).err().unwrap();
    assert_eq!(
        js_sys::Reflect::get(&error, &"name".into())
            .unwrap()
            .as_string()
            .unwrap(),
        "IndexSizeError"
    );
    let token: JsValue = js_sys::Error::new("canvas read callback").into();
    inject_read_error(&ctx, &token);
    assert_eq!(
        prepare_canvas(&ctx, scan(), options()).err().unwrap(),
        token
    );
}
