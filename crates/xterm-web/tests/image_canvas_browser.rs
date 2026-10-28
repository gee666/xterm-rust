#![cfg(target_arch = "wasm32")]
#[path = "image_canvas_support/failures.rs"]
mod image_canvas_failures;
mod image_canvas_support;
use image_canvas_support::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_addons::image::storage::{AddImageOptions, ImageLayer, ImageResource};
use xterm_web::image::{primitives, BrowserImageSurface};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn primitives_use_real_local_documents_and_rgba_bytes() {
    let document = eval("document.implementation.createHTMLDocument('images')");
    let canvas = primitives::create_canvas(&document, 4_294_967_299.8, 2.9).unwrap();
    assert_eq!(get(&canvas, "width").as_f64(), Some(3.0));
    assert_eq!(get(&canvas, "height").as_f64(), Some(2.0));
    assert!(get(&canvas, "ownerDocument") == document);
    let ctx = call(&canvas, "getContext", &["2d".into()]);
    let bytes = js_sys::Uint8Array::from(&[255, 0, 0, 255, 0, 255, 0, 255][..]);
    let data = primitives::create_image_data(&ctx, 2.0, 1.0, &bytes.buffer().into()).unwrap();
    call(&ctx, "putImageData", &[data, 0.into(), 0.into()]);
    assert_eq!(&pixels(&canvas)[..8], bytes.to_vec());
    let fallback = primitives::create_canvas(&JsValue::UNDEFINED, f64::NAN, 0.0).unwrap();
    assert_eq!(get(&fallback, "width").as_f64(), Some(0.0));
}

#[wasm_bindgen_test]
fn layer_order_isolation_clear_and_idempotent_removal_match_source() {
    let h = Harness::new(3.0, 4.0);
    for layer in [ImageLayer::Top, ImageLayer::Bottom, ImageLayer::Top] {
        h.renderer.insert_layer(layer).unwrap();
    }
    assert_eq!(
        get(&get(&h.host.screen, "children"), "length").as_f64(),
        Some(3.0)
    );
    let top = h.renderer.canvas(ImageLayer::Top).unwrap().unwrap();
    let bottom = h.renderer.canvas(ImageLayer::Bottom).unwrap().unwrap();
    assert!(get(&h.host.screen, "firstChild") == bottom);
    assert!(get(&h.host.screen, "lastChild") == top);
    assert_eq!(
        get(&get(&h.host.screen, "style"), "isolation")
            .as_string()
            .as_deref(),
        Some("isolate")
    );
    assert_eq!(
        get(&get(&bottom, "style"), "zIndex").as_string().as_deref(),
        Some("-1")
    );
    for canvas in [&top, &bottom] {
        let ctx = call(canvas, "getContext", &["2d".into()]);
        js_sys::Reflect::set(&ctx, &"fillStyle".into(), &"red".into()).unwrap();
        call(
            &ctx,
            "fillRect",
            &[0.into(), 0.into(), 80.into(), 60.into()],
        );
    }
    h.renderer
        .clear_lines(1.0, 2.0, Some(ImageLayer::Bottom))
        .unwrap();
    assert!(pixels(&top).chunks_exact(4).all(|p| p == [255, 0, 0, 255]));
    let bytes = pixels(&bottom);
    assert_eq!(&bytes[(4 * 80 * 4)..(12 * 80 * 4)], vec![0; 8 * 80 * 4]);
    h.renderer.clear_all(None).unwrap();
    assert!(pixels(&top).iter().all(|p| *p == 0));
    h.renderer.remove_layer(ImageLayer::Top).unwrap();
    h.renderer.remove_layer(ImageLayer::Top).unwrap();
    assert!(!h.renderer.has_layer(ImageLayer::Top));
}

#[wasm_bindgen_test]
fn real_storage_tiles_render_extract_and_return_original_identity() {
    let h = Harness::new(3.0, 4.0);
    let image = h.image(7, 5);
    let id = h
        .storage
        .add_image(image.clone(), AddImageOptions::default())
        .unwrap();
    h.storage.render(0, 9).unwrap();
    let canvas = h.renderer.canvas(ImageLayer::Top).unwrap().unwrap();
    assert!(pixels(&canvas).chunks_exact(4).any(|pixel| pixel[3] == 255));
    let spec = h.storage.image_spec(id).unwrap();
    assert!(spec.actual_resource().unwrap().same_identity(&image));
    let tile = h.renderer.extract_tile(&spec, 5.0).unwrap().unwrap();
    assert_eq!((tile.width().unwrap(), tile.height().unwrap()), (1, 1));
    assert_eq!(pixels(&surface(&tile)), vec![102, 116, 110, 255]);
    h.host.size(6.0, 8.0);
    h.renderer.draw(&spec, 0.0, 0.0, 0.0, 1.0).unwrap();
    assert!(!spec.actual_resource().unwrap().same_identity(&image));
    assert_eq!(spec.actual_resource().unwrap().width().unwrap(), 14);
    h.host.size(3.0, 4.0);
    h.renderer.draw(&spec, 0.0, 0.0, 0.0, 1.0).unwrap();
    assert!(spec.actual_resource().unwrap().same_identity(&image));
    // This is a real engine terminal, not a JS terminal pretending to be Rust.
    assert_eq!(h.terminal.cols().unwrap(), 20);
}

#[wasm_bindgen_test]
fn seeded_fractional_crop_rescale_and_extract_pixels_match_unchanged_source() {
    let h = Harness::new(3.0, 4.0);
    let source_host = Host::new(3.0, 4.0);
    let source = original(&source_host);
    call(&source, "insertLayerToDom", &["top".into()]);
    h.renderer.insert_layer(ImageLayer::Top).unwrap();
    let image = h.image(13, 11);
    let id = h
        .storage
        .add_image(image.clone(), AddImageOptions::default())
        .unwrap();
    let spec = h.storage.image_spec(id).unwrap();
    let source_spec = source_spec(surface(&image), 3.0, 4.0, "top");
    let mut seed = 0x272389u32;
    for step in 0..128 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let width = [3.0, 2.5, 4.25, 6.0][(seed & 3) as usize];
        let height = [4.0, 3.25, 6.5, 8.0][((seed >> 2) & 3) as usize];
        h.host.size(width, height);
        source_host.size(width, height);
        set_dimensions(
            &get(&source, "_terminal"),
            source_host.dimensions.get().unwrap(),
        );
        let tile = ((seed >> 4) % 6) as f64;
        let col = ((seed >> 8) % 4) as f64;
        let row = ((seed >> 10) % 3) as f64;
        let count = ((seed >> 12) % 3 + 1) as f64;
        h.renderer.clear_all(None).unwrap();
        call(&source, "clearAll", &[]);
        h.renderer.draw(&spec, tile, col, row, count).unwrap();
        call(
            &source,
            "draw",
            &[
                source_spec.clone(),
                tile.into(),
                col.into(),
                row.into(),
                count.into(),
            ],
        );
        assert_eq!(
            pixels(&h.renderer.canvas(ImageLayer::Top).unwrap().unwrap()),
            pixels(&get(&source, "canvas")),
            "seed0x272389 step{step}"
        );
        let rust = h.renderer.extract_tile(&spec, tile).unwrap().unwrap();
        let js = call(&source, "extractTile", &[source_spec.clone(), tile.into()]);
        assert_eq!(get(&surface(&rust), "width"), get(&js, "width"));
        assert_eq!(get(&surface(&rust), "height"), get(&js, "height"));
        if rust.width().unwrap() != 0 && rust.height().unwrap() != 0 {
            assert_eq!(pixels(&surface(&rust)), pixels(&js), "extract step{step}");
        }
    }
    call(&source, "dispose", &[]);
}

#[wasm_bindgen_test]
fn placeholder_checkerboard_matches_source_pixels() {
    let h = Harness::new(3.0, 4.0);
    let source_host = Host::new(3.0, 4.0);
    let source = original(&source_host);
    call(&source, "insertLayerToDom", &["top".into()]);
    h.renderer.insert_layer(ImageLayer::Top).unwrap();
    // Use the same global screen.width. Blueprint width is screen aligned.
    h.host
        .screen_width
        .set(eval("screen.width").as_f64().unwrap());
    for (col, row, count) in [(0.0, 0.0, 4.0), (1.0, 1.0, 3.0), (2.0, 2.0, 2.0)] {
        h.renderer.draw_placeholder(col, row, count).unwrap();
        call(
            &source,
            "drawPlaceholder",
            &[col.into(), row.into(), count.into()],
        );
    }
    assert_eq!(
        pixels(&h.renderer.canvas(ImageLayer::Top).unwrap().unwrap()),
        pixels(&get(&source, "canvas"))
    );
    h.renderer.show_placeholder(false).unwrap();
    call(&source, "showPlaceholder", &[false.into()]);
    call(&source, "dispose", &[]);
}

#[wasm_bindgen_test]
fn unchanged_dimensions_do_not_clear_canvas_but_changed_dimensions_do() {
    let h = Harness::new(3.0, 4.0);
    h.renderer.insert_layer(ImageLayer::Top).unwrap();
    h.renderer.draw_placeholder(0.0, 0.0, 1.0).unwrap();
    let canvas = h.renderer.canvas(ImageLayer::Top).unwrap().unwrap();
    let before = pixels(&canvas);
    h.renderer.rescale_canvas().unwrap();
    assert_eq!(pixels(&canvas), before);
    let mut dimensions = h.host.dimensions.get().unwrap();
    dimensions.canvas_width = 90.0;
    h.host.dimensions.set(Some(dimensions));
    h.renderer.rescale_canvas().unwrap();
    assert_eq!(get(&canvas, "width").as_f64(), Some(90.0));
    assert!(pixels(&canvas).iter().all(|p| *p == 0));
}

#[wasm_bindgen_test(async)]
async fn real_bitmaps_convert_pixels_and_storage_closes_original_once() {
    let h = Harness::new(3.0, 4.0);
    let image = h.image(3, 4);
    let promise = primitives::create_image_bitmap(&surface(&image))
        .unwrap()
        .dyn_into::<js_sys::Promise>()
        .unwrap();
    let bitmap = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
    assert!(
        !bitmap.is_undefined(),
        "This test requires a real ImageBitmap, not the unavailable-API branch"
    );
    let resource = ImageResource::new(BrowserImageSurface::bitmap(bitmap.clone()));
    let canvas = h.renderer.bitmap_to_canvas(&resource).unwrap();
    assert_eq!(pixels(&surface(&canvas)), pixels(&surface(&image)));
    let id = h
        .storage
        .add_image(resource, AddImageOptions::default())
        .unwrap();
    h.storage.delete_image(id).unwrap();
    assert_eq!(get(&bitmap, "width").as_f64(), Some(0.0));
    h.storage.delete_image(id).unwrap();
}

#[wasm_bindgen_test]
fn missing_metrics_suppress_drawing_and_extraction_without_changing_specs() {
    let h = Harness::new(3.0, 4.0);
    let id = h
        .storage
        .add_image(h.image(3, 4), AddImageOptions::default())
        .unwrap();
    let spec = h.storage.image_spec(id).unwrap();
    h.renderer.insert_layer(ImageLayer::Top).unwrap();
    h.host.dimensions.set(None);
    h.renderer.draw(&spec, 0.0, 0.0, 0.0, 1.0).unwrap();
    assert!(h.renderer.extract_tile(&spec, 0.0).unwrap().is_none());
    assert_eq!(h.renderer.cell_size().unwrap().width, -1.0);
    assert_eq!(spec.actual_cell_size.get().width, 3.0);
}
