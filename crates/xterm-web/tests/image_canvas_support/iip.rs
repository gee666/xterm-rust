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

fn solid_qoi(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut data = b"qoif".to_vec();
    data.extend(width.to_be_bytes());
    data.extend(height.to_be_bytes());
    data.extend([4, 0, 255]);
    data.extend(rgba);
    let mut remaining = width * height - 1;
    while remaining > 0 {
        let count = remaining.min(62);
        data.push(0xc0 + u8::try_from(count - 1).unwrap());
        remaining -= count;
    }
    data.extend([0, 0, 0, 0, 0, 0, 0, 1]);
    data
}

#[wasm_bindgen_test]
fn nested_qoi_transfer_preserves_source_retained_pixel_view() {
    use super::image_canvas_failures::Patch;
    use std::{cell::RefCell, rc::Rc};
    for (width, height, inner_width, inner_height) in
        [(32, 16, 32, 16), (640, 80, 640, 80), (32, 16, 640, 80)]
    {
        let red = sequence(&solid_qoi(width, height, [255, 0, 0, 255]), width, height);
        let green = sequence(
            &solid_qoi(inner_width, inner_height, [0, 255, 0, 255]),
            inner_width,
            inner_height,
        );
        let source = js_sys::Function::new_with_args(
            "outer,inner",
            "
          const images=[]; const Original=ImageData;
          const h=new SourceIIPHandler({pixelLimit:16777216,iipSizeLimit:33554432},{},
            {addImage(value){images.push(value)}},{cols:80,rows:24,_core:{}});
          function write(text){h.start();const units=Uint32Array.from(text,c=>c.charCodeAt(0));
            h.put(units,0,units.length);return h.end(true)}
          let nested=false;
          globalThis.ImageData=function(...args){if(!nested){nested=true;write(inner)}
            return Reflect.construct(Original,args)};
          let error;
          try{write(outer)}catch(cause){error=cause.name}finally{globalThis.ImageData=Original}
          return {images,error}",
        );
        let reference_result = source
            .call2(
                &JsValue::UNDEFINED,
                &red.clone().into(),
                &green.clone().into(),
            )
            .unwrap();
        let reference = js_sys::Array::from(&get(&reference_result, "images"));
        assert_eq!(reference.length(), if width == inner_width { 2 } else { 1 });
        let decoder = Rc::new(IipImageDecoder::default());
        let images = Rc::new(RefCell::new(Vec::new()));
        let reentrant = decoder.clone();
        let captured = images.clone();
        let inner = request(&green);
        let nested = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
            let IipDecodedImage::Immediate(image) = reentrant.prepare(&inner).unwrap() else {
                panic!("nested image must be synchronous");
            };
            captured.borrow_mut().push(image);
        });
        let constructor = js_sys::Function::new_with_args("nested,Original", "let fired=false;
          return function(...args){if(!fired){fired=true;nested()}return Reflect.construct(Original,args)}")
          .call2(&JsValue::UNDEFINED, nested.as_ref(), &get(&js_sys::global(), "ImageData")).unwrap();
        let patch = Patch::new(js_sys::global().into(), "ImageData", constructor);
        let error = match decoder.prepare(&request(&red)) {
            Ok(IipDecodedImage::Immediate(image)) => {
                images.borrow_mut().push(image);
                JsValue::UNDEFINED
            }
            Ok(IipDecodedImage::Pending(_)) => panic!("outer image must be synchronous"),
            Err(error) => get(&error, "name"),
        };
        drop(patch);
        assert!(error == get(&reference_result, "error"));
        assert_eq!(images.borrow().len(), reference.length() as usize);
        let expected =
            [0, 255, 0, 255].repeat(usize::try_from(inner_width * inner_height).unwrap());
        for index in 0..reference.length() {
            assert_eq!(
                pixels(&reference.get(index)),
                expected,
                "source image {index}"
            );
            assert_eq!(
                pixels(&surface(&images.borrow()[index as usize])),
                expected,
                "Rust image {index}"
            );
        }
    }
}
