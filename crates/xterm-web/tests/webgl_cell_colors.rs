// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use serde_json::{json, Value};
use xterm_core::{
    buffer::{AttributeData, ExtendedAttrs},
    color::channels,
};
use xterm_web::{
    renderer::row_types::RowDecoration,
    theme::{ColorRef, ThemeService},
    webgl::cell_colors::{CellColorContext, CellColorResolver},
};

fn color(value: u32) -> ColorRef {
    channels::to_color(
        (value >> 24).into(),
        ((value >> 16) & 255).into(),
        ((value >> 8) & 255).into(),
        Some((value & 255).into()),
    )
    .into()
}
fn word(value: &Value) -> u32 {
    u32::try_from(value.as_u64().unwrap()).unwrap()
}
fn number(case: &Value, key: &str) -> f64 {
    match case[key].as_str() {
        Some("NaN") => f64::NAN,
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => case[key].as_f64().unwrap(),
    }
}

#[test]
fn unchanged_source_cell_colors_and_variants() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/webgl-cell-colors.json")).unwrap();
    assert_eq!(fixture["seed"], "0x7a12c431");
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2956);
    let theme = ThemeService::default();
    let mut colors = theme.colors().borrow().clone();
    colors.foreground = color(word(&fixture["foreground"]));
    colors.background = color(word(&fixture["background"]));
    colors.selection_background_opaque = color(word(&fixture["selection"]));
    colors.selection_inactive_background_opaque = color(word(&fixture["inactive"]));
    let palette: Vec<_> = fixture["palette"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| color(word(v)))
        .collect();
    let mut resolver = CellColorResolver::default();
    let mut errors = 0;
    for case in cases {
        let name = case["name"].as_str().unwrap();
        colors.ansi = palette[..case["paletteLength"].as_u64().unwrap() as usize].to_vec();
        colors.selection_foreground = case["selectionForeground"]
            .as_u64()
            .map(|v| color(u32::try_from(v).unwrap()));
        let cell = AttributeData {
            bg: word(&case["bg"]),
            fg: word(&case["fg"]),
            extended: ExtendedAttrs::new(word(&case["ext"]), case["url"].as_u64().unwrap()),
        };
        let context = CellColorContext {
            x: number(case, "x"),
            y: number(case, "y"),
            device_cell_width: number(case, "cw"),
            device_cell_height: number(case, "ch"),
            font_size: number(case, "font"),
            device_pixel_ratio: number(case, "dpr"),
            selected: case["selected"].as_bool().unwrap(),
            focused: case["focused"].as_bool().unwrap(),
        };
        let decorations: Vec<_> = case["decorations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| RowDecoration {
                background: d["bg"]
                    .as_u64()
                    .map(|v| color(u32::try_from(v).unwrap()).borrow().clone()),
                foreground: d["fg"]
                    .as_u64()
                    .map(|v| color(u32::try_from(v).unwrap()).borrow().clone()),
                top: d["top"].as_bool().unwrap(),
            })
            .collect();
        for _ in 0..2 {
            let result =
                resolver.resolve(&cell, word(&case["code"]), context, &colors, &decorations);
            if case["error"].is_null() {
                result.unwrap_or_else(|error| panic!("{name}: {error}"));
            } else {
                assert_eq!(case["error"], "TypeError", "{name}");
                assert!(result.is_err(), "{name}: source throws for missing palette");
                errors += 1;
            }
            assert_eq!(
                json!(resolver.result_numbers()),
                case["numbers"],
                "{name} numeric representation"
            );
            assert_eq!(
                json!({"bg":resolver.result.bg,"fg":resolver.result.fg,"ext":resolver.result.ext}),
                case["expected"],
                "{name}"
            );
        }
    }
    assert!(errors > 0);
}

#[test]
fn missing_palette_reports_index_retains_base_and_recovers() {
    use xterm_core::buffer::constants::{attributes as a, bg_flags, fg_flags};
    let theme = ThemeService::default();
    let mut colors = theme.colors().borrow().clone();
    colors.ansi.clear();
    let cell = AttributeData {
        bg: bg_flags::HAS_EXTENDED | a::CM_P256 | 255,
        fg: fg_flags::BOLD,
        extended: ExtendedAttrs::new(0xf000_0000, 0),
    };
    let context = CellColorContext {
        x: 1.0,
        y: 1.0,
        device_cell_width: 9.0,
        device_cell_height: 17.0,
        font_size: 15.0,
        device_pixel_ratio: 1.0,
        selected: true,
        focused: true,
    };
    let mut resolver = CellColorResolver::default();
    let error = resolver
        .resolve(&cell, 65, context, &colors, &[])
        .unwrap_err();
    assert_eq!(error.index, 255);
    assert_eq!(resolver.result.bg, cell.bg);
    assert_eq!(resolver.result.fg, cell.fg);
    assert_eq!(resolver.result.ext, cell.extended.ext());
    colors.ansi = vec![color(0x112233ff); 256];
    resolver.resolve(&cell, 65, context, &colors, &[]).unwrap();
    assert_ne!(resolver.result.bg, cell.bg);
    assert_eq!(resolver.result.ext, 0x3000_0000);
    colors.selection_background_opaque = color(0x776655ff);
    let first = resolver.result;
    resolver.resolve(&cell, 65, context, &colors, &[]).unwrap();
    assert_ne!(resolver.result.bg, first.bg);
}
