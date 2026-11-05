// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl};
use xterm_web::{
    api::{CharacterDimensions, RenderDimensions, Size},
    webgl::{
        glyph_gpu::GlyphRenderer,
        glyphs::{GlyphCell, GlyphColorNumbers},
        model::RenderModel,
    },
};
fn gpu(atlas: Rc<TextureAtlas>) -> (HtmlCanvasElement, Gl, GlyphRenderer) {
    let doc = web_sys::window().unwrap().document().unwrap();
    let canvas: HtmlCanvasElement = doc.create_element("canvas").unwrap().unchecked_into();
    canvas.set_width(18);
    canvas.set_height(18);
    let options = js_sys::Object::new();
    for key in ["antialias", "depth", "premultipliedAlpha"] {
        js_sys::Reflect::set(&options, &key.into(), &false.into()).unwrap();
    }
    js_sys::Reflect::set(&options, &"preserveDrawingBuffer".into(), &true.into()).unwrap();
    let gl: Gl = canvas
        .get_context_with_context_options("webgl2", &options)
        .unwrap()
        .expect("GPU consumer requires WebGL2")
        .unchecked_into();
    let mut dimensions = RenderDimensions::default();
    dimensions.device.canvas = Size {
        width: 18.0,
        height: 18.0,
    };
    dimensions.device.cell = Size {
        width: 9.0,
        height: 18.0,
    };
    dimensions.device.character = CharacterDimensions {
        width: 8.0,
        height: 16.0,
        left: 0.0,
        top: 0.0,
    };
    let mut renderer = GlyphRenderer::new(gl.clone(), 2, 1, dimensions, None).unwrap();
    renderer.set_atlas(atlas);
    (canvas, gl, renderer)
}
#[wasm_bindgen_test]
fn existing_gpu_consumer_draws_and_refreshes_actual_texture_atlas() {
    let source: Value = serde_json::from_str(&source_scenario("styles").unwrap()).unwrap();
    let atlas = make_atlas(&source["spec"]);
    let (canvas, gl, mut renderer) = gpu(atlas.clone());
    assert!(renderer.begin_frame());
    let cell = GlyphCell {
        x: 0,
        y: 0,
        code: Some(0x2588),
        bg: 0,
        fg: 0,
        ext: 0,
        chars: &[0x2588],
        width: 1.0,
        last_bg: 0,
    };
    renderer.update_cell(cell, false).unwrap();
    let mut model = RenderModel::new();
    model.resize(2, 1).unwrap();
    model.line_lengths[0] = 1;
    renderer.render(&model).unwrap();
    let pixel = |x| {
        let mut bytes = [0; 4];
        gl.read_pixels_with_opt_u8_array(x, 9, 1, 1, Gl::RGBA, Gl::UNSIGNED_BYTE, Some(&mut bytes))
            .unwrap();
        bytes
    };
    assert_eq!(pixel(4), [238, 238, 238, 255]);
    assert_eq!(pixel(13), [0, 0, 0, 0]);
    assert!(!renderer.begin_frame());
    atlas.clear_texture().unwrap();
    assert!(renderer.begin_frame());
    renderer.clear();
    renderer.update_cell(cell, false).unwrap();
    gl.clear_color(0.0, 0.0, 0.0, 0.0);
    gl.clear(Gl::COLOR_BUFFER_BIT);
    renderer.render(&model).unwrap();
    assert_eq!(pixel(4), [238, 238, 238, 255]);
    assert_eq!(gl.get_error(), Gl::NO_ERROR);
    renderer.dispose();
    atlas.dispose().unwrap();
    canvas.remove();
}

#[wasm_bindgen_test]
fn actual_gpu_forwards_original_numbers_to_texture_atlas() {
    let source: Value = serde_json::from_str(&source_scenario("signedCache").unwrap()).unwrap();
    let atlas = make_atlas(&source["spec"]);
    let (canvas, gl, mut renderer) = gpu(atlas.clone());
    let cell = GlyphCell {
        x: 0,
        y: 0,
        code: Some(0x2588),
        bg: 0,
        fg: 0,
        ext: 0,
        chars: &[0x2588],
        width: 1.0,
        last_bg: 0,
    };
    let count = || {
        atlas
            .pages()
            .iter()
            .map(|p| p.glyphs.borrow().len())
            .sum::<usize>()
    };
    let mut expected = 0;
    for field in 0..3 {
        for word in [2147483648_i64, -2147483648] {
            let mut current = [0; 3];
            current[field] = word;
            let numbers = GlyphColorNumbers {
                current,
                previous_background: 0,
            };
            let [bg, fg, ext] = current.map(|n| n as u32);
            let cell = GlyphCell {
                bg,
                fg,
                ext,
                ..cell
            };
            renderer
                .update_cell_with_colors(cell, numbers, false)
                .unwrap();
            expected += 1;
            assert_eq!(count(), expected, "field={field}, number={word}");
            // The actual GPU update must already have installed this exact key.
            let cached = atlas
                .get_glyph_with_colors(GlyphKey::Number(0x2588), current, false, None)
                .unwrap();
            assert_eq!(count(), expected);
            renderer
                .update_cell_with_colors(cell, numbers, false)
                .unwrap();
            assert_eq!(count(), expected);
            assert!(Rc::ptr_eq(
                &cached,
                &atlas
                    .get_glyph_with_colors(GlyphKey::Number(0x2588), current, true, None,)
                    .unwrap()
            ));
        }
    }
    assert_eq!(gl.get_error(), Gl::NO_ERROR);
    renderer.dispose();
    atlas.dispose().unwrap();
    canvas.remove();
}
