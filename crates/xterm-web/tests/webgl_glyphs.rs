// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use serde_json::Value;
use xterm_web::{
    api::{RenderDimensions, Size},
    webgl::glyphs::{GlyphCell, GlyphGeometry, RasterizedGlyph},
};
#[path = "support/webgl_glyph_fixture.rs"]
mod fixture;
use fixture::{bits, dimensions, expected};
#[test]
fn unchanged_source_vertex_and_compaction_bits() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/webgl-glyphs.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 147);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let cols = case["cols"].as_u64().unwrap() as usize;
        let rows = case["rows"].as_u64().unwrap() as usize;
        let mut geometry = GlyphGeometry::new(cols, rows, dimensions(&case["dimensions"])).unwrap();
        for (step, c) in case["updates"].as_array().unwrap().iter().enumerate() {
            let glyph = fixture::glyph(&c["glyph"]);
            let chars = fixture::chars(c);
            let cell = fixture::cell(c, &chars);
            geometry
                .update_cell(
                    cell,
                    case["installed"].as_bool().unwrap().then_some(&glyph),
                    if glyph.texture_page == 0 { 64.0 } else { 128.0 },
                    case["rescale"].as_bool().unwrap(),
                )
                .unwrap();
            assert_eq!(
                bits(geometry.attributes()),
                expected(&case["snapshots"][step]),
                "{name} step{step}"
            );
        }
        if let Some(resize) = case["resize"].as_array() {
            geometry
                .resize(
                    resize[0].as_u64().unwrap() as usize,
                    resize[1].as_u64().unwrap() as usize,
                )
                .unwrap();
        }
        assert_eq!(
            bits(geometry.attributes()),
            expected(&case["attributes"]),
            "{name} attributes"
        );
        let lengths: Vec<u32> = case["lineLengths"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u32)
            .collect();
        if case["installed"].as_bool().unwrap() {
            for _ in 0..3 {
                let compact = geometry.compact(&lengths).unwrap();
                assert_eq!(bits(compact), expected(&case["upload"]), "{name} compact");
                assert_eq!(compact.len() / 11, case["count"].as_u64().unwrap() as usize);
            }
        }
        geometry.clear();
        assert_eq!(
            bits(geometry.attributes()),
            expected(&case["cleared"]),
            "{name} clear"
        );
    }
}
#[test]
fn invalid_inputs_fail_without_losing_existing_geometry() {
    let mut g = GlyphGeometry::new(2, 2, RenderDimensions::default()).unwrap();
    let before = bits(g.attributes());
    assert!(g.resize(usize::MAX, 2).is_err());
    assert_eq!(bits(g.attributes()), before);
    assert!(g.compact(&[3]).is_err());
    assert!(g.compact(&[0, 0, 0]).is_err());
    let cell = GlyphCell {
        x: 2,
        y: 0,
        code: Some(65),
        bg: 0,
        fg: 0,
        ext: 0,
        chars: &[65],
        width: 1.0,
        last_bg: 0,
    };
    assert!(g.update_cell(cell, None, 64.0, true).is_err());
    assert_eq!(bits(g.attributes()), before);
    g.resize(0, 0).unwrap();
    assert!(g.compact(&[]).unwrap().is_empty());
}
#[test]
fn dimension_changes_do_not_clear_vertices_until_resize() {
    let mut d = RenderDimensions::default();
    d.device.canvas = Size {
        width: 16.0,
        height: 16.0,
    };
    d.device.cell = Size {
        width: 8.0,
        height: 8.0,
    };
    let mut g = GlyphGeometry::new(2, 2, d).unwrap();
    let cell = GlyphCell {
        x: 0,
        y: 0,
        code: Some(65),
        bg: 0,
        fg: 0,
        ext: 0,
        chars: &[65],
        width: 1.0,
        last_bg: 0,
    };
    let glyph = RasterizedGlyph {
        size: [4.0, 4.0],
        texture_size: [0.25, 0.25],
        ..Default::default()
    };
    g.update_cell(cell, Some(&glyph), 0.0, false).unwrap();
    let before = bits(g.attributes());
    d.device.canvas.width = 32.0;
    g.set_dimensions(d);
    assert_eq!(bits(g.attributes()), before);
    g.update_cell(cell, Some(&glyph), 0.0, false).unwrap();
    assert_eq!(g.attributes()[2], 0.125);
    assert_eq!(g.attributes()[5], 0.0);
    g.resize(2, 2).unwrap();
    assert_eq!(g.attributes()[2], 0.0);
}
