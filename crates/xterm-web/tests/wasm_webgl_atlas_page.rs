// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
#![cfg(target_arch = "wasm32")]
use serde_json::Value;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_web::webgl::{
    atlas_page::{AtlasRow, CanvasAtlasPage, PageClock, StoredGlyph},
    atlas_pixels::GlyphBounds,
};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(inline_js = "
export function missingPageContext() {
 return {createElement(tag) {
  const canvas = document.createElement(tag);
  canvas.getContext = () => null;
  return canvas;
 }};
}
")]
extern "C" {
    #[wasm_bindgen(js_name = missingPageContext)]
    fn missing_page_context() -> JsValue;
}

#[wasm_bindgen_test]
fn missing_context_consumes_the_source_version_before_failing() {
    let clock = PageClock::default();
    let fake_document = missing_page_context().unchecked_into();
    let result = CanvasAtlasPage::new(&fake_document, 16, &clock, &[]);
    assert!(result.is_err());
    let error = js_sys::Reflect::get(&result.err().unwrap(), &"message".into()).unwrap();
    assert_eq!(error.as_string().unwrap(), "value must not be falsy");
    let doc = web_sys::window().unwrap().document().unwrap();
    let page = CanvasAtlasPage::new(&doc, 16, &clock, &[]).unwrap();
    assert_eq!(page.texture().version, 2);
}

fn assert_page(
    page: &CanvasAtlasPage,
    row: &xterm_web::webgl::atlas_page::RowRef,
    glyphs: &[xterm_web::webgl::atlas_page::GlyphRef],
    expected: &Value,
) {
    let current = page.current_row();
    let state = *current.borrow();
    let indices: Vec<_> = page
        .glyphs
        .borrow()
        .iter()
        .map(|g| {
            glyphs
                .iter()
                .position(|candidate| Rc::ptr_eq(g, candidate))
                .unwrap()
        })
        .collect();
    assert_eq!(
        page.texture().version,
        expected["version"].as_u64().unwrap()
    );
    assert_eq!(
        page.percentage_used(),
        expected["percentageUsed"].as_f64().unwrap()
    );
    assert_eq!(serde_json::to_value(indices).unwrap(), expected["glyphs"]);
    assert_eq!(state.x, expected["row"]["x"].as_f64().unwrap());
    assert_eq!(state.y, expected["row"]["y"].as_f64().unwrap());
    assert_eq!(state.height, expected["row"]["height"].as_f64().unwrap());
    assert_eq!(
        Rc::ptr_eq(row, &current),
        expected["sameRow"].as_bool().unwrap()
    );
    assert_eq!(
        page.fixed.borrow().len() as u64,
        expected["fixedCount"].as_u64().unwrap()
    );
}

#[wasm_bindgen_test]
fn quad_and_fallback_page_metadata_match_unchanged_source() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/webgl-atlas-pages.json")).unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 5);
    let doc = web_sys::window().unwrap().document().unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let clock = PageClock::default();
        let pages: Vec<_> = (0..4)
            .map(|_| CanvasAtlasPage::new(&doc, 16, &clock, &[]).unwrap())
            .collect();
        let glyphs: Vec<_> = (0..4)
            .map(|i| {
                Rc::new(RefCell::new(StoredGlyph::new(GlyphBounds {
                    top: 0,
                    left: 0,
                    right: i,
                    bottom: i + 1,
                    size: [(i + 1) as f64, (i + 2) as f64],
                    offset: [0.0, 0.0],
                })))
            })
            .collect();
        for (page, glyph) in pages.iter().zip(&glyphs) {
            page.add_glyph(glyph.clone()).unwrap();
        }
        let sources: Vec<_> = case["indices"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| pages[i.as_u64().unwrap() as usize].clone())
            .collect();
        let merged = CanvasAtlasPage::new(&doc, 32, &clock, &sources).unwrap();
        let row = merged.current_row();
        *row.borrow_mut() = AtlasRow {
            x: 9.0,
            y: 8.0,
            height: 7.0,
        };
        merged.fixed.borrow_mut().push(row.clone());
        assert_page(&merged, &row, &glyphs, &case["before"]);
        merged.clear(&clock).unwrap();
        assert_page(&merged, &row, &glyphs, &case["after"]);
        pages[0].clear(&clock).unwrap();
        assert_eq!(
            pages[0].texture().version,
            case["sourceVersionAfterClear"].as_u64().unwrap()
        );
    }
}
#[wasm_bindgen_test]
fn page_clear_preserves_historical_glyph_aliases_and_used_pixels() {
    let doc = web_sys::window().unwrap().document().unwrap();
    let clock = PageClock::default();
    let page = CanvasAtlasPage::new(&doc, 16, &clock, &[]).unwrap();
    let glyph = Rc::new(RefCell::new(StoredGlyph::new(GlyphBounds {
        top: 0,
        left: 0,
        right: 2,
        bottom: 3,
        size: [3.0, 4.0],
        offset: [0.0, 0.0],
    })));
    glyph.borrow_mut().place(0, [2.0, 3.0], 16.0, 16.0);
    page.add_glyph(glyph.clone()).unwrap();
    let row = page.current_row();
    *row.borrow_mut() = AtlasRow {
        x: 5.0,
        y: 3.0,
        height: 4.0,
    };
    page.fixed.borrow_mut().push(row.clone());
    page.context.set_fill_style_str("red");
    page.context.fill_rect(0.0, 0.0, 16.0, 16.0);
    assert_eq!(page.texture().version, 1);
    assert_eq!(page.percentage_used(), 12.0 / 256.0);
    page.clear(&clock).unwrap();
    assert!(Rc::ptr_eq(&row, &page.current_row()));
    assert_eq!(*row.borrow(), AtlasRow::default());
    assert!(page.fixed.borrow().is_empty());
    assert_eq!(page.texture().version, 2);
    assert_eq!(page.percentage_used(), 12.0 / 256.0);
    assert_eq!(page.glyphs.borrow().len(), 1);
    assert!(Rc::ptr_eq(&page.glyphs.borrow()[0], &glyph));
    assert!(page
        .context
        .get_image_data(0.0, 0.0, 16.0, 16.0)
        .unwrap()
        .data()
        .0
        .iter()
        .all(|&v| v == 0));
}
#[wasm_bindgen_test]
fn adopts_the_supplied_document_realm() {
    let doc = web_sys::window().unwrap().document().unwrap();
    let iframe = doc
        .create_element("iframe")
        .unwrap()
        .dyn_into::<web_sys::HtmlIFrameElement>()
        .unwrap();
    doc.body().unwrap().append_child(&iframe).unwrap();
    let child = iframe.content_document().unwrap();
    let result = CanvasAtlasPage::new(&child, 16, &PageClock::default(), &[]);
    let pixels = match &result {
        Ok(page) => {
            page.context.set_fill_style_str("red");
            page.context.fill_rect(0.0, 0.0, 1.0, 1.0);
            page.context
                .get_image_data(0.0, 0.0, 1.0, 1.0)
                .map(|image| image.data().0)
        }
        Err(error) => Err(error.clone()),
    };
    // Chromium stops painting when the iframe is removed. Read first, then
    // clean up before assertions so both failure paths release the test frame.
    iframe.remove();
    let page = result.unwrap();
    assert_eq!(page.canvas.owner_document().unwrap(), child);
    assert_eq!(pixels.unwrap(), [255, 0, 0, 255]);
}
#[wasm_bindgen_test]
fn merge_constructor_shares_glyphs_and_clock_but_does_not_copy_pixels() {
    let doc = web_sys::window().unwrap().document().unwrap();
    let clock = PageClock::default();
    let page = CanvasAtlasPage::new(&doc, 16, &clock, &[]).unwrap();
    let glyph = Rc::new(RefCell::new(StoredGlyph::new(GlyphBounds {
        top: 0,
        left: 0,
        right: 2,
        bottom: 3,
        size: [3.0, 4.0],
        offset: [0.0, 0.0],
    })));
    page.add_glyph(glyph.clone()).unwrap();
    page.context.set_fill_style_str("lime");
    page.context.fill_rect(0.0, 0.0, 16.0, 16.0);
    let merged = CanvasAtlasPage::new(&doc, 32, &clock, &[page.clone()]).unwrap();
    assert_eq!(merged.texture().version, 2);
    assert_eq!(merged.percentage_used(), 12.0 / 1024.0);
    assert!(Rc::ptr_eq(&merged.glyphs.borrow()[0], &glyph));
    assert!(merged
        .context
        .get_image_data(0.0, 0.0, 32.0, 32.0)
        .unwrap()
        .data()
        .0
        .iter()
        .all(|&v| v == 0));
    page.touch(&clock).unwrap();
    assert_eq!(page.texture().version, 3);
    assert_eq!(merged.texture().version, 2);
}
