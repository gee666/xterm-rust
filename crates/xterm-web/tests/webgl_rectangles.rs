// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use serde_json::Value;
use xterm_core::buffer::constants::{attributes, fg_flags};
use xterm_web::{
    api::{RenderDimensions, Size},
    theme::ThemeService,
    webgl::{
        model::{CursorRenderModel, CursorShape, RenderModel},
        rectangles::{RectangleError, RectangleGeometry},
    },
};

fn number(value: &Value) -> f64 {
    value.as_f64().unwrap()
}
fn dimensions(cell: [f64; 2], canvas: [f64; 2]) -> RenderDimensions {
    let mut result = RenderDimensions::default();
    result.device.cell = Size {
        width: cell[0],
        height: cell[1],
    };
    result.device.canvas = Size {
        width: canvas[0],
        height: canvas[1],
    };
    result
}
fn pair(value: &Value) -> [f64; 2] {
    [number(&value[0]), number(&value[1])]
}
fn verify(actual: &xterm_web::webgl::rectangles::RectangleVertices, expected: &Value, name: &str) {
    assert_eq!(
        actual.count(),
        expected["count"].as_u64().unwrap() as usize,
        "{name}"
    );
    let bits: Vec<u32> = expected["attributes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u32)
        .collect();
    assert_eq!(
        actual
            .attributes()
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>(),
        bits,
        "{name}"
    );
}
#[test]
fn unchanged_source_vertices_match_directed_and_seeded_cases_bit_for_bit() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/webgl-rectangles.json")).unwrap();
    assert_eq!(fixture["seed"], "0x57e61a23");
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 141);
    let theme = ThemeService::default();
    let shared = theme.colors();
    {
        let colors = shared.borrow_mut();
        colors.foreground.borrow_mut().rgba = fixture["colors"][0].as_u64().unwrap() as u32;
        colors.background.borrow_mut().rgba = fixture["colors"][1].as_u64().unwrap() as u32;
        colors.cursor.borrow_mut().rgba = fixture["colors"][2].as_u64().unwrap() as u32;
        for (index, color) in colors.ansi.iter().enumerate() {
            color.borrow_mut().rgba = fixture["palette"][index].as_u64().unwrap() as u32;
        }
    }
    for case in fixture["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let cols = case["cols"].as_u64().unwrap() as usize;
        let rows = case["rows"].as_u64().unwrap() as usize;
        let mut geometry = RectangleGeometry::new(
            cols,
            rows,
            dimensions(pair(&case["cell"]), pair(&case["canvas"])),
            shared.clone(),
        );
        let mut model = RenderModel::new();
        model.resize(cols, rows).unwrap();
        for (index, cell) in case["cells"].as_array().unwrap().iter().enumerate() {
            model.cells[index * 4 + 1] = cell[1].as_u64().unwrap() as u32;
            model.cells[index * 4 + 2] = cell[0].as_u64().unwrap() as u32;
        }
        let cursor = &case["cursor"];
        if !cursor.is_null() {
            let style = match cursor["style"].as_str().unwrap() {
                "block" => CursorShape::Block,
                "bar" => CursorShape::Bar,
                "underline" => CursorShape::Underline,
                "outline" => CursorShape::Outline,
                "none" => CursorShape::None,
                other => panic!("unknown cursor {other}"),
            };
            model.cursor = Some(CursorRenderModel {
                x: cursor["x"].as_u64().unwrap() as usize,
                y: cursor["y"].as_u64().unwrap() as usize,
                width: cursor["width"].as_u64().unwrap() as usize,
                style,
                cursor_width: number(&cursor["cursorWidth"]),
                device_pixel_ratio: number(&cursor["dpr"]),
            });
        }
        geometry.handle_resize(cols, rows);
        geometry.update_backgrounds(&model).unwrap();
        geometry.update_cursor(&model);
        verify(geometry.backgrounds(), &case["background"], name);
        verify(geometry.cursors(), &case["expectedCursor"], name);
    }
}

#[test]
fn resize_and_theme_only_refresh_viewport_and_cached_cursor_color() {
    let theme = ThemeService::default();
    let mut geometry =
        RectangleGeometry::new(2, 1, dimensions([5.0, 10.0], [10.0, 10.0]), theme.colors());
    let mut model = RenderModel::new();
    model.resize(2, 1).unwrap();
    model.cells[1] = attributes::CM_RGB | 0xff0000;
    geometry.update_backgrounds(&model).unwrap();
    // Construction does not implicitly perform handleResize in the source.
    assert_eq!(&geometry.backgrounds().attributes()[..8], &[0.0; 8]);
    let old_run = geometry.backgrounds().attributes()[8..16].to_vec();
    geometry.handle_resize(2, 1);
    assert_eq!(
        &geometry.backgrounds().attributes()[..4],
        &[0.0, 0.0, 1.0, 1.0]
    );
    geometry.set_dimensions(dimensions([7.0, 12.0], [28.0, 24.0]));
    assert_eq!(geometry.backgrounds().attributes()[2], 1.0);
    theme.colors().borrow().background.borrow_mut().rgba = 0xabcdef80;
    geometry.colors_changed();
    assert_eq!(
        &geometry.backgrounds().attributes()[..4],
        &[0.0, 0.0, 0.5, 0.5]
    );
    assert_eq!(&geometry.backgrounds().attributes()[8..16], old_run);
    geometry.handle_resize(4, 2);
    assert_eq!(
        &geometry.backgrounds().attributes()[..4],
        &[0.0, 0.0, 1.0, 1.0]
    );
    geometry
        .update_backgrounds(&RenderModel::new())
        .unwrap_err();
    assert_eq!(geometry.backgrounds().count(), 2);
}

#[test]
fn invalid_native_model_and_palette_are_errors_not_panics() {
    let theme = ThemeService::default();
    let mut geometry =
        RectangleGeometry::new(1, 1, dimensions([1.0, 1.0], [1.0, 1.0]), theme.colors());
    assert!(matches!(
        geometry.update_backgrounds(&RenderModel::new()),
        Err(RectangleError::ModelTooSmall)
    ));
    let mut model = RenderModel::new();
    model.resize(1, 1).unwrap();
    theme.colors().borrow_mut().ansi.clear();
    model.cells[1] = attributes::CM_P256 | 255;
    assert!(matches!(
        geometry.update_backgrounds(&model),
        Err(RectangleError::MissingPaletteColor(255))
    ));
    model.cells[1] = 0;
    model.cells[2] = fg_flags::INVERSE;
    geometry.update_backgrounds(&model).unwrap();
    assert_eq!(geometry.backgrounds().count(), 2);
    let mut geometry =
        RectangleGeometry::new(usize::MAX, 2, RenderDimensions::default(), theme.colors());
    assert!(matches!(
        geometry.update_backgrounds(&model),
        Err(RectangleError::DimensionsOverflow)
    ));
}
