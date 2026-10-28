// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Faithful assertions from common/services/BufferService.test.ts.
use std::{cell::Cell, rc::Rc};
use xterm_core::{
    buffer::{AttributeData, BufferService},
    options::{OptionValue, OptionsService},
};

fn service(cols: usize, rows: usize, scrollback: usize) -> BufferService {
    let options = [("cols", cols), ("rows", rows), ("scrollback", scrollback)]
        .map(|(key, value)| (key, OptionValue::Number(value as f64)));
    BufferService::new(Rc::new(OptionsService::new(false, options))).unwrap()
}
#[test]
fn full_scrolled_buffer_decrements_display_without_changing_base() {
    let service = service(10, 3, 2);
    let active = service.buffer();
    let attr = AttributeData::default();
    while !active.borrow().lines.is_full() {
        service.scroll(&attr, false).unwrap();
    }
    assert_eq!(active.borrow().lines.len(), 5);
    service.is_user_scrolling.set(true);
    active.borrow_mut().ydisp = 2;
    let ybase = active.borrow().ybase;
    service.scroll(&attr, false).unwrap();
    assert_eq!(active.borrow().ybase, ybase);
    assert_eq!(active.borrow().ydisp, 1);
}
#[test]
fn nonfull_scrolled_buffer_does_not_advance_display() {
    let service = service(10, 3, 2);
    let active = service.buffer();
    service.is_user_scrolling.set(true);
    active.borrow_mut().ydisp = 0;
    let ybase = active.borrow().ybase;
    service.scroll(&AttributeData::default(), false).unwrap();
    assert_eq!(active.borrow().ybase, ybase + 1);
    assert_eq!(active.borrow().ydisp, 0);
}
#[test]
fn tracking_display_follows_base_when_full() {
    let service = service(10, 3, 2);
    let active = service.buffer();
    let attr = AttributeData::default();
    while !active.borrow().lines.is_full() {
        service.scroll(&attr, false).unwrap();
    }
    service.is_user_scrolling.set(false);
    service.scroll(&attr, false).unwrap();
    assert_eq!(active.borrow().ydisp, active.borrow().ybase);
}
#[test]
fn scrolling_within_margins_keeps_outside_rows() {
    let service = service(10, 5, 10);
    let active = service.buffer();
    let attr = AttributeData::default();
    for (row, ch) in ['A', 'B', 'C', 'D', 'E'].into_iter().enumerate() {
        let line = active.borrow().lines.get(row).unwrap();
        line.borrow_mut()
            .set_cell_from_codepoint(0, ch as u32, 1, &attr);
    }
    active.borrow_mut().scroll_top = 1;
    active.borrow_mut().scroll_bottom = 3;
    service.scroll(&attr, false).unwrap();
    for (row, expected) in ["A", "C", "D", "", "E"].into_iter().enumerate() {
        let line = active.borrow().lines.get(row).unwrap();
        assert_eq!(
            line.borrow().translate_to_string(true, None, None, None),
            expected.encode_utf16().collect::<Vec<_>>()
        );
    }
}
#[test]
fn scroll_up_changes_display_and_emits_position() {
    let service = service(80, 10, 10);
    let active = service.buffer();
    active.borrow_mut().ybase = 5;
    active.borrow_mut().ydisp = 5;
    let emitted = Rc::new(Cell::new(None));
    let seen = Rc::clone(&emitted);
    service
        .on_scroll()
        .subscribe(move |value| seen.set(Some(*value)));
    service.scroll_lines(-2, false).unwrap();
    assert_eq!(active.borrow().ydisp, 3);
    assert!(service.is_user_scrolling.get());
    assert_eq!(emitted.get(), Some(3));
}
#[test]
fn scroll_up_at_top_leaves_scroll_flag_false() {
    let service = service(80, 10, 10);
    let active = service.buffer();
    active.borrow_mut().ybase = 5;
    active.borrow_mut().ydisp = 0;
    service.scroll_lines(-1, false).unwrap();
    assert_eq!(active.borrow().ydisp, 0);
    assert!(!service.is_user_scrolling.get());
}
#[test]
fn scroll_to_bottom_clears_user_scrolling() {
    let service = service(80, 10, 10);
    let active = service.buffer();
    active.borrow_mut().ybase = 5;
    active.borrow_mut().ydisp = 2;
    service.is_user_scrolling.set(true);
    service.scroll_lines(10, false).unwrap();
    assert_eq!(active.borrow().ydisp, 5);
    assert!(!service.is_user_scrolling.get());
}
