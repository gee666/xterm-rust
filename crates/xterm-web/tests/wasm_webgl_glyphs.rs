// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
#![cfg(target_arch = "wasm32")]
#[path = "support/webgl_glyphs.rs"]
mod helpers;
#[path = "support/webgl_glyph_source.rs"]
mod source;
use helpers::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{CanvasRenderingContext2d, WebGl2RenderingContext as Gl};
use xterm_web::webgl::{glyph_atlas::AtlasPage, glyph_gpu::GlyphRenderer};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn real_gpu_draws_multiple_pages_and_compacts_line_prefixes() {
    let (_canvas, gl) = context();
    record_resources(&gl, "");
    let mut renderer = GlyphRenderer::new(gl.clone(), 3, 2, dimensions(), None).unwrap();
    let atlas = TestAtlas::new();
    renderer.set_atlas(atlas.clone());
    renderer.update_cell(cell(0, 65, 0), false).unwrap();
    renderer.update_cell(cell(1, 66, 1), false).unwrap();
    renderer.update_cell(cell(2, 67, 0), false).unwrap();
    clear(&gl);
    renderer.render(&model(&[2, 0])).unwrap();
    assert_eq!(uploads(&gl), 2);
    assert_eq!(pixel(&gl, 2, 2), [255, 0, 0, 255]);
    assert_eq!(pixel(&gl, 10, 2), [0, 255, 0, 255]);
    assert_eq!(pixel(&gl, 18, 2), [0, 0, 0, 0]);
    assert_eq!(pixel(&gl, 2, 12), [0, 0, 0, 0]);
    renderer.render(&model(&[3, 0])).unwrap();
    assert_eq!(uploads(&gl), 2);
    assert_eq!(pixel(&gl, 18, 2), [255, 0, 0, 255]);
    renderer.clear();
    clear(&gl);
    renderer.render(&model(&[3, 2])).unwrap();
    assert_eq!(pixel(&gl, 2, 2), [0, 0, 0, 0]);
    paint(&atlas.pages.borrow()[0].canvas, "rgba(255,0,0,0.5)");
    renderer.invalidate_atlas_textures();
    renderer.update_cell(cell(0, 65, 0), false).unwrap();
    clear(&gl);
    renderer.render(&model(&[1])).unwrap();
    // Source uses SRC_ALPHA for alpha too, so 0.5 squared becomes 0.25.
    assert_eq!(pixel(&gl, 2, 2), [128, 0, 0, 64]);
}

#[wasm_bindgen_test]
fn page_versions_replacement_and_explicit_invalidation_upload_live_canvas() {
    let (_canvas, gl) = context();
    record_resources(&gl, "");
    let mut r = GlyphRenderer::new(gl.clone(), 3, 2, dimensions(), None).unwrap();
    let atlas = TestAtlas::new();
    r.set_atlas(atlas.clone());
    r.update_cell(cell(0, 65, 0), false).unwrap();
    r.render(&model(&[1])).unwrap();
    paint(&atlas.pages.borrow()[0].canvas, "blue");
    r.render(&model(&[1])).unwrap();
    assert_eq!(pixel(&gl, 2, 2), [255, 0, 0, 255]);
    assert_eq!(uploads(&gl), 2);
    r.invalidate_atlas_textures();
    r.render(&model(&[1])).unwrap();
    assert_eq!(pixel(&gl, 2, 2), [0, 0, 255, 255]);
    assert_eq!(uploads(&gl), 4);
    let replacement = canvas(8, 10);
    paint(&replacement, "cyan");
    atlas.pages.borrow_mut()[0] = AtlasPage {
        canvas: replacement,
        version: 3,
    };
    r.render(&model(&[1])).unwrap();
    assert_eq!(pixel(&gl, 2, 2), [0, 255, 255, 255]);
    assert_eq!(uploads(&gl), 5);
    paint(&atlas.pages.borrow()[0].canvas, "lime");
    atlas.pages.borrow_mut()[0].version = 4;
    r.render(&model(&[1])).unwrap();
    assert_eq!(pixel(&gl, 2, 2), [0, 255, 0, 255]);
    assert_eq!(uploads(&gl), 6);
    r.set_atlas(atlas);
    r.render(&model(&[1])).unwrap();
    assert_eq!(uploads(&gl), 8);
}

#[wasm_bindgen_test]
fn frame_epochs_utf16_routing_null_cells_and_rasterizer_errors() {
    let (_canvas, gl) = context();
    let element = canvas(1, 1).dyn_into::<web_sys::HtmlElement>().unwrap();
    let mut r = GlyphRenderer::new(gl.clone(), 3, 2, dimensions(), Some(element.clone())).unwrap();
    assert!(r.begin_frame());
    assert!(r.begin_frame());
    r.update_cell(cell(0, 65, 0), false).unwrap();
    r.render(&model(&[1])).unwrap();
    let atlas = TestAtlas::new();
    r.set_atlas(atlas.clone());
    assert!(r.begin_frame());
    assert!(!r.begin_frame());
    *atlas.version.borrow_mut() = 1;
    assert!(r.begin_frame());
    assert!(!r.begin_frame());
    r.set_atlas(atlas.clone());
    assert!(r.begin_frame());
    let mut c = cell(0, 0x1f680, 0);
    c.chars = &[0xd83d, 0xde80];
    r.update_cell(c, false).unwrap();
    c = cell(1, -2147483583, 0);
    c.chars = &[65, 0x301];
    r.update_cell(c, false).unwrap();
    c = cell(2, 65, 0);
    c.chars = &[];
    r.update_cell(c, false).unwrap();
    assert_eq!(
        atlas
            .calls
            .borrow()
            .iter()
            .map(|c| c.combined)
            .collect::<Vec<_>>(),
        [true, true, false]
    );
    assert_eq!(atlas.calls.borrow()[1].code, Some(-2147483583));
    assert_eq!(atlas.calls.borrow()[0].fg, 17);
    assert!(atlas
        .elements
        .borrow()
        .iter()
        .all(|actual| actual.as_ref() == Some(&element)));
    let error = js_sys::Error::new("rasterization failed");
    *atlas.failure.borrow_mut() = Some(error.clone().into());
    assert_eq!(
        r.update_cell(cell(0, 65, 0), false).unwrap_err(),
        wasm_bindgen::JsValue::from(error)
    );
    let mut null = cell(0, 0, 0);
    r.update_cell(null, false).unwrap();
    null.x = 1;
    null.code = None;
    r.update_cell(null, false).unwrap();
    clear(&gl);
    r.render(&model(&[3])).unwrap();
    assert_eq!(pixel(&gl, 2, 2), [0, 0, 0, 0]);
    assert_eq!(pixel(&gl, 10, 2), [0, 0, 0, 0]);
    assert_eq!(pixel(&gl, 18, 2), [255, 0, 0, 255]);
}

#[wasm_bindgen_test]
fn gpu_clips_left_overlap_rescales_and_handles_resize() {
    let (_canvas, gl) = context();
    let atlas = TestAtlas::new();
    let page = canvas(16, 10);
    paint(&page, "lime");
    let ctx: CanvasRenderingContext2d =
        page.get_context("2d").unwrap().unwrap().dyn_into().unwrap();
    ctx.set_fill_style_str("red");
    ctx.fill_rect(0.0, 0.0, 4.0, 10.0);
    atlas.pages.borrow_mut()[0] = AtlasPage {
        canvas: page,
        version: 3,
    };
    atlas.glyph.borrow_mut().offset = [4.0, 0.0];
    atlas.glyph.borrow_mut().size = [12.0, 10.0];
    atlas.glyph.borrow_mut().texture_size = [0.75, 1.0];
    let mut r = GlyphRenderer::new(gl.clone(), 3, 2, dimensions(), None).unwrap();
    r.set_atlas(atlas.clone());
    let mut c = cell(1, 65, 0);
    r.update_cell(c, false).unwrap();
    clear(&gl);
    r.render(&model(&[2])).unwrap();
    assert_eq!(pixel(&gl, 5, 2), [255, 0, 0, 255]);
    c.bg = 1;
    r.update_cell(c, false).unwrap();
    clear(&gl);
    r.render(&model(&[2])).unwrap();
    assert_eq!(pixel(&gl, 5, 2), [0, 0, 0, 0]);
    assert_eq!(pixel(&gl, 10, 2), [0, 255, 0, 255]);
    atlas.glyph.borrow_mut().offset = [0.0, 0.0];
    atlas.glyph.borrow_mut().size = [16.0, 10.0];
    atlas.glyph.borrow_mut().texture_size = [1.0, 1.0];
    c = cell(0, 0x100, 0);
    r.update_cell(c, true).unwrap();
    clear(&gl);
    r.render(&model(&[1])).unwrap();
    assert_ne!(pixel(&gl, 5, 2), [0, 0, 0, 0]);
    assert_eq!(pixel(&gl, 8, 2), [0, 0, 0, 0]);
    c.code = Some(65);
    r.update_cell(c, true).unwrap();
    clear(&gl);
    r.render(&model(&[1])).unwrap();
    assert_eq!(pixel(&gl, 10, 2), [0, 255, 0, 255]);
    let mut d = dimensions();
    d.device.canvas.width = 12.0;
    d.device.canvas.height = 10.0;
    r.set_dimensions(d);
    r.handle_resize(3, 2).unwrap();
    let viewport = gl.get_parameter(Gl::VIEWPORT).unwrap();
    assert_eq!(js_sys::Int32Array::new(&viewport).to_vec(), [0, 0, 12, 10]);
    r.render(&model(&[3, 3])).unwrap();
    assert_eq!(pixel(&gl, 10, 2), [0, 255, 0, 255]); // Resize clears vertices, not the canvas.
}

#[wasm_bindgen_test]
fn owned_resources_are_deleted_on_disposal_drop_and_partial_failure() {
    for failure in ["shader", "uniform", "buffer", "texture"] {
        let (_canvas, gl) = context();
        record_resources(&gl, failure);
        let error = GlyphRenderer::new(gl.clone(), 3, 2, dimensions(), None)
            .err()
            .expect("injected failure must return Err");
        assert!(error.is_instance_of::<js_sys::Error>());
        gl.use_program(None);
        assert_eq!(live_resources(&gl), 0, "partial failure {failure}");
    }
    let (_canvas, gl) = context();
    record_resources(&gl, "");
    let mut r = GlyphRenderer::new(gl.clone(), 3, 2, dimensions(), None).unwrap();
    let atlas = TestAtlas::new();
    let weak = std::rc::Rc::downgrade(&atlas);
    r.set_atlas(atlas.clone());
    drop(atlas);
    r.update_cell(cell(0, 65, 0), false).unwrap();
    r.render(&model(&[1])).unwrap();
    assert_eq!(live_resources(&gl), r.texture_capacity() + 5);
    r.dispose();
    r.dispose();
    gl.use_program(None);
    assert!(weak.upgrade().is_none());
    assert_eq!(live_resources(&gl), 0);
    assert!(r.render(&model(&[1])).is_err());
    assert!(r.update_cell(cell(0, 65, 0), false).is_err());
    assert!(r.handle_resize(3, 2).is_err());
    let r = GlyphRenderer::new(gl.clone(), 3, 2, dimensions(), None).unwrap();
    assert!(r.texture_capacity() >= 8);
    drop(r);
    gl.use_program(None);
    assert_eq!(live_resources(&gl), 0);
}

#[wasm_bindgen_test]
fn overflow_warns_once_and_context_loss_is_an_error() {
    let (_canvas, gl) = context();
    record_resources(&gl, "");
    let mut r = GlyphRenderer::new(gl.clone(), 3, 2, dimensions(), None).unwrap();
    let atlas = TestAtlas::new();
    let count = r.texture_capacity();
    let template = atlas.pages.borrow()[0].clone();
    atlas.pages.borrow_mut().resize(count + 1, template);
    r.set_atlas(atlas);
    start_warnings();
    let first = r.render(&model(&[]));
    let second = r.render(&model(&[]));
    let warnings = stop_warnings();
    first.unwrap();
    second.unwrap();
    assert_eq!(warnings, 1);
    assert_eq!(uploads(&gl), count);
    lose_context(&gl);
    assert!(gl.is_context_lost());
    assert!(r.render(&model(&[])).is_err());
}
