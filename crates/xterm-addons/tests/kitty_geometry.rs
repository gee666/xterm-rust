// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Source arithmetic only. Browser bitmap coercion and rendered pixels remain
// separate acceptance tests, not simulated by these stage comparisons.
use serde_json::{json, Value};
use xterm_addons::image::kitty::{command::parse_command, geometry::*};
use xterm_addons::image::storage::{CellSize, ImageLayer};

fn f(value: &Value) -> f64 {
    let text = value.as_str().unwrap();
    if text == "nan" {
        f64::NAN
    } else {
        f64::from_bits(u64::from_str_radix(text, 16).unwrap())
    }
}
fn n(value: f64) -> Value {
    json!(if value.is_nan() {
        "nan".to_owned()
    } else {
        format!("{:016x}", value.to_bits())
    })
}
fn bitmap(input: &Value) -> BitmapSize {
    BitmapSize {
        width: u32::try_from(input["bitmap"]["width"].as_u64().unwrap()).unwrap(),
        height: u32::try_from(input["bitmap"]["height"].as_u64().unwrap()).unwrap(),
    }
}
fn input_layout(value: &Value) -> Layout {
    Layout {
        cell: CellSize {
            width: f(&value["cw"]),
            height: f(&value["ch"]),
        },
        columns: f(&value["columns"]),
        rows: f(&value["rows"]),
        width: f(&value["width"]),
        height: f(&value["height"]),
        layer: if value["layer"] == "bottom" {
            ImageLayer::Bottom
        } else {
            ImageLayer::Top
        },
    }
}
fn output_layout(value: Layout) -> Value {
    json!({"cw":n(value.cell.width),"ch":n(value.cell.height),"columns":n(value.columns),"rows":n(value.rows),"width":n(value.width),"height":n(value.height),"layer":if value.layer==ImageLayer::Bottom {"bottom"} else {"top"}})
}

#[test]
fn geometry_stages_match_original_arithmetic_including_nan_and_signed_zero() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/kitty-geometry.json")).unwrap();
    assert_eq!(rows.len(), 4148);
    for (index, row) in rows.iter().enumerate() {
        let input = &row["input"];
        let mut command = parse_command(
            &row["control"]
                .as_str()
                .unwrap()
                .encode_utf16()
                .collect::<Vec<_>>(),
        );
        for (key, value) in row["overrides"].as_object().unwrap() {
            let target = match key.as_str() {
                "x" => &mut command.x,
                "y" => &mut command.y,
                "sourceWidth" => &mut command.source_width,
                "sourceHeight" => &mut command.source_height,
                "xOffset" => &mut command.x_offset,
                "yOffset" => &mut command.y_offset,
                "columns" => &mut command.columns,
                "rows" => &mut command.rows,
                "zIndex" => &mut command.z_index,
                "cursorMovement" => &mut command.cursor_movement,
                other => panic!("unknown override {other}"),
            };
            *target = Some(f(value));
        }
        let result: Result<Value, PlacementError> = match row["kind"].as_str().unwrap() {
            "crop" => crop(bitmap(input), &command).map(|value| json!({"x":n(value.x),"y":n(value.y),"width":n(value.width),"height":n(value.height),"required":value.required})),
            "layout" => layout(bitmap(input), CellSize { width: f(&input["cell"]["width"]), height: f(&input["cell"]["height"]) }, &command, f(&input["limit"])).map(output_layout),
            "offset" => Ok(offset(bitmap(input), input_layout(&input["layout"]), &command).map_or(Value::Null, |value| json!({"x":n(value.x),"y":n(value.y),"width":n(value.canvas_width),"height":n(value.canvas_height)}))),
            "complete" => complete_offset(bitmap(input), input_layout(&input["layout"]), &command, f(&input["limit"])).map(output_layout),
            "cursor" => {
                let value = cursor(&command, f(&input["savedX"]), f(&input["savedY"]), f(&input["savedYbase"]), f(&input["currentYbase"]), f(&input["columns"]), f(&input["imgCols"]));
                Ok(json!({"x":n(value.x),"y":n(value.y.unwrap_or_else(|| f(&input["currentY"]))) }))
            }
            other => panic!("unknown stage {other}"),
        };
        let actual = match result {
            Ok(value) => json!({"value":value}),
            Err(error) => json!({"error":error.to_string()}),
        };
        assert_eq!(
            actual, row["expected"],
            "case {index} stage {} control {}",
            row["kind"], row["control"]
        );
    }
}

#[test]
fn layer_is_captured_before_await_but_z_index_is_read_at_insertion() {
    let mut command = parse_command(&"c=1,z=-1".encode_utf16().collect::<Vec<_>>());
    let plan = layout(
        BitmapSize {
            width: 7,
            height: 14,
        },
        CellSize {
            width: 7.0,
            height: 14.0,
        },
        &command,
        1000.0,
    )
    .unwrap();
    command.z_index = Some(5.0);
    let options = placement_options(plan, &command);
    assert_eq!(options.layer, ImageLayer::Bottom);
    assert_eq!(options.z_index, 5.0);
}
