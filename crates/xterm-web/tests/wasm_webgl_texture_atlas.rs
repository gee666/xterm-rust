// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
#![cfg(target_arch = "wasm32")]
use serde_json::{json, Value};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_core::{
    color::channels, log_service::LogService, options::OptionsService,
    unicode_service::UnicodeService, unicode_v6::UnicodeV6,
};
use xterm_wasm_support::logger::console_sink;
use xterm_web::{
    theme::ThemeService,
    webgl::{
        atlas_page::GlyphRef,
        cache::AtlasResource,
        config::{AtlasDimensions, AtlasOptions, FontWeight},
        glyph_atlas::GlyphAtlas,
        texture_atlas::{AtlasRuntime, GlyphKey, TextureAtlas},
    },
};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(
    inline_js = "export function getSourceAtlasScenario(name){return globalThis.sourceAtlasScenario(name);} export function captureAtlasWarmup(f){return globalThis.captureAtlasCallbacks(f);}"
)]
extern "C" {
    #[wasm_bindgen(catch,js_name=getSourceAtlasScenario)]
    fn source_scenario(name: &str) -> Result<String, JsValue>;
    #[wasm_bindgen(catch, js_name=captureAtlasWarmup)]
    fn capture_warmup(invoke: &js_sys::Function) -> Result<u32, JsValue>;
}
fn config(spec: &Value, runtime: &AtlasRuntime) -> Rc<xterm_web::webgl::config::AtlasConfig> {
    let theme = ThemeService::default();
    let mut colors = theme.colors().borrow().clone();
    let color = |v: u32| {
        channels::to_color(
            (v >> 24).into(),
            ((v >> 16) & 255).into(),
            ((v >> 8) & 255).into(),
            Some((v & 255).into()),
        )
        .into()
    };
    colors.foreground = color(0xeeeeeeff);
    colors.background = color(0x102030ff);
    colors.ansi = (0..256).map(|i| color(((i * 65793) << 8) | 255)).collect();
    let options = AtlasOptions {
        letter_spacing: 0.0,
        line_height: spec["lineHeight"].as_f64().unwrap(),
        font_family: Rc::from(
            spec["fontFamily"]
                .as_str()
                .unwrap()
                .encode_utf16()
                .collect::<Vec<_>>(),
        ),
        font_size: spec["fontSize"].as_f64().unwrap(),
        font_weight: FontWeight::Number(400.0),
        font_weight_bold: FontWeight::Number(700.0),
        allow_transparency: spec["transparency"].as_bool().unwrap(),
        draw_bold_text_in_bright_colors: true,
        minimum_contrast_ratio: spec["ratio"].as_f64().unwrap(),
    };
    let dimensions = AtlasDimensions {
        device_pixel_ratio: spec["dpr"].as_f64().unwrap(),
        device_max_texture_size: spec["maxTexture"].as_f64().unwrap(),
        device_cell_width: spec["cellWidth"].as_f64().unwrap(),
        device_cell_height: spec["cellHeight"].as_f64().unwrap(),
        device_char_width: spec["charWidth"].as_f64().unwrap(),
        device_char_height: spec["charHeight"].as_f64().unwrap(),
    };
    Rc::new(runtime.config.generate(
        dimensions,
        &options,
        &colors,
        spec["custom"].as_bool().unwrap(),
    ))
}
fn glyph(glyph: &GlyphRef) -> Value {
    let g = glyph.borrow();
    let r = g.raster;
    json!({"page":r.texture_page,"position":g.position,"texture":r.texture_position,
        "size":r.size,"offset":r.offset,"textureSize":r.texture_size})
}
fn assert_same(actual: &Value, expected: &Value, path: &str) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => assert_eq!(a.as_f64(), b.as_f64(), "{path}"),
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{path}");
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                assert_same(a, b, &format!("{path}/{i}"));
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len(), "{path}");
            for (k, a) in a {
                assert_same(
                    a,
                    b.get(k).unwrap_or_else(|| panic!("Missing {path}/{k}")),
                    &format!("{path}/{k}"),
                );
            }
        }
        _ => assert_eq!(actual, expected, "{path}"),
    }
}
fn make_atlas(spec: &Value) -> Rc<TextureAtlas> {
    let runtime = Rc::new(AtlasRuntime::default());
    runtime
        .max_pages
        .set(Some(spec["maxPages"].as_u64().unwrap() as usize));
    runtime
        .max_texture_size
        .set(Some(spec["maxTexture"].as_u64().unwrap() as u32));
    let unicode = Rc::new(UnicodeService::new());
    unicode.register(Rc::new(UnicodeV6::new())).unwrap();
    let log = Rc::new(LogService::new(Rc::new(OptionsService::default()), console_sink()).unwrap());
    let doc = web_sys::window().unwrap().document().unwrap();
    TextureAtlas::new(&doc, config(spec, &runtime), runtime, unicode, log).unwrap()
}
fn scenario(name: &str) {
    let source: Value = serde_json::from_str(&source_scenario(name).unwrap()).unwrap();
    let spec = &source["spec"];
    let atlas = make_atlas(spec);
    let doc = web_sys::window().unwrap().document().unwrap();
    let element = doc.create_element("div").unwrap();
    doc.body().unwrap().append_child(&element).unwrap();
    let events = Rc::new(std::cell::RefCell::new(Vec::new()));
    let added = events.clone();
    atlas
        .on_add_canvas()
        .subscribe(move |c| added.borrow_mut().push(json!(["add", c.width()])));
    if spec["reenter"] == true {
        let weak = Rc::downgrade(&atlas);
        let cleared = std::cell::Cell::new(false);
        atlas.on_add_canvas().try_subscribe(move |_| {
            if !cleared.replace(true) {
                weak.upgrade().unwrap().clear_texture().map_err(|e| {
                    xterm_core::event::EmitError::callback(
                        xterm_wasm_support::js_error::JsCallbackError(e),
                    )
                })?;
            }
            Ok(())
        });
    }
    let removed = events.clone();
    atlas
        .on_remove_canvas()
        .subscribe(move |c| removed.borrow_mut().push(json!(["remove", c.width()])));
    let scheduled = if spec["warmup"] == true {
        let warm = atlas.clone();
        let invoke = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
            warm.warm_up()?;
            warm.warm_up()
        }) as Box<dyn Fn() -> Result<(), JsValue>>);
        json!(capture_warmup(invoke.as_ref().unchecked_ref()).unwrap())
    } else {
        Value::Null
    };
    let mut refs = Vec::new();
    let mut initial = Vec::new();
    let mut layouts = Vec::new();
    for step in spec["steps"].as_array().unwrap() {
        if step["clear"] == true {
            atlas.clear_texture().unwrap();
            layouts.push(atlas.page_layout_version());
            continue;
        }
        let key = if let Some(code) = step["value"].as_i64() {
            GlyphKey::Number(code as i32)
        } else {
            GlyphKey::Text(
                step["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|u| u.as_u64().unwrap() as u16)
                    .collect(),
            )
        };
        let g = atlas
            .get_glyph(
                key,
                step["bg"].as_u64().unwrap() as u32,
                step["fg"].as_u64().unwrap() as u32,
                step["ext"].as_u64().unwrap() as u32,
                step["restrict"].as_bool().unwrap(),
                Some(element.unchecked_ref()),
            )
            .unwrap();
        initial.push(glyph(&g));
        refs.push(g);
        layouts.push(atlas.page_layout_version());
    }
    let pages:Vec<_>=atlas.pages().iter().map(|p|{
        let data=p.context.get_image_data(0.0,0.0,p.canvas.width().into(),p.canvas.height().into()).unwrap().data().0;
        let mut hash=2166136261u32;let mut nonzero=0;
        for (i,byte)in data.iter().enumerate(){hash=(hash^u32::from(*byte)).wrapping_mul(16777619);if i%4==3&&*byte!=0{nonzero+=1;}}
        let row=*p.current_row().borrow();
        let fixed:Vec<_>=p.fixed.borrow().iter().map(|r|{let r=r.borrow();json!({"x":r.x,"y":r.y,"height":r.height})}).collect();
        json!({"width":p.canvas.width(),"height":p.canvas.height(),"version":p.texture().version,
            "row":{"x":row.x,"y":row.y,"height":row.height},"fixed":fixed,"used":p.percentage_used(),
            "glyphs":p.glyphs.borrow().len(),"hash":hash,"nonzero":nonzero})
    }).collect();
    let aliases: Vec<_> = refs
        .iter()
        .map(|g| refs.iter().position(|h| Rc::ptr_eq(g, h)).unwrap())
        .collect();
    let actual = json!({"initial":initial,"final":refs.iter().map(glyph).collect::<Vec<_>>(),"layouts":layouts,
        "events":*events.borrow(),"pages":pages,"aliases":aliases,"scheduled":scheduled,"attached":element.query_selector("canvas").unwrap().is_some()});
    atlas.dispose().unwrap();
    element.remove();
    assert_same(&actual, &source["expected"], name);
}
#[wasm_bindgen_test]
fn text_styles_and_custom_glyphs() {
    scenario("styles");
}
#[wasm_bindgen_test]
fn high_dpi_and_line_height() {
    scenario("styles2");
}
#[wasm_bindgen_test]
fn transparent_styles() {
    scenario("stylesTransparent");
}
#[wasm_bindgen_test]
fn disabled_custom_glyphs() {
    scenario("noCustom");
}
#[wasm_bindgen_test]
fn contrast_and_dim() {
    scenario("contrast");
}
#[wasm_bindgen_test]
fn merge_updates_cached_aliases() {
    scenario("merge");
}
#[wasm_bindgen_test]
fn capacity_eviction_and_page_events() {
    scenario("evict");
}
#[wasm_bindgen_test]
fn oversized_ligatures_use_overflow_page() {
    scenario("overflow");
}
#[wasm_bindgen_test]
fn clear_retains_metadata_and_invalidates_layout() {
    scenario("clear");
}
#[wasm_bindgen_test]
fn warmup_is_idle_cached_and_only_scheduled_once() {
    scenario("warmup");
}
#[wasm_bindgen_test]
fn page_added_listener_can_clear_live_atlas() {
    scenario("reentry");
}
#[path = "support/texture_atlas_gpu.rs"]
mod gpu_case;
#[path = "support/texture_atlas_host_cases.rs"]
mod host_cases;
#[wasm_bindgen_test]
fn seeded_mixed_styles_pack_like_source() {
    scenario("seeded");
}
