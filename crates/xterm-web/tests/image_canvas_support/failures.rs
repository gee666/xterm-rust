// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::image_canvas_support::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_addons::image::storage::{AddImageOptions, ImageLayer, ImageResource, StorageRenderer};
use xterm_web::image::{primitives, BrowserImageSurface};

struct Patch {
    target: JsValue,
    key: String,
    descriptor: JsValue,
}
impl Patch {
    fn new(target: JsValue, key: &str, value: JsValue) -> Self {
        let object = js_sys::Object::from(target.clone());
        let descriptor =
            js_sys::Reflect::get_own_property_descriptor(&object, &key.into()).unwrap();
        let replacement = eval("({configurable:true,writable:true})");
        js_sys::Reflect::set(&replacement, &"value".into(), &value).unwrap();
        assert!(
            js_sys::Reflect::define_property(&object, &key.into(), &replacement.into()).unwrap()
        );
        Self {
            target,
            key: key.into(),
            descriptor,
        }
    }
}
impl Drop for Patch {
    fn drop(&mut self) {
        let object = js_sys::Object::from(self.target.clone());
        if self.descriptor.is_undefined() {
            js_sys::Reflect::delete_property(&object, &self.key.clone().into()).unwrap();
        } else {
            js_sys::Reflect::define_property(
                &object,
                &self.key.clone().into(),
                &self.descriptor.clone().into(),
            )
            .unwrap();
        }
    }
}

#[wasm_bindgen_test(async)]
async fn unavailable_primitives_follow_original_fallbacks() {
    let h = Harness::new(3.0, 4.0);
    let image = h.image(3, 4);
    let canvas = surface(&image);
    let context = call(&canvas, "getContext", &["2d".into()]);
    let patch = Patch::new(js_sys::global().into(), "ImageData", JsValue::UNDEFINED);
    let bytes = js_sys::Uint8Array::from(&[8, 16, 32, 255][..]);
    let data = primitives::create_image_data(&context, 1.0, 1.0, &bytes.buffer().into()).unwrap();
    drop(patch);
    assert_eq!(
        js_sys::Uint8Array::new(&get(&get(&data, "data"), "buffer")).to_vec(),
        bytes.to_vec()
    );
    let patch = Patch::new(
        js_sys::global().into(),
        "createImageBitmap",
        JsValue::UNDEFINED,
    );
    let promise = primitives::create_image_bitmap(&canvas)
        .unwrap()
        .dyn_into::<js_sys::Promise>()
        .unwrap();
    let actual = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
    drop(patch);
    assert!(actual.is_undefined());
}

#[wasm_bindgen_test]
fn no_2d_context_removes_new_layer_without_reporting_it_present() {
    let h = Harness::new(3.0, 4.0);
    let prototype = eval("HTMLCanvasElement.prototype");
    let patch = Patch::new(prototype, "getContext", eval("(function(){return null})"));
    h.renderer.insert_layer(ImageLayer::Top).unwrap();
    drop(patch);
    assert!(!h.renderer.has_layer(ImageLayer::Top));
    assert_eq!(
        get(&get(&h.host.screen, "children"), "length").as_f64(),
        Some(1.0)
    );
}

#[wasm_bindgen_test]
fn rescale_and_extract_context_failure_preserves_original_resource() {
    let h = Harness::new(3.0, 4.0);
    let image = h.image(6, 8);
    let id = h
        .storage
        .add_image(image.clone(), AddImageOptions::default())
        .unwrap();
    let spec = h.storage.image_spec(id).unwrap();
    h.renderer.insert_layer(ImageLayer::Top).unwrap();
    h.host.size(6.0, 8.0);
    let patch = Patch::new(
        eval("HTMLCanvasElement.prototype"),
        "getContext",
        eval("(function(){return null})"),
    );
    h.renderer.draw(&spec, 0.0, 0.0, 0.0, 1.0).unwrap();
    assert!(h.renderer.extract_tile(&spec, 0.0).unwrap().is_none());
    drop(patch);
    assert!(spec.actual_resource().unwrap().same_identity(&image));
    assert_eq!(spec.actual_cell_size.get().width, 3.0);
}

#[wasm_bindgen_test]
fn draw_and_bitmap_close_errors_keep_thrown_js_identity_and_partial_state() {
    let h = Harness::new(3.0, 4.0);
    let image = h.image(3, 4);
    let id = h
        .storage
        .add_image(image, AddImageOptions::default())
        .unwrap();
    let spec = h.storage.image_spec(id).unwrap();
    h.renderer.insert_layer(ImageLayer::Top).unwrap();
    let failure = eval("({sentinel:'draw'})");
    let global = js_sys::global();
    js_sys::Reflect::set(&global, &"imageCanvasFailure".into(), &failure).unwrap();
    let patch = Patch::new(
        eval("CanvasRenderingContext2D.prototype"),
        "drawImage",
        eval("(function(){throw globalThis.imageCanvasFailure})"),
    );
    let error = StorageRenderer::draw(h.renderer.as_ref(), &spec, 0.0, 0, 0, 1).unwrap_err();
    drop(patch);
    assert!(xterm_wasm_support::js_error::to_js_error(&error) == failure);
    let object = eval("({width:3,height:4,close(){throw globalThis.imageCanvasFailure}})");
    let resource = ImageResource::new(BrowserImageSurface::bitmap(object));
    let id = h
        .storage
        .add_image(resource, AddImageOptions::default())
        .unwrap();
    let error = h.storage.delete_image(id).unwrap_err();
    assert!(h.storage.image_spec(id).is_none());
    assert!(xterm_wasm_support::js_error::to_js_error(&error) == failure);
    js_sys::Reflect::delete_property(&global, &"imageCanvasFailure".into()).unwrap();
}

#[wasm_bindgen_test(async)]
async fn pending_placeholder_bitmap_uses_source_late_assignment_and_close_order() {
    let h = Harness::new(3.0, 4.0);
    h.renderer.insert_layer(ImageLayer::Top).unwrap();
    let patch = Patch::new(
        js_sys::global().into(),
        "createImageBitmap",
        eval("(function(){return new Promise(resolve=>globalThis.resolveImageBitmap=resolve)})"),
    );
    h.renderer.show_placeholder(true).unwrap();
    h.renderer.show_placeholder(false).unwrap();
    eval("globalThis.lateBitmap={close(){globalThis.imageBitmapClosed=(globalThis.imageBitmapClosed||0)+1}}; resolveImageBitmap(lateBitmap)");
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::UNDEFINED))
        .await
        .unwrap();
    h.renderer.show_placeholder(false).unwrap();
    assert_eq!(eval("globalThis.imageBitmapClosed").as_f64(), Some(1.0));
    h.renderer.dispose().unwrap();
    assert_eq!(eval("globalThis.imageBitmapClosed").as_f64(), Some(1.0));
    drop(patch);
    eval("delete globalThis.resolveImageBitmap; delete globalThis.lateBitmap; delete globalThis.imageBitmapClosed");
}
