// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use serde_json::{json, Value};
use std::rc::Rc;
use xterm_core::{buffer::AttributeData, color::channels};
use xterm_web::{
    api::Size,
    theme::{ColorRef, ThemeService},
    webgl::{
        atlas_colors::AtlasColors,
        config::{AtlasDimensions, AtlasOptions, ConfigFactory, FontWeight},
    },
};
fn color(v: u32) -> ColorRef {
    channels::to_color(
        (v >> 24).into(),
        ((v >> 16) & 255).into(),
        ((v >> 8) & 255).into(),
        Some((v & 255).into()),
    )
    .into()
}
fn snapshot(v: &ColorRef) -> Value {
    let v = v.borrow();
    json!({"rgba":v.rgba,"css":v.css})
}
#[test]
fn unchanged_source_colors_identity_and_contrast_cache() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/webgl-atlas-colors.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 476);
    for c in cases {
        let name = c["name"].as_str().unwrap();
        let theme = ThemeService::default();
        let mut colors = theme.colors().borrow().clone();
        colors.foreground = color(fixture["foreground"].as_u64().unwrap() as u32);
        colors.background = color(fixture["background"].as_u64().unwrap() as u32);
        colors.ansi = fixture["palette"]
            .as_array()
            .unwrap()
            .iter()
            .take(c["paletteLength"].as_u64().unwrap() as usize)
            .map(|v| color(v.as_u64().unwrap() as u32))
            .collect();
        let options = AtlasOptions {
            letter_spacing: 0.0,
            line_height: 1.0,
            font_family: Rc::from("monospace".encode_utf16().collect::<Vec<_>>()),
            font_size: 15.0,
            font_weight: FontWeight::Number(400.0),
            font_weight_bold: FontWeight::Number(700.0),
            allow_transparency: c["allowTransparency"].as_bool().unwrap(),
            draw_bold_text_in_bright_colors: c["bright"].as_bool().unwrap(),
            minimum_contrast_ratio: c["ratio"].as_f64().unwrap(),
        };
        let dimensions = AtlasDimensions {
            device_pixel_ratio: 1.0,
            device_max_texture_size: 4096.0,
            device_cell_width: 8.0,
            device_cell_height: 16.0,
            device_char_width: 8.0,
            device_char_height: 16.0,
        };
        let factory = ConfigFactory::new();
        let config = Rc::new(factory.generate(dimensions, &options, &colors, true));
        let attrs = AttributeData {
            bg: c["bg"].as_u64().unwrap() as u32,
            fg: c["fg"].as_u64().unwrap() as u32,
            ..Default::default()
        };
        let cache = if attrs.is_dim() != 0 {
            &config.colors.half_contrast_cache
        } else {
            &config.colors.contrast_cache
        };
        let numbers = c["original"]
            .as_array()
            .map_or([i64::from(attrs.bg), i64::from(attrs.fg)], |v| {
                [v[0].as_i64().unwrap(), v[1].as_i64().unwrap()]
            });
        let seeded = c["seedColor"].as_u64().map(|rgba| color(rgba as u32));
        if c["cacheNull"].as_bool().unwrap() {
            cache
                .borrow_mut()
                .set_color(numbers[0] as f64, numbers[1] as f64, None);
        } else if let Some(color) = &seeded {
            cache
                .borrow_mut()
                .set_color(numbers[0] as f64, numbers[1] as f64, Some(color.clone()));
        }
        let resolver = AtlasColors::new(config.clone(), &factory);
        let result =
            resolver.resolve_with_colors(&attrs, numbers, c["excluded"].as_bool().unwrap());
        if let Some(error) = c["error"].as_str() {
            assert_eq!(result.err().unwrap().to_string(), error, "{name}");
            continue;
        }
        let first = result.unwrap();
        if let Some(color) = seeded {
            assert!(
                first.foreground.same_object(&color),
                "seeded color identity: {name}"
            );
        }
        let second = resolver
            .resolve_with_colors(&attrs, numbers, c["excluded"].as_bool().unwrap())
            .unwrap();
        let cached = cache
            .borrow()
            .get_color(numbers[0] as f64, numbers[1] as f64)
            .cloned();
        let cache_value = match cached {
            None => json!("missing"),
            Some(None) => json!("null"),
            Some(Some(v)) => snapshot(&v),
        };
        assert_eq!(
            json!({"background":snapshot(&first.background),"foreground":snapshot(&first.foreground),
            "sameBackground":first.background.same_object(&second.background),"sameForeground":first.foreground.same_object(&second.foreground),
            "cache":cache_value}),
            c["expected"],
            "{name}"
        );
    }
}
#[test]
fn source_palette_errors_are_explicit() {
    // Keep this independent from the generated corpus so malformed native input
    // cannot turn into an unchecked negative array index.
    let theme = ThemeService::default();
    let factory = ConfigFactory::new();
    let options = AtlasOptions {
        letter_spacing: 0.0,
        line_height: 1.0,
        font_family: Rc::from(Vec::new()),
        font_size: 15.0,
        font_weight: FontWeight::Number(400.0),
        font_weight_bold: FontWeight::Number(700.0),
        allow_transparency: false,
        draw_bold_text_in_bright_colors: true,
        minimum_contrast_ratio: 1.0,
    };
    let size = Size {
        width: 8.0,
        height: 16.0,
    };
    let cfg = factory.generate(
        AtlasDimensions {
            device_pixel_ratio: 1.0,
            device_max_texture_size: 4096.0,
            device_cell_width: size.width,
            device_cell_height: size.height,
            device_char_width: size.width,
            device_char_height: size.height,
        },
        &options,
        &theme.colors().borrow(),
        true,
    );
    let resolver = AtlasColors::new(Rc::new(cfg), &factory);
    assert_eq!(
        resolver.palette(-1).err().unwrap().to_string(),
        "No color found for idx -1"
    );
    assert_eq!(
        resolver.palette(256).err().unwrap().to_string(),
        "No color found for idx 256"
    );
}
