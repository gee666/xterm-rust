// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
#![cfg(target_arch = "wasm32")]
#[path = "support/canvas_layer.rs"]
mod support;
use serde_json::Value;
use std::{cell::Cell, rc::Rc};
use support::{dim, event, live, note, snapshot, Capture, Harness};
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_test::*;
use xterm_core::buffer::{CellData, CharData};
use xterm_web::webgl::{canvas_layer::CanvasLayer, link_layer::LinkRenderLayer};
wasm_bindgen_test_configure!(run_in_browser);

fn run(name: &str) {
    let expected: Value = serde_json::from_str(&support::source(name)).unwrap();
    let capture = Capture::new();
    let h = Harness::new();
    if name == "constructor_error" {
        support::fail("context");
        let error = CanvasLayer::new(h.services.clone(), h.container.clone(), "test", 3, true)
            .err()
            .unwrap();
        note(if error == support::failure() {
            "same"
        } else {
            "different"
        });
        support::fail("null");
        let error = CanvasLayer::new(h.services.clone(), h.container.clone(), "test", 3, true)
            .err()
            .unwrap();
        note(
            &js_sys::Reflect::get(&error, &"message".into())
                .unwrap()
                .as_string()
                .unwrap(),
        );
        h.children();
    } else if name == "base" || name == "transparency_error" || name == "font_reentry" {
        let base = h.base(false);
        base.resize(dim()).unwrap();
        support::scheduled();
        if name == "font_reentry" {
            let resized = base.clone();
            let callback = Closure::wrap(Box::new(move || {
                let mut d = dim();
                d.device.cell.width = 12.0;
                d.device.cell.height = 22.0;
                d.device.character.left = 2.0;
                d.device.character.top = 3.0;
                d.device.canvas.width = 96.0;
                d.device.canvas.height = 88.0;
                d.css.canvas.width = 64.0;
                d.css.canvas.height = 88.0 / 1.5;
                resized.resize(d).unwrap();
            }) as Box<dyn Fn()>);
            support::reenter("beginPath", callback.as_ref().unchecked_ref());
            let cell = CellData::from_char_data(&CharData {
                attr: 0,
                chars: vec![65],
                width: 1,
                code: Some(65),
            });
            base.fill_char_true_color(&cell, 1.0, 0.0).unwrap();
            snapshot(&base.canvas());
        } else if name == "base" {
            base.clear_cells(1.0, 1.0, 2.0, 1.0).unwrap();
            base.fill_style(&"#00ff00".into()).unwrap();
            let cell = CellData::from_char_data(&CharData {
                attr: 0,
                chars: "界".encode_utf16().collect(),
                width: 2,
                code: Some(30028),
            });
            base.fill_char_true_color(&cell, 1.0, 0.0).unwrap();
            snapshot(&base.canvas());
            note(&base.font(true, true).unwrap().as_string().unwrap());
            base.set_transparency(true, support::redraw).unwrap();
            snapshot(&base.canvas());
            base.set_transparency(true, support::redraw).unwrap();
            base.set_transparency(false, support::redraw).unwrap();
            snapshot(&base.canvas());
            h.services
                .theme
                .set_theme(&Harness::theme("#ff0000", "#112233"))
                .unwrap();
            snapshot(&base.canvas());
        } else {
            support::fail("context");
            let error = base.set_transparency(true, support::redraw).unwrap_err();
            note(if error == support::failure() {
                "same"
            } else {
                "different"
            });
            h.children();
            base.set_transparency(true, support::redraw).unwrap();
        }
        base.dispose().unwrap();
        base.dispose().unwrap();
        h.children();
    } else {
        let layer = LinkRenderLayer::new(h.services.clone(), h.container.clone(), 2).unwrap();
        layer.resize(dim()).unwrap();
        support::scheduled();
        match name {
            "link" => {
                layer.show(live(2.0, 1.0, 6.0, 1.0)).unwrap();
                snapshot(&layer.base.canvas());
                layer.reset().unwrap();
                snapshot(&layer.base.canvas());
                layer.show(live(6.0, 0.0, 3.0, 3.0)).unwrap();
                snapshot(&layer.base.canvas());
                layer.reset().unwrap();
                layer.reset().unwrap();
                snapshot(&layer.base.canvas());
            }
            "colors" => {
                let mut e = event(2.0, 1.0, 6.0, 1.0);
                e.foreground = Some(257);
                layer.show(Rc::new(Cell::new(e))).unwrap();
                snapshot(&layer.base.canvas());
                layer.reset().unwrap();
                e.foreground = Some(1);
                layer.show(Rc::new(Cell::new(e))).unwrap();
                snapshot(&layer.base.canvas());
                e.foreground = Some(0x1000001);
                let error = layer.show(Rc::new(Cell::new(e))).unwrap_err();
                note(
                    &js_sys::Reflect::get(&error, &"name".into())
                        .unwrap()
                        .as_string()
                        .unwrap(),
                );
                layer.reset().unwrap();
                snapshot(&layer.base.canvas());
            }
            "retained" => {
                let e = live(2.0, 1.0, 6.0, 1.0);
                layer.show(e.clone()).unwrap();
                e.set(event(1.0, 0.0, 4.0, 3.0));
                layer.reset().unwrap();
                snapshot(&layer.base.canvas());
            }
            "theme" => {
                layer.show(live(6.0, 0.0, 3.0, 3.0)).unwrap();
                h.services
                    .theme
                    .set_theme(&Harness::theme("#ff0000", "#112233"))
                    .unwrap();
                snapshot(&layer.base.canvas());
                note(&format!("atlas:{}", layer.base.atlas().is_some()));
                support::scheduled();
                let atlas = layer.base.atlas().unwrap();
                atlas
                    .get_glyph(
                        xterm_web::webgl::texture_atlas::GlyphKey::Number(65),
                        0,
                        0,
                        0,
                        false,
                        None,
                    )
                    .unwrap();
                snapshot(&atlas.canvases()[0]);
                let sibling =
                    LinkRenderLayer::new(h.services.clone(), h.container.clone(), 2).unwrap();
                sibling.resize(dim()).unwrap();
                note(&format!(
                    "shared:{}",
                    Rc::ptr_eq(&atlas, &sibling.base.atlas().unwrap())
                ));
                support::scheduled();
                sibling.dispose().unwrap();
            }
            "dpr_reentry" => {
                let theme = h.services.theme.clone();
                let callback = Closure::wrap(Box::new(move || {
                    theme
                        .set_theme(&Harness::theme("#ff0000", "#112233"))
                        .unwrap();
                }) as Box<dyn Fn()>);
                support::reenter_dpr(callback.as_ref().unchecked_ref());
                layer.resize(dim()).unwrap();
                let atlas = layer.base.atlas().unwrap();
                atlas
                    .get_glyph(
                        xterm_web::webgl::texture_atlas::GlyphKey::Number(65),
                        0,
                        0,
                        0,
                        false,
                        None,
                    )
                    .unwrap();
                snapshot(&atlas.canvases()[0]);
                support::scheduled();
            }
            "resize" => {
                layer.show(live(2.0, 1.0, 6.0, 1.0)).unwrap();
                layer.resize(dim()).unwrap();
                layer.reset().unwrap();
                snapshot(&layer.base.canvas());
                let previous = layer.base.atlas().unwrap();
                let mut d = dim();
                d.device.character.width = 0.0;
                d.device.character.height = 0.0;
                layer.resize(d).unwrap();
                note(&format!(
                    "retained:{}",
                    Rc::ptr_eq(&previous, &layer.base.atlas().unwrap())
                ));
            }
            "draw_error" => {
                layer.show(live(2.0, 1.0, 6.0, 1.0)).unwrap();
                support::fail("clearRect");
                let error = layer.reset().unwrap_err();
                note(if error == support::failure() {
                    "same"
                } else {
                    "different"
                });
                layer.reset().unwrap();
                snapshot(&layer.base.canvas());
            }
            "reentry" => {
                let e = live(6.0, 0.0, 3.0, 3.0);
                let mutated = e.clone();
                let callback =
                    Closure::wrap(
                        Box::new(move || mutated.set(event(1.0, 1.0, 2.0, 3.0))) as Box<dyn Fn()>
                    );
                support::reenter("fillRect", callback.as_ref().unchecked_ref());
                layer.show(e).unwrap();
                layer.reset().unwrap();
                snapshot(&layer.base.canvas());
            }
            _ => panic!("unknown scenario"),
        }
        layer.dispose().unwrap();
        layer.dispose().unwrap();
        h.children();
    }
    // Cleanup the atlas while observation is active, matching source disposal.
    drop(h);
    let actual = capture.finish();
    console_log!("layer-source {name} {expected}");
    console_log!("layer-rust {name} {actual}");
    assert_eq!(actual, expected, "scenario {name}");
}
#[wasm_bindgen_test]
fn text_clip_reentry_uses_current_canvas_dimensions() {
    run("font_reentry");
}
#[wasm_bindgen_test]
fn dpr_reentry_observes_mutated_live_theme_before_config() {
    run("dpr_reentry");
}
#[wasm_bindgen_test]
fn actual_link_canvas_single_and_multiline() {
    run("link");
}
#[wasm_bindgen_test]
fn source_inverted_color_fallback_and_palette_error() {
    run("colors");
}
#[wasm_bindgen_test]
fn retained_event_mutation_controls_hide() {
    run("retained");
}
#[wasm_bindgen_test]
fn theme_refresh_warms_shared_atlas_before_link_reset() {
    run("theme");
}
#[wasm_bindgen_test]
fn resize_discards_state_and_keeps_atlas_for_zero_chars() {
    run("resize");
}
#[wasm_bindgen_test]
fn draw_error_preserves_identity_and_clear_state() {
    run("draw_error");
}
#[wasm_bindgen_test]
fn draw_reentry_reads_live_retained_event() {
    run("reentry");
}
#[wasm_bindgen_test]
fn actual_base_opaque_text_alpha_clone_theme_and_disposal() {
    run("base");
}
#[wasm_bindgen_test]
fn failed_alpha_context_preserves_source_partial_mutation() {
    run("transparency_error");
}
#[wasm_bindgen_test]
fn construction_errors_precede_canvas_attachment() {
    run("constructor_error");
}
