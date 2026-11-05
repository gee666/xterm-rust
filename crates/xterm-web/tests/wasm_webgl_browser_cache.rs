// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
#![cfg(target_arch = "wasm32")]
#[path = "support/browser_atlas_cache.rs"]
mod support;
mod webgl_support;
use serde_json::Value;
use std::rc::Rc;
use support::{Capture, Harness};
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_test::*;
use xterm_web::webgl::cache::CacheError;
wasm_bindgen_test_configure!(run_in_browser);

fn run(name: &str) {
    let expected: Value = serde_json::from_str(&support::source(name)).unwrap();
    let capture = Capture::new();
    let h = Rc::new(Harness::new());
    match name {
        "sharing" => {
            let first = h.acquire(&h.a, "a", 15.0);
            let shared = h.acquire(&h.b, "b", 15.0);
            h.acquire(&h.a, "a", 15.0);
            h.draw(&first);
            h.snapshot(&shared);
            shared.clear_texture().unwrap();
            h.snapshot(&first);
            h.remove(&h.a);
            h.snapshot(&shared);
            h.remove(&h.b);
            h.remove(&h.b);
            h.acquire(&h.c, "c", 15.0);
        }
        "move" => {
            h.acquire(&h.a, "a", 15.0);
            h.acquire(&h.b, "b", 15.0);
            h.acquire(&h.a, "a", 16.0);
            h.remove(&h.b);
            h.acquire(&h.c, "c", 16.0);
            h.remove(&h.a);
            h.remove(&h.c);
        }
        "failure" => {
            h.acquire(&h.a, "a", 15.0);
            support::fail_remove();
            let result = h.try_acquire(&h.a, 16.0, || {
                panic!("failed disposal must precede services")
            });
            assert!(matches!(result, Err(CacheError::Dispose(e)) if e == support::failure()));
            support::note("caught");
            h.acquire(&h.a, "a", 15.0);
            h.acquire(&h.a, "a", 16.0);
            h.remove(&h.a);
        }
        "creation_failure" => {
            h.acquire(&h.a, "a", 15.0);
            support::fail_create();
            let result = h.try_acquire(&h.a, 16.0, || Ok(h.services("a")));
            assert!(matches!(result, Err(CacheError::Create(e)) if e == support::failure()));
            support::note("caught");
            h.acquire(&h.a, "a", 15.0);
            h.remove(&h.a);
        }
        "services_failure" => {
            h.acquire(&h.a, "a", 15.0);
            let result = h.try_acquire(&h.a, 16.0, || {
                support::note("services:a");
                Err(support::failure())
            });
            assert!(matches!(result, Err(CacheError::Create(e)) if e == support::failure()));
            support::note("caught");
            h.acquire(&h.a, "a", 15.0);
            h.remove(&h.a);
        }
        "reentry" => {
            h.acquire(&h.a, "a", 15.0);
            h.acquire(&h.b, "b", 16.0);
            let pool = h.pool.clone();
            let a = Rc::downgrade(&h.a);
            let callback = Closure::once(move || pool.remove_terminal(&a).unwrap());
            support::reenter_remove(callback.as_ref().unchecked_ref());
            h.remove(&h.a);
            h.acquire(&h.b, "b", 16.0);
            h.remove(&h.b);
        }
        "canvas_reentry" => {
            let nested = h.clone();
            let callback = Closure::once(move || {
                nested.acquire(&nested.b, "b", 15.0);
            });
            support::reenter_create(callback.as_ref().unchecked_ref());
            h.acquire(&h.a, "a", 15.0);
            h.acquire(&h.c, "c", 15.0);
            h.remove(&h.a);
            h.remove(&h.b);
            h.remove(&h.c);
        }
        "create_reentry" => {
            let d = Harness::terminal();
            h.try_acquire(&d, 15.0, || {
                support::note("services:d");
                h.acquire(&h.b, "b", 15.0);
                Ok(Harness::raw_services())
            })
            .unwrap();
            h.acquire(&h.a, "a", 15.0);
            h.remove(&d);
            h.remove(&h.a);
            h.remove(&h.b);
        }
        _ => panic!("unknown scenario"),
    }
    let actual = h.output(capture.finish());
    web_sys::console::info_1(&format!("cache-source {name} {expected}").into());
    web_sys::console::info_1(&format!("cache-rust {name} {actual}").into());
    if name == "sharing" {
        assert_ne!(
            actual["pixels"][0], actual["pixels"][1],
            "drawn glyph must not match cleared pixels"
        );
        assert_eq!(actual["pixels"][1], actual["pixels"][2]);
    }
    assert_eq!(actual, expected, "unchanged CharAtlasCache scenario {name}");
}

#[wasm_bindgen_test]
fn actual_shared_canvas_owners_pixels_and_upload_clock() {
    run("sharing");
}
#[wasm_bindgen_test]
fn configuration_change_moves_only_one_owner() {
    run("move");
}
#[wasm_bindgen_test]
fn failed_actual_disposal_preserves_owner_and_error_identity() {
    run("failure");
}
#[wasm_bindgen_test]
fn failed_actual_canvas_creation_does_not_publish_an_entry() {
    run("creation_failure");
}
#[wasm_bindgen_test]
fn failed_lazy_services_happen_after_release() {
    run("services_failure");
}
#[wasm_bindgen_test]
fn actual_disposal_reentry_keeps_source_live_splice_behavior() {
    run("reentry");
}
#[wasm_bindgen_test]
fn lazy_service_reentry_inserts_nested_atlas_before_outer() {
    run("create_reentry");
}

#[wasm_bindgen_test]
fn actual_canvas_creation_reentry_preserves_out_of_order_page_versions() {
    run("canvas_reentry");
}

#[wasm_bindgen_test]
fn real_runtime_keeps_one_pool_and_global_document_across_iframes() {
    use xterm_web::{terminal::runtime::RuntimeContext, webgl::browser_cache::AtlasRequest};
    struct Attached(web_sys::Element);
    impl Drop for Attached {
        fn drop(&mut self) {
            self.0.remove();
        }
    }
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.create_element("iframe").unwrap();
    document.body().unwrap().append_child(&element).unwrap();
    let attached = Attached(element);
    let frame: &web_sys::HtmlIFrameElement = attached.0.unchecked_ref();
    let frame_document = frame.content_document().unwrap();
    assert_ne!(document, frame_document);
    let runtime = RuntimeContext::default();
    let pool = runtime.webgl_atlas_cache(&document);
    let moved = runtime.webgl_atlas_cache(&frame_document);
    assert!(Rc::ptr_eq(&pool, &moved));
    let isolated = RuntimeContext::default().webgl_atlas_cache(&frame_document);
    assert!(!Rc::ptr_eq(&pool, &isolated));
    let config = webgl_support::config();
    let owner = Harness::terminal();
    let acquire = |pool: &xterm_web::webgl::browser_cache::BrowserAtlasCache<_>| {
        pool.acquire(
            &owner,
            AtlasRequest {
                dimensions: config.dimensions,
                options: &config.options,
                colors: &config.colors,
                custom_glyphs: true,
            },
            || Ok(Harness::raw_services()),
        )
        .unwrap()
    };
    let main_atlas = acquire(&moved);
    let iframe_atlas = acquire(&isolated);
    assert_eq!(main_atlas.canvases()[0].owner_document().unwrap(), document);
    assert_eq!(
        iframe_atlas.canvases()[0].owner_document().unwrap(),
        frame_document
    );
    pool.remove_terminal(&Rc::downgrade(&owner)).unwrap();
    isolated.remove_terminal(&Rc::downgrade(&owner)).unwrap();
}
