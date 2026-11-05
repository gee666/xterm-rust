// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
mod serialize_html_support;
use serialize_html_support::*;
use xterm_core::serialize::{
    serialize_html_numeric_rows, BufferAccess, HtmlNumericRange, HtmlOptions, SerializeError,
};

fn numeric(
    buffer: &dyn BufferAccess,
    start: f64,
    end: f64,
    start_col: usize,
    end_col: usize,
) -> String {
    String::from_utf16(
        &serialize_html_numeric_rows(
            buffer,
            HtmlNumericRange {
                start_line: start,
                end_line: end,
                start_col,
                end_col,
            },
            &config(),
            false,
        )
        .unwrap(),
    )
    .unwrap()
}
fn rows(output: &str) -> Vec<&str> {
    output
        .split("<div><span>")
        .skip(1)
        .map(|row| row.split("</span></div>").next().unwrap())
        .collect()
}

#[test]
fn negative_rows_are_empty_without_skipping_later_real_rows() {
    let text = buffer(&["abc", "def"], 3);
    // The first row's column restriction belongs to missing row -1, not row 0.
    assert_eq!(rows(&numeric(&text, -1.0, 1.0, 2, 2)), ["", "abc", "de"]);
}

#[test]
fn fractional_rows_are_missing_and_increment_without_truncation() {
    let text = buffer(&["abc", "def"], 3);
    assert_eq!(rows(&numeric(&text, -0.5, 1.5, 0, 3)), ["", "", ""]);
    assert_eq!(rows(&numeric(&text, 0.5, 1.5, 0, 3)), ["", ""]);
    // A fractional final bound never equals row 1, so its live line length wins.
    assert_eq!(rows(&numeric(&text, 0.0, 1.5, 1, 1)), ["bc", "def"]);
}

#[test]
fn large_missing_rows_and_reversed_ranges_keep_header_and_footer() {
    let text = buffer(&["abc"], 3);
    let large = 4_294_967_296.0;
    assert_eq!(rows(&numeric(&text, large, large + 1.0, 0, 3)), ["", ""]);
    let last = 9_007_199_254_740_991.0;
    assert_eq!(rows(&numeric(&text, last, last, 0, 3)), [""]);
    let reversed = numeric(&text, 1.0, -1.0, 0, 3);
    assert!(rows(&reversed).is_empty());
    assert!(reversed.ends_with("</div></pre><!--EndFragment--></body></html>"));
}

#[test]
fn native_integral_output_and_negative_zero_are_unchanged() {
    let text = buffer(&["a<&", "def"], 3);
    let expected = html(
        &text,
        3,
        &HtmlOptions {
            only_selection: true,
            ..HtmlOptions::default()
        },
        Some(range(1, 0, 2, 1)),
        &config(),
    );
    assert_eq!(numeric(&text, -0.0, 1.0, 1, 2), expected);
}

#[test]
fn full_rotated_ring_preserves_raw_numeric_key_aliases() {
    use std::{cell::RefCell, rc::Rc};
    use xterm_core::{
        buffer::{AttributeData, Buffer as NativeBuffer},
        options::{OptionValue, OptionsService},
        public_api::{BufferApiView, BufferType},
        serialize::PublicBufferView,
    };
    let options = Rc::new(OptionsService::new(
        false,
        [("scrollback", OptionValue::Number(0.0))],
    ));
    let mut buffer = NativeBuffer::new(true, 3, 2, options).unwrap();
    buffer.fill_viewport_rows(None).unwrap();
    buffer.lines.recycle().unwrap();
    for (row, text) in ["def", "ghi"].iter().enumerate() {
        let line = buffer.lines.get(row).unwrap();
        for (col, ch) in text.chars().enumerate() {
            line.borrow_mut().set_cell_from_codepoint(
                col,
                u32::from(ch),
                1,
                &AttributeData::default(),
            );
        }
    }
    let expected = [
        (-1.0, "ghi"),
        (f64::from_bits(1), "def"),
        (4_294_967_296.0, "def"),
        (9_007_199_254_740_991.0, "ghi"),
    ];
    for (row, text) in expected {
        assert_eq!(rows(&numeric(&buffer, row, row, 0, 3)), [text]);
    }
    let view = PublicBufferView::new(Rc::new(BufferApiView::new(
        Rc::new(RefCell::new(buffer)),
        BufferType::Normal,
    )))
    .unwrap();
    for (row, text) in expected {
        assert_eq!(rows(&numeric(&view, row, row, 0, 3)), [text]);
    }
}

#[test]
fn nonprogressing_and_nonfinite_ranges_are_explicit_errors() {
    let text = buffer(&["abc"], 3);
    for bound in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        9_007_199_254_740_992.0,
    ] {
        for (start, end) in [(bound, 0.0), (0.0, bound)] {
            let result = serialize_html_numeric_rows(
                &text,
                HtmlNumericRange {
                    start_line: start,
                    end_line: end,
                    start_col: 0,
                    end_col: 3,
                },
                &config(),
                false,
            );
            assert_eq!(result, Err(SerializeError::InvalidRange));
        }
    }
}
