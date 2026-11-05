// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
#![cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use xterm_web::webgl::{
    atlas_page::{AtlasRow, CanvasAtlasPage, PageClock, StoredGlyph},
    atlas_pixels::GlyphBounds,
};
wasm_bindgen_test_configure!(run_in_browser);
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
