// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
#[path = "webgl_glyph_fixture.rs"]
mod fixture;
use super::helpers::{canvas, context, model};
use serde_json::{json, Value};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use web_sys::WebGl2RenderingContext as Gl;
use xterm_web::webgl::{
    glyph_atlas::{AtlasPage, GlyphAtlas},
    glyph_gpu::GlyphRenderer,
    glyphs::{GlyphCell, RasterizedGlyph},
};

struct FixtureAtlas {
    pages: Vec<AtlasPage>,
    layout: Cell<u64>,
    glyph: RefCell<RasterizedGlyph>,
    calls: RefCell<Vec<Value>>,
}
impl GlyphAtlas for FixtureAtlas {
    fn page_layout_version(&self) -> u64 {
        self.layout.get()
    }
    fn page_count(&self) -> usize {
        self.pages.len()
    }
    fn page(&self, index: usize) -> Option<AtlasPage> {
        self.pages.get(index).cloned()
    }
    fn rasterize(
        &self,
        c: GlyphCell<'_>,
        combined: bool,
        _element: Option<&web_sys::HtmlElement>,
    ) -> Result<RasterizedGlyph, JsValue> {
        let first = if combined {
            json!(c.chars)
        } else {
            json!(c.code.unwrap())
        };
        self.calls.borrow_mut().push(json!([
            if combined { "combined" } else { "single" },
            first,
            c.bg,
            c.fg,
            c.ext,
            false
        ]));
        Ok(*self.glyph.borrow())
    }
}
#[wasm_bindgen_test]
fn unchanged_source_gpu_upload_bits_frames_and_atlas_calls() {
    let fixture: Value =
        serde_json::from_str(include_str!("../fixtures/webgl-glyphs.json")).unwrap();
    let (_canvas, gl) = context();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 147);
    for c in cases {
        let name = c["name"].as_str().unwrap();
        let mut r = GlyphRenderer::new(
            gl.clone(),
            c["cols"].as_u64().unwrap() as usize,
            c["rows"].as_u64().unwrap() as usize,
            fixture::dimensions(&c["dimensions"]),
            None,
        )
        .unwrap();
        let atlas = Rc::new(FixtureAtlas {
            pages: vec![
                AtlasPage {
                    canvas: canvas(64, 64),
                    version: 1,
                },
                AtlasPage {
                    canvas: canvas(128, 64),
                    version: 2,
                },
            ],
            layout: Cell::new(0),
            glyph: RefCell::new(RasterizedGlyph::default()),
            calls: RefCell::new(Vec::new()),
        });
        let installed = c["installed"].as_bool().unwrap();
        if installed {
            r.set_atlas(atlas.clone());
        }
        let mut frames = vec![r.begin_frame(), r.begin_frame()];
        atlas.layout.set(1);
        frames.extend([r.begin_frame(), r.begin_frame()]);
        assert_eq!(json!(frames), c["frames"], "{name} frames");
        for update in c["updates"].as_array().unwrap() {
            *atlas.glyph.borrow_mut() = fixture::glyph(&update["glyph"]);
            let chars = fixture::chars(update);
            r.update_cell(
                fixture::cell(update, &chars),
                c["rescale"].as_bool().unwrap(),
            )
            .unwrap();
        }
        if let Some(resize) = c["resize"].as_array() {
            r.handle_resize(
                resize[0].as_u64().unwrap() as usize,
                resize[1].as_u64().unwrap() as usize,
            )
            .unwrap();
        }
        let lengths: Vec<u32> = c["lineLengths"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_u64().unwrap() as u32)
            .collect();
        r.render(&model(&lengths)).unwrap();
        assert_eq!(
            json!(*atlas.calls.borrow()),
            c["calls"],
            "{name} lookup calls"
        );
        if installed {
            let expected = fixture::expected(&c["upload"]);
            let result = js_sys::Float32Array::new_with_length(expected.len() as u32);
            // Read the actual GPU buffer, not another copy of portable geometry.
            gl.get_buffer_sub_data_with_i32_and_array_buffer_view(Gl::ARRAY_BUFFER, 0, &result);
            assert_eq!(
                fixture::bits(&result.to_vec()),
                expected,
                "{name} GPU upload"
            );
            assert_eq!(gl.get_error(), Gl::NO_ERROR, "{name} GPU error");
        }
    }
}
