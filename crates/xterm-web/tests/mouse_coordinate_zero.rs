// Copyright (c) 2017-2026 The xterm.js authors. MIT license.
use xterm_web::input::mouse_coords::MouseGeometry;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn number(text: &str) -> f64 {
    text.parse().unwrap()
}

fn same_number(actual: f64, expected: &str, row: usize, field: usize) {
    let expected = number(expected);
    if expected.is_nan() {
        assert!(actual.is_nan(), "row {row} field {field}: expected NaN");
    } else {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "row {row} field {field}: actual {actual:?}, expected {expected:?}"
        );
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn source_coordinate_zero_signs_and_nonfinite_bounds() {
    let mut count = 0;
    // Dedicated numeric fixture keeps zero signs literal, unlike CSS strings in
    // the broader mouse-policy corpus. Compare bits, not f64 PartialEq.
    for (row, text) in include_str!("fixtures/mouse-coordinate-zero.tsv")
        .lines()
        .enumerate()
    {
        let p: Vec<_> = text.split('\t').collect();
        assert_eq!(p.len(), 20, "row {row}");
        let geometry = MouseGeometry {
            valid_char_size: p[0] == "1",
            rect_left: number(p[1]),
            rect_top: number(p[2]),
            padding_left: number(p[3]),
            padding_top: number(p[4]),
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
        assert_eq!(coords.is_some(), p[14] != "u", "row {row}");
        if let Some(coords) = coords {
            same_number(coords[0], p[14], row, 14);
            same_number(coords[1], p[15], row, 15);
        }
        let report = geometry.report_coords(number(p[9]), number(p[10]));
        assert_eq!(report.is_some(), p[16] != "u", "row {row}");
        if let Some(report) = report {
            for (field, actual) in [report.col, report.row, report.x, report.y]
                .into_iter()
                .enumerate()
            {
                same_number(actual, p[16 + field], row, 16 + field);
            }
        }
        count += 1;
    }
    assert_eq!(count, 3249);
}
