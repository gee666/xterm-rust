// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Shared decoding for native geometry and browser GPU source comparisons.
use serde_json::Value;
use xterm_web::{
    api::{CharacterDimensions, RenderDimensions, Size},
    webgl::glyphs::{GlyphCell, RasterizedGlyph},
};
fn number(v: &Value) -> f64 {
    v.as_f64().unwrap()
}
fn pair(v: &Value) -> [f64; 2] {
    [number(&v["x"]), number(&v["y"])]
}
pub fn dimensions(v: &Value) -> RenderDimensions {
    let v = &v["device"];
    let size = |key: &str| Size {
        width: number(&v[key]["width"]),
        height: number(&v[key]["height"]),
    };
    let mut d = RenderDimensions::default();
    d.device.cell = size("cell");
    d.device.canvas = size("canvas");
    d.device.character = CharacterDimensions {
        width: number(&v["char"]["width"]),
        height: number(&v["char"]["height"]),
        left: number(&v["char"]["left"]),
        top: number(&v["char"]["top"]),
    };
    d
}
pub fn glyph(v: &Value) -> RasterizedGlyph {
    RasterizedGlyph {
        offset: pair(&v["offset"]),
        size: pair(&v["size"]),
        texture_page: v["texturePage"].as_u64().unwrap() as usize,
        texture_position: pair(&v["texturePositionClipSpace"]),
        texture_size: pair(&v["sizeClipSpace"]),
    }
}
pub fn chars(c: &Value) -> Vec<u16> {
    c["chars"]
        .as_array()
        .unwrap()
        .iter()
        .map(|unit| u16::try_from(unit.as_u64().unwrap()).unwrap())
        .collect()
}
pub fn cell<'a>(c: &Value, chars: &'a [u16]) -> GlyphCell<'a> {
    GlyphCell {
        x: c["x"].as_u64().unwrap() as usize,
        y: c["y"].as_u64().unwrap() as usize,
        code: c["code"].as_i64(),
        bg: c["bg"].as_u64().unwrap() as u32,
        fg: c["fg"].as_u64().unwrap() as u32,
        ext: c["ext"].as_u64().unwrap() as u32,
        chars,
        width: number(&c["width"]),
        last_bg: c["lastBg"].as_u64().unwrap() as u32,
    }
}
pub fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|x| x.to_bits()).collect()
}
pub fn expected(v: &Value) -> Vec<u32> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_u64().unwrap() as u32)
        .collect()
}
