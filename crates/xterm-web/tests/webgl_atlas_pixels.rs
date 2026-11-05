// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
#[path = "support/atlas_pixel_fixture.rs"]
mod fixture;
use serde_json::Value;
use xterm_web::{
    api::Size,
    webgl::atlas_pixels::{
        clear_background, completely_transparent, find_glyph_bounds, GlyphScan, PixelError,
    },
};

#[test]
fn unchanged_source_rgba_and_bounds() {
    let input: Value =
        serde_json::from_str(include_str!("fixtures/webgl-atlas-pixels.json")).unwrap();
    assert_eq!(input["cases"].as_array().unwrap().len(), 656);
    for c in input["cases"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let mut data = fixture::pixels(&c["pixels"]);
        let scan = fixture::scan(&c["scan"]);
        let width = c["width"].as_u64().unwrap() as usize;
        let height = c["height"].as_u64().unwrap() as usize;
        assert_eq!(
            completely_transparent(&data).unwrap(),
            c["beforeTransparent"].as_bool().unwrap(),
            "{name} before transparent"
        );
        fixture::assert_bounds(
            find_glyph_bounds(&data, width, height, scan).unwrap(),
            &c["beforeBounds"],
            name,
        );
        let empty = clear_background(
            &mut data,
            c["bg"].as_u64().unwrap() as u32,
            c["fg"].as_u64().unwrap() as u32,
            c["threshold"].as_bool().unwrap(),
        )
        .unwrap();
        assert_eq!(empty, c["empty"].as_bool().unwrap(), "{name} color empty");
        assert_eq!(data, fixture::pixels(&c["cleared"]), "{name} pixels");
        assert_eq!(
            completely_transparent(&data).unwrap(),
            c["afterTransparent"].as_bool().unwrap(),
            "{name} after transparent"
        );
        fixture::assert_bounds(
            find_glyph_bounds(&data, width, height, scan).unwrap(),
            &c["afterBounds"],
            name,
        );
    }
}
fn scan() -> GlyphScan {
    GlyphScan {
        allowed_width: 1,
        padding: 0,
        restricted: false,
        custom: false,
        cell: Size {
            width: 1.0,
            height: 1.0,
        },
        character: Size {
            width: 1.0,
            height: 1.0,
        },
        line_height: 1.0,
    }
}
#[test]
fn malformed_buffers_and_geometry_return_errors_without_mutation() {
    let mut partial = [1, 2, 3];
    assert_eq!(
        clear_background(&mut partial, 0, 0, true),
        Err(PixelError::IncompletePixel)
    );
    assert_eq!(partial, [1, 2, 3]);
    assert_eq!(
        completely_transparent(&partial),
        Err(PixelError::IncompletePixel)
    );
    assert_eq!(
        find_glyph_bounds(&[0; 4], usize::MAX, 2, scan()),
        Err(PixelError::ImageDimensions)
    );
    assert_eq!(
        find_glyph_bounds(&[0; 4], 2, 1, scan()),
        Err(PixelError::ImageDimensions)
    );
    let mut s = scan();
    s.padding = usize::MAX;
    assert_eq!(
        find_glyph_bounds(&[0; 4], 1, 1, s),
        Err(PixelError::ScanDimensions)
    );
    s = scan();
    s.restricted = true;
    s.cell.width = 0.5;
    assert_eq!(
        find_glyph_bounds(&[0; 4], 1, 1, s),
        Err(PixelError::ScanDimensions)
    );
    s.cell.width = 1.0;
    s.cell.height = 2.0;
    assert_eq!(
        find_glyph_bounds(&[0; 4], 1, 1, s),
        Err(PixelError::ScanDimensions)
    );
    s = scan();
    s.character.height = f64::NAN;
    assert_eq!(
        find_glyph_bounds(&[0; 4], 1, 1, s),
        Err(PixelError::ScanDimensions)
    );
}
#[test]
fn zero_area_images_do_not_walk_unbacked_rows_or_columns() {
    let mut s = scan();
    s.allowed_width = usize::MAX;
    let wide = find_glyph_bounds(&[], usize::MAX, 0, s).unwrap();
    assert_eq!(
        (wide.left, wide.top, wide.right, wide.bottom),
        (0, 0, usize::MAX, 0)
    );
    s.allowed_width = 0;
    let tall = find_glyph_bounds(&[], 0, usize::MAX, s).unwrap();
    assert_eq!(
        (tall.left, tall.top, tall.right, tall.bottom),
        (0, 0, 0, usize::MAX)
    );
}
#[test]
fn color_empty_and_alpha_empty_are_deliberately_different() {
    let mut transparent_red = [255, 0, 0, 0];
    assert!(completely_transparent(&transparent_red).unwrap());
    assert!(!clear_background(&mut transparent_red, 0x000000ff, 0xffffffff, false).unwrap());
    let mut opaque_red = [255, 0, 0, 255];
    assert!(clear_background(&mut opaque_red, 0xff000000, 0xffffffff, false).unwrap());
    assert_eq!(opaque_red, [255, 0, 0, 0]);
    assert!(clear_background(&mut [], 0, 0, true).unwrap());
    assert!(completely_transparent(&[]).unwrap());
}
