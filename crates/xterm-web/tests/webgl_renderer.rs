// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
mod webgl_renderer_support;
use serde_json::{json, Value};
use webgl_renderer_support::{usize_value, Host};
use xterm_web::{
    renderer::{row_types::Selection, selection_model::SelectionRenderModel},
    webgl::{model::CursorShape, renderer_types::UpdateContext, renderer_update::RendererState},
};
fn shape(value: &Value) -> CursorShape {
    match value.as_str().unwrap() {
        "block" => CursorShape::Block,
        "bar" => CursorShape::Bar,
        "underline" => CursorShape::Underline,
        "outline" => CursorShape::Outline,
        "none" => CursorShape::None,
        _ => panic!("unknown shape"),
    }
}
fn shape_name(shape: CursorShape) -> &'static str {
    match shape {
        CursorShape::Block => "block",
        CursorShape::Bar => "bar",
        CursorShape::Underline => "underline",
        CursorShape::Outline => "outline",
        CursorShape::None => "none",
    }
}
fn context(c: &Value) -> UpdateContext {
    UpdateContext {
        cols: usize_value(&c["cols"]),
        rows: usize_value(&c["rows"]),
        viewport_y: usize_value(&c["viewport"]),
        cursor_x: usize_value(&c["cursorX"]),
        cursor_y: usize_value(&c["cursorY"]),
        focused: c["focused"].as_bool().unwrap(),
        cursor_initialized: c["initialized"].as_bool().unwrap(),
        cursor_hidden: c["hidden"].as_bool().unwrap(),
        cursor_blink_visible: c["cursorVisible"].as_bool().unwrap(),
        cursor_style: shape(&c["style"]),
        inactive_style: shape(&c["inactive"]),
        cursor_width: c["cursorWidth"].as_f64().unwrap(),
        device_pixel_ratio: c["dpr"].as_f64().unwrap(),
        text_blink_enabled: c["textEnabled"].as_bool().unwrap(),
        text_blink_on: c["textOn"].as_bool().unwrap(),
    }
}
#[test]
fn unchanged_webgl_renderer_model_and_frame_traces() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/webgl-renderer.json")).unwrap();
    assert_eq!(fixture["seed"], "0x54b7d329");
    let mut steps = 0;
    let mut errors = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let c = context(&case["steps"][0]["context"]);
        let mut renderer = RendererState::default();
        renderer.resize(c.cols, c.rows).unwrap();
        let mut host = Host::new(c);
        for (index, step) in case["steps"].as_array().unwrap().iter().enumerate() {
            steps += 1;
            let label = format!("{} step{index}", case["name"]);
            let c = context(&step["context"]);
            host.step(step, c);
            let selection = &step["selection"];
            renderer.model.selection = SelectionRenderModel::update(
                c.rows,
                c.viewport_y,
                Selection {
                    start: serde_json::from_value(selection["start"].clone()).unwrap(),
                    end: serde_json::from_value(selection["end"].clone()).unwrap(),
                    column: selection["column"].as_bool().unwrap_or(false),
                },
            );
            let start = step["start"].as_i64().unwrap();
            let end = step["end"].as_i64().unwrap();
            let result = if step["frame"].as_bool().unwrap() {
                renderer.render_rows(start, end, c, &mut host)
            } else {
                renderer.update(start, end, c, &mut host)
            };
            if step["expected"]["error"].is_null() {
                result.unwrap_or_else(|e| panic!("{label}: {e:?}"));
            } else {
                assert!(result.is_err(), "{label}: expected source error");
                errors += 1;
            }
            assert_eq!(
                host.trace,
                step["expected"]["trace"].as_array().unwrap().clone(),
                "{label} trace"
            );
            if case["name"] == "frame-41" {
                assert_eq!(host.trace.iter().filter(|e| e[0] == "begin").count(), 34);
                assert_eq!(host.trace.iter().filter(|e| e[0] == "clear").count(), 32);
                assert_eq!(
                    host.trace.iter().filter(|e| e[0] == "invalidate").count(),
                    1
                );
            }
            let cursor=renderer.model.cursor.map(|v|json!({"x":v.x,"y":v.y,"width":v.width,"style":shape_name(v.style),"cursorWidth":v.cursor_width,"dpr":v.device_pixel_ratio}));
            assert_eq!(
                json!(renderer.model.cells),
                step["expected"]["cells"],
                "{label} cells"
            );
            assert_eq!(
                json!(renderer.model.line_lengths),
                step["expected"]["lengths"],
                "{label} lengths"
            );
            let mut expected_cursor = step["expected"]["cursor"].clone();
            if !expected_cursor.is_null() {
                for key in ["cursorWidth", "dpr"] {
                    expected_cursor[key] = json!(expected_cursor[key].as_f64().unwrap());
                }
            }
            assert_eq!(json!(cursor), expected_cursor, "{label} cursor");
            assert_eq!(
                json!(renderer.blinking_rows()),
                step["expected"]["blinking"],
                "{label} blink rows"
            );
            assert_eq!(
                json!(renderer.blinking_row_count()),
                step["expected"]["count"],
                "{label} blink count"
            );
            let r = renderer.colors.result;
            assert_eq!(
                json!([r.bg, r.fg, r.ext]),
                step["expected"]["colors"],
                "{label} resolver"
            );
        }
    }
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 162);
    assert_eq!(steps, 455);
    assert_eq!(errors, 4);
}

#[test]
fn invalid_native_grid_and_nonprogressing_joins_are_errors() {
    use xterm_web::webgl::renderer_types::UpdateError;
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/webgl-renderer.json")).unwrap();
    let step = &fixture["cases"][0]["steps"][0];
    let c = context(&step["context"]);
    let mut renderer = RendererState::default();
    let mut host = Host::new(c);
    host.step(step, c);
    assert!(matches!(
        renderer.update(0, 2, c, &mut host),
        Err(UpdateError::InvalidGrid)
    ));
    assert!(host.trace.is_empty());
    renderer.resize(c.cols, c.rows).unwrap();
    for range in [[0, 0], [0, c.cols + 1]] {
        host.step(step, c);
        host.joins[0] = vec![range];
        assert!(
            matches!(renderer.update(0, 0, c, &mut host), Err(UpdateError::InvalidJoin(r)) if r == range)
        );
        assert_eq!(host.trace.last().unwrap(), &json!(["load", 0]));
    }
    renderer.reset_blinking_rows();
    assert_eq!(renderer.blinking_row_count(), 0);
    assert!(renderer.blinking_rows().iter().all(|b| !b));
}
