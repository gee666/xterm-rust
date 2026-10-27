// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{AttributeData, Buffer, BufferError, BufferLine},
    circular_list::ListChange,
    options::{OptionValue, OptionsService},
};
fn buffer(cols: usize, rows: usize) -> Buffer {
    Buffer::new(true, cols, rows, Rc::new(OptionsService::default())).unwrap()
}
#[test]
fn zero_dimensions_reject_without_mutation() {
    let options = Rc::new(OptionsService::default());
    assert!(matches!(
        Buffer::new(true, 0, 1, options.clone()),
        Err(BufferError::InvalidDimensions)
    ));
    assert!(matches!(
        Buffer::new(true, 1, 0, options),
        Err(BufferError::InvalidDimensions)
    ));
    let mut buffer = buffer(4, 3);
    buffer.fill_viewport_rows(None).unwrap();
    let first = buffer.lines.get(0).unwrap();
    for (cols, rows) in [(0, 3), (4, 0)] {
        assert!(matches!(
            buffer.resize(cols, rows),
            Err(BufferError::InvalidDimensions)
        ));
        assert_eq!(
            (buffer.cols(), buffer.rows(), buffer.lines.len()),
            (4, 3, 3)
        );
        assert!(Rc::ptr_eq(&first, &buffer.lines.get(0).unwrap()));
    }
}
#[test]
fn scratch_cells_share_extended_attributes_but_default_resets_them() {
    let buffer = buffer(4, 3);
    let mut attr = AttributeData {
        fg: 123,
        bg: 456,
        ..AttributeData::default()
    };
    attr.extended.set_url_id(789);
    let null = buffer.get_null_cell(Some(&attr));
    let space = buffer.get_whitespace_cell(Some(&attr));
    assert_eq!(null.get_code(), Some(0));
    assert_eq!(space.get_code(), Some(32));
    assert!(null.attributes.extended.same_identity(&attr.extended));
    assert!(space.attributes.extended.same_identity(&attr.extended));
    attr.extended.set_url_id(33);
    assert_eq!(null.attributes.extended.url_id(), 33);
    assert_eq!(space.attributes.extended.url_id(), 33);
    for cell in [buffer.get_null_cell(None), buffer.get_whitespace_cell(None)] {
        assert_eq!(
            (
                cell.attributes.fg,
                cell.attributes.bg,
                cell.attributes.extended.url_id()
            ),
            (0, 0, 0)
        );
        assert_eq!(cell.get_width(), 1);
    }
}
#[test]
fn blank_line_uses_live_host_width_not_last_buffer_resize() {
    let mut buffer = buffer(4, 3);
    buffer.resize(6, 3).unwrap();
    assert_eq!(
        buffer
            .get_blank_line(&AttributeData::default(), true)
            .borrow()
            .len(),
        4
    );
    buffer.host_dimensions.cols.set(9);
    let blank = buffer.get_blank_line(&AttributeData::default(), true);
    assert_eq!(blank.borrow().len(), 9);
    assert!(blank.borrow().is_wrapped);
}
#[test]
fn fill_does_not_pad_a_partially_filled_viewport() {
    let mut buffer = buffer(4, 3);
    buffer
        .lines
        .push(Rc::new(RefCell::new(BufferLine::new(4, None, false))))
        .unwrap();
    buffer.fill_viewport_rows(None).unwrap();
    assert_eq!(buffer.lines.len(), 1);
}
#[test]
fn clear_preserves_saved_state_and_source_marker_lifetime() {
    let mut buffer = buffer(4, 3);
    buffer.fill_viewport_rows(None).unwrap();
    let marker = buffer.add_marker(1);
    buffer.saved_x = 3;
    buffer.saved_y = 2;
    buffer.saved_origin_mode = true;
    buffer.saved_wraparound_mode = false;
    buffer.x = 2;
    buffer.y = 1;
    buffer.ybase = 4;
    buffer.ydisp = 3;
    buffer.scroll_top = 1;
    buffer.clear().unwrap();
    assert_eq!(
        (buffer.x, buffer.y, buffer.ybase, buffer.ydisp),
        (0, 0, 0, 0)
    );
    assert_eq!((buffer.saved_x, buffer.saved_y), (3, 2));
    assert!(buffer.saved_origin_mode);
    assert!(!buffer.saved_wraparound_mode);
    assert_eq!((buffer.scroll_top, buffer.scroll_bottom), (0, 2));
    assert_eq!(buffer.lines.len(), 0);
    // Upstream clear replaces the CircularList but does not clear markers.
    assert!(!marker.is_disposed());
    buffer.dispose().unwrap();
    assert!(marker.is_disposed());
}
#[test]
fn cursor_viewport_uses_base_display_and_exclusive_bottom() {
    let mut buffer = buffer(4, 3);
    buffer.ybase = 10;
    buffer.y = 1;
    for (display, visible) in [(8, false), (9, true), (11, true), (12, false)] {
        buffer.ydisp = display;
        assert_eq!(buffer.is_cursor_in_viewport(), visible);
    }
}
#[test]
fn tab_stops_exclude_start_clamp_and_default_to_cursor() {
    let mut buffer = buffer(20, 3);
    assert_eq!(buffer.tabs.iter().copied().collect::<Vec<_>>(), [0, 8, 16]);
    for (x, previous, next) in [
        (0, 0, 8),
        (7, 0, 8),
        (8, 0, 16),
        (9, 8, 16),
        (16, 8, 19),
        (50, 16, 19),
    ] {
        assert_eq!(buffer.prev_stop(Some(x)), previous);
        assert_eq!(buffer.next_stop(Some(x)), next);
        buffer.x = x;
        assert_eq!(buffer.prev_stop(None), previous);
        assert_eq!(buffer.next_stop(None), next);
    }
    buffer
        .options
        .set_raw("tabStopWidth", OptionValue::Number(5.0))
        .unwrap();
    buffer.setup_tab_stops(Some(9)).unwrap();
    assert_eq!(
        buffer.tabs.iter().copied().collect::<Vec<_>>(),
        [0, 8, 13, 16, 18]
    );
    buffer.setup_tab_stops(None).unwrap();
    assert_eq!(
        buffer.tabs.iter().copied().collect::<Vec<_>>(),
        [0, 5, 10, 15]
    );
    buffer
        .options
        .set_raw("tabStopWidth", OptionValue::Number(0.0))
        .unwrap();
    assert!(matches!(
        buffer.setup_tab_stops(None),
        Err(BufferError::InvalidOption("tabStopWidth"))
    ));
}
#[test]
fn single_column_tabs_and_missing_translation() {
    let buffer = buffer(1, 1);
    assert_eq!((buffer.prev_stop(None), buffer.next_stop(None)), (0, 0));
    assert!(buffer
        .translate_buffer_line_to_string(0, true, None, None)
        .unwrap()
        .is_empty());
}
#[test]
fn deletion_disposes_half_open_range_and_shifts_only_later_markers() {
    let mut buffer = buffer(4, 5);
    let markers: Vec<_> = (0..5).map(|y| buffer.add_marker(y)).collect();
    buffer
        .lines
        .on_delete_emitter
        .fire(&ListChange {
            index: 1,
            amount: 2,
        })
        .unwrap();
    assert_eq!(
        markers
            .iter()
            .map(|m| (m.line(), m.is_disposed()))
            .collect::<Vec<_>>(),
        [(0, false), (-1, true), (-1, true), (1, false), (2, false)]
    );
    assert_eq!(buffer.markers.borrow().len(), 3);
}
#[test]
fn disposal_callback_can_dispose_another_marker_without_a_buffer_borrow() {
    let mut buffer = buffer(4, 5);
    let first = buffer.add_marker(1);
    let second = buffer.add_marker(1);
    let other = second.clone();
    let _subscription = first.on_dispose().try_subscribe(move |_| other.dispose());
    buffer.clear_markers(1).unwrap();
    assert!(first.is_disposed());
    assert!(second.is_disposed());
    assert!(buffer.markers.borrow().is_empty());
}
