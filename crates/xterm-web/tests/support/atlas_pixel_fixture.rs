// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use serde_json::Value;
use xterm_web::{
    api::Size,
    webgl::atlas_pixels::{GlyphBounds, GlyphScan},
};
pub fn scan(v: &Value) -> GlyphScan {
    let size = |key: &str| Size {
        width: v[key][0].as_f64().unwrap(),
        height: v[key][1].as_f64().unwrap(),
    };
    GlyphScan {
        allowed_width: v["allowedWidth"].as_u64().unwrap() as usize,
        padding: v["padding"].as_u64().unwrap() as usize,
        restricted: v["restricted"].as_bool().unwrap(),
        custom: v["custom"].as_bool().unwrap(),
        cell: size("cell"),
        character: size("character"),
        line_height: v["lineHeight"].as_f64().unwrap(),
    }
}
pub fn pixels(v: &Value) -> Vec<u8> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|n| u8::try_from(n.as_u64().unwrap()).unwrap())
        .collect()
}
pub fn assert_bounds(actual: GlyphBounds, expected: &Value, name: &str) {
    for (key, value) in [
        ("top", actual.top),
        ("left", actual.left),
        ("bottom", actual.bottom),
        ("right", actual.right),
    ] {
        assert_eq!(
            value,
            expected["box"][key].as_u64().unwrap() as usize,
            "{name} {key}"
        );
    }
    for (key, values) in [("size", actual.size), ("offset", actual.offset)] {
        for (coordinate, value) in ["x", "y"].into_iter().zip(values) {
            assert_eq!(
                value.to_bits(),
                expected["glyph"][key][coordinate]
                    .as_f64()
                    .unwrap()
                    .to_bits(),
                "{name} {key}.{coordinate}"
            );
        }
    }
}
