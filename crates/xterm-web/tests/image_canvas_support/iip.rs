// Copyright (c) 2023 The xterm.js authors. MIT license.
use super::image_canvas_support::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_addons::image::{
    iip::{
        geometry::IipGeometry,
        handler::IipHandler,
        storage::IipImageStorage,
        types::{DisplayRequest, IipEffect, IipOptions},
    },
    storage::ImageLayer,
    NameDecodePolicy,
};
use xterm_web::image::{IipDecodedImage, IipImageDecoder};

const PALETTE: &[u8] = include_bytes!("../../../xterm-addons/tests/fixtures/qoi/palette.qoi");
const RGBA: &[u8] = include_bytes!("../../../xterm-addons/tests/fixtures/qoi/palette.rgba");
const PNG: &[u8] =
    include_bytes!("../../../xterm-addons/tests/fixtures/image-metrics/w3c_home.png");

fn sequence(bytes: &[u8], width: u32, height: u32) -> String {
    let binary: String = bytes.iter().map(|&byte| char::from(byte)).collect();
    let base64 = call(&js_sys::global(), "btoa", &[binary.into()])
        .as_string()
        .unwrap();
    format!("File=inline=1;width={width}px;height={height}px;preserveAspectRatio=0:{base64}")
}
fn request(sequence: &str) -> DisplayRequest {
    let mut handler =
        IipHandler::new(IipOptions::default(), NameDecodePolicy::BrowserAtob).unwrap();
    handler.start();
    let units: Vec<_> = sequence.chars().map(u32::from).collect();
    // Exercise the real incremental header/base64 path, not an invented bitmap.
    for chunk in units.chunks(23) {
        handler.put(chunk, 0, chunk.len()).unwrap();
    }
    let mut effects = handler
        .end(
            true,
            &IipGeometry {
                columns: 80.0,
                rows: 24.0,
                render: None,
                device_pixel_ratio: None,
            },
        )
        .unwrap();
    assert_eq!(effects.len(), 1);
    match effects.remove(0) {
        IipEffect::Display(request) => request,
        effect => panic!("expected display, got {effect:?}"),
    }
}
async fn source_image(sequence: &str) -> JsValue {
    let run = js_sys::Function::new_with_args(
        "text",
        "return (async () => {
          let image;
          const handler = new SourceIIPHandler(
            {pixelLimit:16777216,iipSizeLimit:33554432}, {},
            {addImage(value){ image=value; }}, {cols:80,rows:24,_core:{}});
          handler.start();
          const units=Uint32Array.from(text,c=>c.charCodeAt(0));
          for(let i=0;i<units.length;i+=23)handler.put(units,i,Math.min(i+23,units.length));
          const result=handler.end(true);
          const synchronous=!(result instanceof Promise);
          await result;
          return {image,synchronous};
        })();",
    );
    let promise = run
        .call1(&JsValue::UNDEFINED, &sequence.into())
        .unwrap()
        .dyn_into::<js_sys::Promise>()
        .unwrap();
    JsFuture::from(promise).await.unwrap()
}
fn canvas(image: JsValue) -> JsValue {
    if image.is_instance_of::<web_sys::HtmlCanvasElement>() {
        return image;
    }
    let canvas = xterm_web::image::primitives::create_canvas(
        &JsValue::UNDEFINED,
        get(&image, "width").as_f64().unwrap(),
        get(&image, "height").as_f64().unwrap(),
    )
    .unwrap();
    let ctx = call(&canvas, "getContext", &["2d".into()]);
    call(&ctx, "drawImage", &[image.clone(), 0.into(), 0.into()]);
    call(&image, "close", &[]);
    canvas
}

#[wasm_bindgen_test(async)]
async fn iip_palette_qoi_fast_path_matches_source_pixels_and_real_storage() {
    let sequence = sequence(PALETTE, 640, 80);
    let reference = source_image(&sequence).await;
    assert_eq!(get(&reference, "synchronous").as_bool(), Some(true));
    let decoder = IipImageDecoder::default();
    let IipDecodedImage::Immediate(image) = decoder.prepare(&request(&sequence)).unwrap() else {
        panic!("unscaled QOI must not cross a promise boundary");
    };
    assert_eq!(pixels(&surface(&image)), RGBA);
    assert_eq!(pixels(&get(&reference, "image")), RGBA);
    let h = Harness::new(3.0, 4.0);
    IipImageStorage::new(h.storage.clone())
        .add_image(image)
        .unwrap();
    h.storage.render(0, 9).unwrap();
    assert!(
        pixels(&h.renderer.canvas(ImageLayer::Top).unwrap().unwrap())
            .chunks_exact(4)
            .any(|pixel| pixel[3] != 0)
    );
    assert_eq!(h.storage.image_count(), 1);
}

#[wasm_bindgen_test(async)]
async fn iip_qoi_and_png_resize_match_source_and_pending_survives_decoder_reset() {
    for bytes in [PALETTE, PNG] {
        let sequence = sequence(bytes, 80, 40);
        let reference = source_image(&sequence).await;
        assert_eq!(get(&reference, "synchronous").as_bool(), Some(false));
        let decoder = IipImageDecoder::default();
        let IipDecodedImage::Pending(pending) = decoder.prepare(&request(&sequence)).unwrap()
        else {
            panic!("resized images require a bitmap promise");
        };
        decoder.reset();
        drop(decoder);
        let actual = pending.resolve().await.unwrap();
        assert_eq!(
            (actual.width().unwrap(), actual.height().unwrap()),
            (80, 40)
        );
        let actual_canvas = canvas(surface(&actual));
        let reference_canvas = canvas(get(&reference, "image"));
        assert_eq!(pixels(&actual_canvas), pixels(&reference_canvas));
    }
}
