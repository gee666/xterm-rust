// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::rc::Rc;
use xterm_core::{
    buffer::{Buffer, BufferError, BufferService},
    options::{OptionValue as V, OptionsService},
};
fn options() -> Rc<OptionsService> {
    let options = Rc::new(OptionsService::default());
    for (key, value) in [("cols", 6.0), ("rows", 2.0), ("scrollback", 3.0)] {
        options.set_raw(key, V::Number(value)).unwrap();
    }
    options
}
#[test]
fn accepted_live_nan_keeps_capacity_and_does_not_throw() {
    let options = options();
    let service = BufferService::new(options.clone()).unwrap();
    let normal = service.buffers.normal();
    let alt = service.buffers.alt();
    assert_eq!(normal.borrow().lines.max_length(), 5);
    options.set("scrollback", V::Number(f64::NAN)).unwrap();
    assert!(options
        .get("scrollback")
        .unwrap()
        .to_number()
        .unwrap()
        .is_nan());
    assert_eq!(normal.borrow().lines.max_length(), 5);
    assert_eq!(normal.borrow().lines.len(), 2);
    assert_eq!(alt.borrow().lines.max_length(), 2);
    service.resize(8, 4).unwrap();
    assert_eq!(normal.borrow().lines.max_length(), 5);
    service.resize(3, 1).unwrap();
    assert_eq!(normal.borrow().lines.max_length(), 5);
}
#[test]
fn nan_construction_has_source_array_length_error_but_alternate_does_not() {
    let options = options();
    options.set_raw("scrollback", V::Number(f64::NAN)).unwrap();
    let error = Buffer::new(true, 6, 2, options.clone()).err().unwrap();
    assert!(matches!(error, BufferError::InvalidArrayLength));
    assert_eq!(error.to_string(), "Invalid array length");
    assert_eq!(
        Buffer::new(false, 6, 2, options)
            .unwrap()
            .lines
            .max_length(),
        2
    );
}
#[test]
fn clear_fails_after_cursor_reset_without_replacing_existing_lines() {
    let options = options();
    let mut buffer = Buffer::new(true, 6, 2, options.clone()).unwrap();
    buffer.fill_viewport_rows(None).unwrap();
    let first = buffer.lines.get(0).unwrap();
    buffer.x = 3;
    buffer.y = 1;
    buffer.ybase = 3;
    buffer.ydisp = 2;
    buffer.scroll_top = 1;
    buffer.saved_y = 9;
    options.set_raw("scrollback", V::Number(f64::NAN)).unwrap();
    assert!(matches!(
        buffer.clear(),
        Err(BufferError::InvalidArrayLength)
    ));
    assert_eq!(
        (buffer.x, buffer.y, buffer.ybase, buffer.ydisp),
        (0, 0, 0, 0)
    );
    assert_eq!(buffer.scroll_top, 1);
    assert_eq!(buffer.saved_y, 9);
    assert_eq!(buffer.lines.len(), 2);
    assert_eq!(buffer.lines.max_length(), 5);
    assert!(Rc::ptr_eq(&first, &buffer.lines.get(0).unwrap()));
}
#[test]
fn reset_constructor_failure_preserves_raw_buffers_and_scroll_flag() {
    let options = options();
    let service = BufferService::new(options.clone()).unwrap();
    let normal = service.buffers.normal();
    let alt = service.buffers.alt();
    let marker = normal.borrow_mut().add_marker(0);
    options.set("scrollback", V::Number(f64::NAN)).unwrap();
    service.is_user_scrolling.set(true);
    assert!(matches!(
        service.reset(),
        Err(BufferError::InvalidArrayLength)
    ));
    assert!(Rc::ptr_eq(&normal, &service.buffers.normal()));
    assert!(Rc::ptr_eq(&alt, &service.buffers.alt()));
    assert!(service.is_user_scrolling.get());
    assert!(!marker.is_disposed());
}
