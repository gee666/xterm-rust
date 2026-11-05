// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
use serde_json::Value;
use xterm_web::webgl::{
    glyphs::{GlyphColorNumbers, GlyphGeometry},
    renderer_types::GlyphColorNumbers as RendererColorNumbers,
};
#[path = "support/webgl_glyph_fixture.rs"]
mod fixture;

#[test]
fn signed_backgrounds_match_unchanged_source_vertex_bits() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/webgl-glyph-backgrounds.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 8);
    for case in cases {
        let mut geometry =
            GlyphGeometry::new(2, 1, fixture::dimensions(&case["dimensions"])).unwrap();
        for (step, c) in case["updates"].as_array().unwrap().iter().enumerate() {
            let chars = fixture::chars(c);
            let mut packed = c.clone();
            packed["bg"] = Value::from(c["bg"].as_i64().unwrap() as u32);
            packed["lastBg"] = Value::from(c["lastBg"].as_i64().unwrap() as u32);
            let cell = fixture::cell(&packed, &chars);
            // The source clips all four unequal signed/unsigned pairs even
            // though their packed words are equal.
            assert_eq!(cell.bg, cell.last_bg);
            let numbers: RendererColorNumbers = GlyphColorNumbers {
                current: [
                    c["bg"].as_i64().unwrap(),
                    c["fg"].as_i64().unwrap(),
                    c["ext"].as_i64().unwrap(),
                ],
                previous_background: c["lastBg"].as_i64().unwrap(),
            };
            geometry
                .update_cell_with_colors(
                    cell,
                    numbers,
                    Some(&fixture::glyph(&c["glyph"])),
                    64.0,
                    false,
                )
                .unwrap();
            assert_eq!(
                fixture::bits(geometry.attributes()),
                fixture::expected(&case["snapshots"][step]),
                "{} step{step}",
                case["name"],
            );
        }
        assert_eq!(
            fixture::bits(geometry.compact(&[2]).unwrap()),
            fixture::expected(&case["upload"]),
            "{} upload",
            case["name"],
        );
    }
}
