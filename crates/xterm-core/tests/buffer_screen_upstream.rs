// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Non-resize assertions from common/buffer/Buffer.test.ts. Resize assertions
// also run against Rust through the complete source-derived snapshot fixture.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{AttributeData, Buffer, BufferLine, CellData, CharData},
    options::{OptionValue, OptionsService},
};

fn buffer(scrollback: usize) -> Buffer {
    Buffer::new(
        true,
        80,
        24,
        Rc::new(OptionsService::new(
            false,
            [("scrollback", OptionValue::Number(scrollback as f64))],
        )),
    )
    .unwrap()
}
fn cell(text: &str, width: i32) -> CellData {
    CellData::from_char_data(&CharData {
        attr: 0,
        chars: text.encode_utf16().collect(),
        width,
        code: None,
    })
}
fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
fn replace_line(buffer: &Buffer, text: &[(&str, i32)]) {
    let mut line = BufferLine::new(text.len(), None, false);
    for (x, (text, width)) in text.iter().enumerate() {
        line.set_cell(x, &cell(text, *width));
    }
    buffer
        .lines
        .set(0, Some(Rc::new(RefCell::new(line))))
        .unwrap();
}
#[test]
fn constructor_capacity_is_rows_plus_scrollback() {
    assert_eq!(buffer(1000).lines.max_length(), 24 + 1000);
}
#[test]
fn constructor_scroll_bottom_is_rows_minus_one() {
    assert_eq!(buffer(1000).scroll_bottom, 23);
}
#[test]
fn fill_viewport_has_exact_dimensions_and_blank_cells() {
    let mut buffer = buffer(1000);
    let blank = buffer
        .get_blank_line(&AttributeData::default(), false)
        .borrow()
        .get(0)
        .unwrap();
    buffer.fill_viewport_rows(None).unwrap();
    assert_eq!(buffer.lines.len(), 24);
    for y in 0..24 {
        let line = buffer.lines.get(y).unwrap();
        let line = line.borrow();
        assert_eq!(line.len(), 80);
        for x in 0..80 {
            assert_eq!(line.get(x).unwrap(), blank);
        }
    }
}
macro_rules! wrapped_case {
    ($name:ident, $y:expr, [$($wrap:expr),*], $first:expr, $last:expr) => {
        #[test]
        fn $name() {
            let mut buffer = buffer(1000);
            buffer.fill_viewport_rows(None).unwrap();
            $(buffer.set_wrapped($wrap, true).unwrap();)*
            let range = buffer.get_wrapped_range_for_line($y).unwrap();
            assert_eq!((range.first, range.last), ($first, $last));
        }
    };
}
wrapped_case!(nonwrapped_first, 0, [], 0, 0);
wrapped_case!(nonwrapped_middle, 12, [], 12, 12);
wrapped_case!(nonwrapped_last, 23, [], 23, 23);
wrapped_case!(wrapped_first, 0, [1], 0, 1);
wrapped_case!(wrapped_middle_up, 12, [12], 11, 12);
wrapped_case!(wrapped_middle_down, 12, [13], 12, 13);
wrapped_case!(wrapped_middle_both_ways, 12, [11, 12, 13, 14], 10, 14);
wrapped_case!(wrapped_last, 23, [23], 22, 23);
wrapped_case!(wrapped_to_first_row, 1, [1], 0, 1);
wrapped_case!(wrapped_to_last_row, 22, [23], 22, 23);
#[test]
fn marker_line_adjusts_when_trimmed() {
    let mut buffer = buffer(0);
    buffer.fill_viewport_rows(None).unwrap();
    let marker = buffer.add_marker((buffer.lines.len() - 1) as i64);
    assert_eq!(marker.line(), (buffer.lines.len() - 1) as i64);
    buffer.lines.on_trim_emitter.fire(&1).unwrap();
    assert_eq!(marker.line(), (buffer.lines.len() - 2) as i64);
}
#[test]
fn marker_disposes_when_trimmed_out() {
    let mut buffer = buffer(0);
    buffer.fill_viewport_rows(None).unwrap();
    assert_eq!(buffer.markers.borrow().len(), 0);
    let marker = buffer.add_marker(0);
    assert!(!marker.is_disposed());
    assert_eq!(buffer.markers.borrow().len(), 1);
    buffer.lines.on_trim_emitter.fire(&1).unwrap();
    assert!(marker.is_disposed());
    assert_eq!(buffer.markers.borrow().len(), 0);
}
#[test]
fn marker_calls_on_dispose() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut buffer = buffer(0);
    buffer.fill_viewport_rows(None).unwrap();
    assert_eq!(buffer.markers.borrow().len(), 0);
    let marker = buffer.add_marker(0);
    let output = events.clone();
    let _subscription = marker
        .on_dispose()
        .subscribe(move |_| output.borrow_mut().push("disposed"));
    assert!(!marker.is_disposed());
    assert_eq!(buffer.markers.borrow().len(), 1);
    buffer.lines.on_trim_emitter.fire(&1).unwrap();
    assert!(marker.is_disposed());
    assert_eq!(buffer.markers.borrow().len(), 0);
    assert_eq!(*events.borrow(), ["disposed"]);
}
#[test]
fn translate_ascii_section() {
    let buffer = buffer(1000);
    replace_line(&buffer, &[("a", 1), ("b", 1), ("c", 1), ("d", 1)]);
    assert_eq!(
        buffer
            .translate_buffer_line_to_string(0, true, Some(0), Some(2))
            .unwrap(),
        utf16("ab")
    );
}
#[test]
fn translate_cutoff_wide_character_includes_it() {
    let buffer = buffer(1000);
    replace_line(&buffer, &[("語", 2), ("", 0), ("a", 1)]);
    assert_eq!(
        buffer
            .translate_buffer_line_to_string(0, true, Some(0), Some(1))
            .unwrap(),
        utf16("語")
    );
}
#[test]
fn translate_skips_wide_placeholder() {
    let buffer = buffer(1000);
    replace_line(&buffer, &[("語", 2), ("", 0), ("a", 1)]);
    for (end, expected) in [(1, "語"), (2, "語"), (3, "語a")] {
        assert_eq!(
            buffer
                .translate_buffer_line_to_string(0, true, Some(0), Some(end))
                .unwrap(),
            utf16(expected)
        );
    }
}
#[test]
fn translate_single_width_emoji() {
    let buffer = buffer(1000);
    replace_line(&buffer, &[("😁", 1), ("a", 1)]);
    for (end, expected) in [(1, "😁"), (2, "😁a")] {
        assert_eq!(
            buffer
                .translate_buffer_line_to_string(0, true, Some(0), Some(end))
                .unwrap(),
            utf16(expected)
        );
    }
}
#[test]
fn translate_double_width_emoji() {
    let buffer = buffer(1000);
    replace_line(&buffer, &[("😁", 2), ("", 0)]);
    for end in [1, 2] {
        assert_eq!(
            buffer
                .translate_buffer_line_to_string(0, true, Some(0), Some(end))
                .unwrap(),
            utf16("😁")
        );
    }
    replace_line(&buffer, &[("😁", 2), ("", 0), ("a", 1)]);
    assert_eq!(
        buffer
            .translate_buffer_line_to_string(0, true, Some(0), Some(3))
            .unwrap(),
        utf16("😁a")
    );
}
#[test]
fn idle_cleanup_realigns_retained_storage_after_shrink() {
    let mut buffer = buffer(1000);
    buffer.fill_viewport_rows(None).unwrap();
    buffer.resize(39, 24).unwrap();
    let null = buffer.get_null_cell_snapshot(None);
    for y in 0..24 {
        let line = buffer.lines.get(y).unwrap();
        assert_eq!(line.borrow().len(), 39);
        // Same-size resize reports whether the old backing store still needs
        // cleanup, without clearing it. No production-private field exposure.
        assert!(line.borrow_mut().resize(39, &null));
    }
    assert!(buffer.has_pending_cleanup());
    assert!(buffer.poll_cleanup().unwrap());
    assert!(!buffer.poll_cleanup().unwrap());
    for y in 0..24 {
        let line = buffer.lines.get(y).unwrap();
        assert!(!line.borrow_mut().resize(39, &null));
        assert_eq!(line.borrow().len(), 39);
    }
}
