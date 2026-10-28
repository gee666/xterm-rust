// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{cleanup::CleanupError, AttributeData, Buffer, BufferCleanup, BufferError},
    circular_list::ListChange,
    options::{OptionValue, OptionsService},
    task_queue::{CallbackKind, Scheduler},
};

fn buffer(cols: usize, rows: usize) -> Buffer {
    let mut buffer = Buffer::new(true, cols, rows, Rc::new(OptionsService::default())).unwrap();
    buffer.fill_viewport_rows(None).unwrap();
    buffer
}
#[derive(Debug)]
struct RequestDenied;
impl std::fmt::Display for RequestDenied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("request denied")
    }
}
impl std::error::Error for RequestDenied {}

#[derive(Default)]
struct HostState {
    next: u64,
    requested: Vec<CallbackKind>,
    cancelled: Vec<u64>,
    fail: bool,
}
#[derive(Clone, Default)]
struct Host(Rc<RefCell<HostState>>);
impl Scheduler for Host {
    type Handle = u64;
    type Error = RequestDenied;
    fn request(&self, kind: CallbackKind) -> Result<u64, Self::Error> {
        let mut state = self.0.borrow_mut();
        if state.fail {
            return Err(RequestDenied);
        }
        state.next += 1;
        state.requested.push(kind);
        Ok(state.next)
    }
    fn cancel(&self, handle: u64) {
        self.0.borrow_mut().cancelled.push(handle);
    }
    fn now(&self) -> f64 {
        0.0
    }
    fn deadline_exceeded(&self, _milliseconds: f64) -> Result<(), Self::Error> {
        panic!("fixed test deadline cannot overrun");
    }
}

#[test]
fn host_idle_cleanup_is_requested_cancelled_and_drained_without_sleep() {
    let shared = Rc::new(RefCell::new(buffer(10, 250)));
    let host = Host::default();
    let adapter = BufferCleanup::new(&shared, host.clone(), true).unwrap();
    assert_eq!(
        Rc::strong_count(&shared),
        1,
        "scheduled work must not own buffer"
    );
    shared.borrow_mut().resize(3, 250).unwrap();
    assert!(adapter.has_pending_callback());
    assert_eq!(host.0.borrow().requested, [CallbackKind::Idle]);
    shared.borrow_mut().resize(2, 250).unwrap();
    assert_eq!(host.0.borrow().cancelled, [1]);
    adapter.process_idle(|| 1000.0).unwrap();
    assert!(!adapter.has_pending_callback());
    assert!(!shared.borrow().has_pending_cleanup());
}
#[test]
fn cleanup_priority_fallback_and_dispose_cancel_the_actual_token() {
    let shared = Rc::new(RefCell::new(buffer(10, 3)));
    let host = Host::default();
    let adapter = BufferCleanup::new(&shared, host.clone(), false).unwrap();
    shared.borrow_mut().resize(3, 3).unwrap();
    assert_eq!(host.0.borrow().requested, [CallbackKind::Priority]);
    shared.borrow_mut().dispose().unwrap();
    assert!(!adapter.has_pending_callback());
    assert_eq!(host.0.borrow().cancelled, [1]);
    assert!(!shared.borrow().has_pending_cleanup());
}
#[test]
fn host_schedule_failure_propagates_with_typed_error_and_explicit_retry() {
    let shared = Rc::new(RefCell::new(buffer(10, 3)));
    let host = Host::default();
    let adapter = BufferCleanup::new(&shared, host.clone(), false).unwrap();
    host.0.borrow_mut().fail = true;
    let resize_error = shared.borrow_mut().resize(3, 3).unwrap_err();
    let BufferError::Dispatch(dispatch) = resize_error else {
        panic!("expected callback error");
    };
    assert!(dispatch.callback_ref::<RequestDenied>().is_some());
    let Some(CleanupError::ScheduledCallback(host_error)) = adapter.take_error() else {
        panic!("expected retained host error");
    };
    let xterm_core::event::EmitError::Callback(dispatched) = dispatch else {
        panic!("expected host callback object");
    };
    let retained: Rc<dyn std::error::Error> = host_error;
    assert!(Rc::ptr_eq(&dispatched, &retained));
    assert!(!adapter.has_pending_callback());
    assert!(shared.borrow().has_pending_cleanup());
    host.0.borrow_mut().fail = false;
    adapter.retry_pending().unwrap();
    adapter.process_priority().unwrap();
    assert!(!shared.borrow().has_pending_cleanup());
}
#[test]
fn cleanup_borrow_failure_is_not_hidden_or_retried_implicitly() {
    let shared = Rc::new(RefCell::new(buffer(10, 3)));
    let adapter = BufferCleanup::new(&shared, Host::default(), true).unwrap();
    shared.borrow_mut().resize(3, 3).unwrap();
    let held = shared.borrow_mut();
    assert!(matches!(
        adapter.process_idle(|| 1000.0),
        Err(CleanupError::Buffer(BufferError::BorrowedLine))
    ));
    drop(held);
    assert!(shared.borrow().has_pending_cleanup());
    adapter.retry_pending().unwrap();
    adapter.process_idle(|| 1000.0).unwrap();
    assert!(!shared.borrow().has_pending_cleanup());
}
#[test]
fn cleanup_owner_is_unique_and_drop_cancels_then_permits_replacement() {
    let shared = Rc::new(RefCell::new(buffer(10, 3)));
    let host = Host::default();
    let adapter = BufferCleanup::new(&shared, host.clone(), true).unwrap();
    assert!(matches!(
        BufferCleanup::new(&shared, Host::default(), true),
        Err(CleanupError::Buffer(BufferError::CleanupAlreadyAttached))
    ));
    shared.borrow_mut().resize(3, 3).unwrap();
    drop(adapter);
    assert_eq!(host.0.borrow().cancelled, [1]);
    let adapter = BufferCleanup::new(&shared, Host::default(), true).unwrap();
    assert!(adapter.has_pending_callback());
    drop(shared);
    adapter.process_idle(|| 1000.0).unwrap();
    assert!(!adapter.has_pending_callback());
}
#[test]
fn cleanup_batches_preserve_source_101_cleaned_rows_and_final_rescan() {
    let mut buffer = buffer(10, 250);
    buffer.resize(3, 250).unwrap();
    assert!(buffer.poll_cleanup().unwrap());
    assert!(buffer.poll_cleanup().unwrap());
    assert!(buffer.poll_cleanup().unwrap());
    assert!(!buffer.poll_cleanup().unwrap());
    assert!(!buffer.poll_cleanup().unwrap());
}
#[test]
fn tab_setup_preserves_incremental_stops_and_rebuilds_current_option() {
    let mut buffer = buffer(10, 2);
    assert_eq!(buffer.tabs.iter().copied().collect::<Vec<_>>(), [0, 8]);
    assert_eq!(buffer.prev_stop(Some(8)), 0);
    assert_eq!(buffer.prev_stop(Some(9)), 8);
    assert_eq!(buffer.next_stop(Some(0)), 8);
    assert_eq!(buffer.next_stop(Some(8)), 9);
    assert_eq!(buffer.next_stop(Some(99)), 9);
    buffer
        .options
        .set_raw("tabStopWidth", OptionValue::Number(3.0))
        .unwrap();
    buffer.setup_tab_stops(Some(7)).unwrap();
    assert_eq!(
        buffer.tabs.iter().copied().collect::<Vec<_>>(),
        [0, 3, 6, 8, 9]
    );
    buffer.setup_tab_stops(None).unwrap();
    assert_eq!(
        buffer.tabs.iter().copied().collect::<Vec<_>>(),
        [0, 3, 6, 9]
    );
    buffer
        .options
        .set_raw("tabStopWidth", OptionValue::Number(0.0))
        .unwrap();
    assert!(matches!(
        buffer.setup_tab_stops(None),
        Err(BufferError::InvalidOption("tabStopWidth"))
    ));
    assert_eq!(
        buffer.tabs.iter().copied().collect::<Vec<_>>(),
        [0, 3, 6, 9]
    );
}
#[test]
fn clear_preserves_saved_state_but_resets_cursor_and_scrolling() {
    let mut buffer = buffer(10, 3);
    buffer.saved_x = 7;
    buffer.saved_y = 12;
    buffer.saved_glevel = 2;
    buffer.saved_wraparound_mode = false;
    buffer.saved_origin_mode = true;
    buffer.x = 5;
    buffer.y = 2;
    buffer.ybase = 10;
    buffer.ydisp = 9;
    buffer.scroll_top = 1;
    buffer.clear().unwrap();
    assert!(buffer.lines.is_empty());
    assert_eq!(
        (
            buffer.x,
            buffer.y,
            buffer.ybase,
            buffer.ydisp,
            buffer.scroll_top,
            buffer.scroll_bottom
        ),
        (0, 0, 0, 0, 0, 2)
    );
    assert_eq!(
        (buffer.saved_x, buffer.saved_y, buffer.saved_glevel),
        (7, 12, 2)
    );
    assert!(!buffer.saved_wraparound_mode);
    assert!(buffer.saved_origin_mode);
}
#[test]
fn real_marker_insert_delete_and_clear_one_line_preserve_identity_and_events() {
    let mut buffer = buffer(10, 4);
    let first = buffer.add_marker(0);
    let middle = buffer.add_marker(2);
    let last = buffer.add_marker(3);
    buffer
        .lines
        .on_insert_emitter
        .fire(&ListChange {
            index: 2,
            amount: 2,
        })
        .unwrap();
    assert_eq!(
        (
            first.line_number(),
            middle.line_number(),
            last.line_number()
        ),
        (0.0, 4.0, 5.0)
    );
    buffer
        .lines
        .on_delete_emitter
        .fire(&ListChange {
            index: 4,
            amount: 1,
        })
        .unwrap();
    assert!(middle.is_disposed());
    assert_eq!(middle.line_number(), -1.0);
    assert_eq!(last.line_number(), 4.0);
    buffer.clear_markers(0).unwrap();
    assert!(first.is_disposed());
    assert_eq!(buffer.markers.borrow().len(), 1);
    assert!(buffer.markers.borrow()[0].ptr_eq(&last));
    buffer.clear_all_markers().unwrap();
    assert!(last.is_disposed());
    assert!(buffer.markers.borrow().is_empty());
}
#[test]
fn line_borrow_contention_reports_error_and_viewport_checks_do_not_overflow() {
    let mut buffer = buffer(10, 3);
    let line = buffer.lines.get(0).unwrap();
    let held = line.borrow_mut();
    assert!(matches!(
        buffer.translate_buffer_line_to_string(0, false, None, None),
        Err(BufferError::BorrowedLine)
    ));
    drop(held);
    buffer.ybase = usize::MAX;
    buffer.y = 1;
    assert!(!buffer.is_cursor_in_viewport());
    buffer.ybase = 2;
    buffer.y = 1;
    buffer.ydisp = 1;
    assert!(buffer.is_cursor_in_viewport());
    buffer.ydisp = 4;
    assert!(!buffer.is_cursor_in_viewport());
}
#[test]
fn scratch_cells_preserve_width_code_and_clear_unspecified_attributes() {
    let buffer = buffer(10, 3);
    let attrs = AttributeData {
        fg: 123,
        bg: 456,
        ..Default::default()
    };
    let null = buffer.get_null_cell(Some(&attrs));
    assert_eq!(
        (null.get_code(), null.get_width(), null.fg, null.bg),
        (Some(0), 1, 123, 456)
    );
    let space = buffer.get_whitespace_cell(Some(&attrs));
    assert_eq!(
        (space.get_code(), space.get_width(), space.fg, space.bg),
        (Some(32), 1, 123, 456)
    );
    let null = buffer.get_null_cell(None);
    assert_eq!((null.fg, null.bg), (0, 0));
}
