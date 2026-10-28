// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
use std::convert::Infallible;
use xterm_core::mouse_state::{MouseAction, MouseButton, MouseEvent, MouseStateService};
use xterm_web::input::mouse_coords::{parse_css_padding, MouseGeometry};
use xterm_web::input::mouse_policy::{MouseOutput, MousePolicy};
use xterm_web::input::mouse_wheel::{cursor_sequence, WheelInput, WheelOptions, WheelState};

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn number(text: &str) -> f64 {
    text.parse().unwrap()
}
fn optional_number(text: &str) -> Option<f64> {
    if text == "u" {
        None
    } else {
        Some(number(text))
    }
}
fn flag(text: &str) -> Option<bool> {
    match text {
        "u" => None,
        "0" => Some(false),
        "1" => Some(true),
        _ => panic!("bad flag"),
    }
}
fn units(text: &str) -> Vec<u16> {
    if text == "-" {
        return Vec::new();
    }
    text.split(',')
        .map(|unit| u16::from_str_radix(unit, 16).unwrap())
        .collect()
}
fn same_number(actual: f64, expected: f64, row: usize) {
    if expected.is_nan() {
        assert!(actual.is_nan(), "row {row}: expected NaN, actual {actual}");
    } else {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "row {row}: actual {actual}, expected {expected}"
        );
    }
}
fn event(fields: &[&str]) -> MouseEvent {
    MouseEvent {
        col: number(fields[0]),
        row: number(fields[1]),
        x: number(fields[2]),
        y: number(fields[3]),
        button: MouseButton::Raw(fields[4].parse().unwrap()),
        action: MouseAction::try_from(fields[5].parse::<u8>().unwrap()).unwrap(),
        ctrl: flag(fields[6]),
        alt: flag(fields[7]),
        shift: flag(fields[8]),
    }
}
fn compare_event(actual: &MouseEvent, expected: &MouseEvent, row: usize) {
    for (actual, expected) in [
        (actual.col, expected.col),
        (actual.row, expected.row),
        (actual.x, expected.x),
        (actual.y, expected.y),
    ] {
        same_number(actual, expected, row);
    }
    assert_eq!(actual.button, expected.button, "button row {row}");
    assert_eq!(actual.action, expected.action, "action row {row}");
    assert_eq!(
        (actual.ctrl, actual.alt, actual.shift),
        (expected.ctrl, expected.alt, expected.shift),
        "mods row {row}"
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn every_original_trigger_invocation_matches_report_and_mutation() {
    let mut previous = "";
    let mut policy = MousePolicy::new();
    let mut service = MouseStateService::<()>::new();
    let mut count = 0;
    for (row, text) in include_str!("fixtures/browser-mouse-trigger.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = text.split('\t').collect();
        assert_eq!(fields.len(), 26, "row {row}");
        if previous != fields[25] {
            policy = MousePolicy::new();
            service = MouseStateService::new();
            previous = fields[25];
        }
        service.set_active_protocol(fields[0]).unwrap();
        service.set_active_encoding(fields[1]).unwrap();
        let mut input = event(&fields[4..13]);
        let mut reports = Vec::new();
        let accepted = policy
            .trigger(
                &service,
                &mut input,
                number(fields[2]),
                number(fields[3]),
                |output| {
                    reports.push(output);
                    Ok::<_, Infallible>(())
                },
            )
            .unwrap();
        assert_eq!(
            accepted,
            fields[13] == "1",
            "accepted row {row}, {}",
            fields[25]
        );
        compare_event(&input, &event(&fields[14..23]), row);
        let expected = match fields[23] {
            "-" => Vec::new(),
            "data" => vec![MouseOutput::UserData(units(fields[24]))],
            "binary" => vec![MouseOutput::Binary(units(fields[24]))],
            _ => panic!("bad channel"),
        };
        assert_eq!(reports, expected, "reports row {row}, {}", fields[25]);
        count += 1;
    }
    assert_eq!(count, 1562);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn every_source_coordinate_snapshot_matches_ieee_and_padding_semantics() {
    let mut count = 0;
    for (row, text) in include_str!("fixtures/browser-mouse-coords.tsv")
        .lines()
        .enumerate()
    {
        let p: Vec<_> = text.split('\t').collect();
        assert_eq!(p.len(), 20);
        let left = String::from_utf16(&units(p[3])).unwrap();
        let top = String::from_utf16(&units(p[4])).unwrap();
        let geometry = MouseGeometry {
            valid_char_size: p[0] == "1",
            rect_left: number(p[1]),
            rect_top: number(p[2]),
            padding_left: parse_css_padding(&left),
            padding_top: parse_css_padding(&top),
            cell_width: number(p[5]),
            cell_height: number(p[6]),
            canvas_width: number(p[7]),
            canvas_height: number(p[8]),
        };
        let coords = geometry.coords(
            number(p[9]),
            number(p[10]),
            number(p[11]),
            number(p[12]),
            p[13] == "1",
        );
        let expected = optional_number(p[14]);
        assert_eq!(coords.is_some(), expected.is_some());
        if let Some(coords) = coords {
            same_number(coords[0], expected.unwrap(), row);
            same_number(coords[1], number(p[15]), row);
        }
        let report = geometry.report_coords(number(p[9]), number(p[10]));
        assert_eq!(report.is_some(), p[16] != "u");
        if let Some(report) = report {
            for (actual, expected) in [report.col, report.row, report.x, report.y]
                .into_iter()
                .zip(&p[16..])
            {
                same_number(actual, number(expected), row);
            }
        }
        count += 1;
    }
    assert_eq!(count, 1024);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn every_source_wheel_operation_preserves_partial_accumulator() {
    let mut state = WheelState::default();
    let mut count = 0;
    for (row, text) in include_str!("fixtures/browser-mouse-wheel.tsv")
        .lines()
        .enumerate()
    {
        if text == "reset" {
            state.reset();
            count += 1;
            continue;
        }
        let p: Vec<_> = text.split('\t').collect();
        assert_eq!(p.len(), 10);
        let mask: u8 = p[2].parse().unwrap();
        let output = state.consume(
            WheelInput {
                delta_y: number(p[0]),
                delta_mode: number(p[1]),
                ctrl: mask & 1 != 0,
                alt: mask & 2 != 0,
                shift: mask & 4 != 0,
            },
            optional_number(p[3]),
            optional_number(p[4]),
            WheelOptions {
                scroll_sensitivity: number(p[5]),
                fast_scroll_sensitivity: number(p[6]),
                rows: number(p[7]),
            },
        );
        same_number(output, number(p[8]), row);
        same_number(state.partial_scroll(), number(p[9]), row);
        count += 1;
    }
    assert_eq!(count, 3034);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn every_source_touch_operation_preserves_accumulator_and_cursor_output() {
    let mut state = WheelState::default();
    let mut count = 0;
    for (row, text) in include_str!("fixtures/browser-mouse-touch.tsv")
        .lines()
        .enumerate()
    {
        if text == "reset" {
            state.touch_start();
            count += 1;
            continue;
        }
        let fields: Vec<_> = text.split('\t').collect();
        assert_eq!(fields.len(), 5);
        let lines = state.consume_touch(number(fields[1]), optional_number(fields[0]));
        same_number(state.touch_accumulator(), number(fields[3]), row);
        assert!(lines.is_finite());
        let expected = if fields[4] == "-" {
            Vec::new()
        } else {
            fields[4].split(';').map(units).collect::<Vec<_>>()
        };
        let mut actual = Vec::new();
        let mut index = 0.0;
        while index < lines.abs() {
            actual.push(cursor_sequence(lines < 0.0, fields[2] == "1").to_vec());
            index += 1.0;
        }
        assert_eq!(actual, expected, "touch outputs row {row}");
        count += 1;
    }
    assert_eq!(count, 516);
}

fn geometry() -> MouseGeometry {
    MouseGeometry {
        rect_left: 0.0,
        rect_top: 0.0,
        padding_left: 0.0,
        padding_top: 0.0,
        valid_char_size: true,
        cell_width: 10.0,
        cell_height: 20.0,
        canvas_width: 100.0,
        canvas_height: 200.0,
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn upstream_clicked_cell() {
    let g = geometry();
    assert_eq!(g.coords(5.0, 10.0, 10.0, 10.0, false), Some([1.0, 1.0]));
    assert_eq!(g.coords(10.0, 20.0, 10.0, 10.0, false), Some([1.0, 1.0]));
    assert_eq!(g.coords(10.0, 21.0, 10.0, 10.0, false), Some([1.0, 2.0]));
    assert_eq!(g.coords(11.0, 20.0, 10.0, 10.0, false), Some([2.0, 1.0]));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn upstream_viewport_bounds() {
    let g = geometry();
    assert_eq!(g.coords(-1.0, -1.0, 10.0, 10.0, false), Some([1.0, 1.0]));
    assert_eq!(
        g.coords(200.0, 400.0, 10.0, 10.0, false),
        Some([10.0, 10.0])
    );
}
