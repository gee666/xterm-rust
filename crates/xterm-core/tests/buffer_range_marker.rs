// Copyright (c) 2018-2021 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Range assertions ported from BufferRange.test.ts.
use std::cell::{Cell, RefCell};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;
use xterm_core::buffer::marker::Marker;
use xterm_core::buffer::range::{get_range_length, BufferPosition, BufferRange};
use xterm_core::event::EmitError;
use xterm_core::lifecycle::{to_disposable, DisposableHandle};

fn range(x1: i32, y1: i32, x2: i32, y2: i32) -> BufferRange {
    BufferRange {
        start: BufferPosition {
            x: f64::from(x1),
            y: f64::from(y1),
        },
        end: BufferPosition {
            x: f64::from(x2),
            y: f64::from(y2),
        },
    }
}

#[test]
fn range_single_line() {
    assert_eq!(get_range_length(range(1, 1, 4, 1), 0), Ok(4.0));
}
#[test]
fn range_invalid_row_order() {
    assert!(get_range_length(range(1, 3, 1, 1), 0).is_err());
}
#[test]
fn range_multiple_lines() {
    assert_eq!(get_range_length(range(1, 1, 4, 5), 5), Ok(24.0));
}
#[test]
fn range_next_line_out_of_width() {
    assert_eq!(get_range_length(range(1, 1, 7, 2), 5), Ok(12.0));
}

#[test]
fn range_signed_boundaries_and_exact_error() {
    assert_eq!(get_range_length(range(2, 1, 1, 1), 0), Ok(0.0));
    assert_eq!(get_range_length(range(4, 1, 1, 1), 0), Ok(-2.0));
    assert_eq!(get_range_length(range(-4, -2, -1, -1), -5), Ok(-1.0));
    assert_eq!(
        get_range_length(range(1, 3, 1, 1), 0)
            .unwrap_err()
            .to_string(),
        "Buffer range end (1, 1) cannot be before start (1, 3)"
    );
    let min = i32::MIN;
    let max = i32::MAX;
    assert_eq!(
        get_range_length(range(min, min, max, max), max),
        Ok(9223372034707292160.0)
    );
}

#[test]
fn range_js_differential_fixture() {
    for (line, row) in include_str!("fixtures/buffer-range-oracle.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = row.split('\t').collect();
        let values: Vec<i32> = fields[..5].iter().map(|v| v.parse().unwrap()).collect();
        let actual = get_range_length(range(values[0], values[1], values[2], values[3]), values[4]);
        if let Some(message) = fields[5].strip_prefix("error:") {
            assert_eq!(actual.unwrap_err().to_string(), message, "line {line}");
        } else {
            assert_eq!(
                actual.unwrap().to_bits(),
                fields[5].parse::<f64>().unwrap().to_bits(),
                "line {line}"
            );
        }
    }
}

#[test]
fn range_all_javascript_number_boundaries_match_the_original() {
    let number = |text: &str| match text {
        "NaN" => f64::NAN,
        "Infinity" => f64::INFINITY,
        "-Infinity" => f64::NEG_INFINITY,
        _ => text.parse::<f64>().unwrap(),
    };
    for (index, row) in include_str!("fixtures/buffer-range-number-oracle.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = row.split('\t').collect();
        let values: Vec<_> = fields[..5].iter().map(|value| number(value)).collect();
        let range = BufferRange {
            start: BufferPosition {
                x: values[0],
                y: values[1],
            },
            end: BufferPosition {
                x: values[2],
                y: values[3],
            },
        };
        let actual = get_range_length(range, values[4]);
        if let Some(expected) = fields[5].strip_prefix("error:") {
            assert_eq!(
                actual.unwrap_err().to_string(),
                expected,
                "numeric row{index}"
            );
        } else {
            let actual = actual.unwrap();
            let expected = number(fields[5]);
            if expected.is_nan() {
                assert!(actual.is_nan(), "numeric row{index}");
            } else {
                assert_eq!(actual.to_bits(), expected.to_bits(), "numeric row{index}");
            }
        }
    }
}

#[test]
fn marker_clones_share_identity_line_and_state() {
    let first = Marker::new(12);
    let clone = first.clone();
    let second = Marker::new(12);
    assert!(first.id() > 0);
    assert!(second.id() > first.id());
    assert_eq!(first.id(), clone.id());
    assert!(first.ptr_eq(&clone));
    assert!(!first.ptr_eq(&second));
    clone.set_line(i64::from(u32::MAX));
    assert_eq!(first.line_number(), f64::from(u32::MAX));
    clone.set_line(13);
    assert_eq!(first.line_number(), 13.0);
    first.dispose().unwrap();
    assert!(clone.is_disposed());
    assert_eq!(clone.line_number(), -1.0);
    clone.set_line(19);
    clone.dispose().unwrap();
    assert_eq!(first.line_number(), 19.0);
}

#[test]
fn marker_trace_matches_javascript() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let marker = Marker::new(7);
    let weak = marker.downgrade();
    let log_event = Rc::clone(&log);
    marker.on_dispose().subscribe(move |_| {
        let marker = weak.upgrade().unwrap();
        log_event.borrow_mut().push(format!(
            "event:{}:{}",
            marker.is_disposed(),
            marker.line_number()
        ));
        marker.dispose().unwrap();
        let log_late = Rc::clone(&log_event);
        marker.register(to_disposable(move || {
            log_late.borrow_mut().push("during-event".into())
        }));
    });
    let log_resource = Rc::clone(&log);
    let duplicate = to_disposable(move || log_resource.borrow_mut().push("resource".into()));
    assert!(Rc::ptr_eq(
        &duplicate,
        &marker.register(Rc::clone(&duplicate))
    ));
    marker.register(duplicate);
    marker.dispose().unwrap();
    marker.dispose().unwrap();
    let late_log = Rc::clone(&log);
    marker.register(to_disposable(move || {
        late_log.borrow_mut().push("late".into())
    }));
    marker
        .on_dispose()
        .subscribe(|_| panic!("disposed emitter fired"));
    marker.dispose().unwrap();
    assert_eq!(
        log.borrow().join("\n") + "\n",
        include_str!("fixtures/marker-trace.txt")
    );
}

#[test]
fn marker_resource_traversal_is_live_and_has_no_callback_borrow() {
    let marker = Marker::new(1);
    let log = Rc::new(RefCell::new(Vec::new()));
    let weak = marker.downgrade();
    let resource_log = Rc::clone(&log);
    marker.register(to_disposable(move || {
        resource_log.borrow_mut().push("first");
        let marker = weak.upgrade().unwrap();
        marker.dispose().unwrap();
        let late_log = Rc::clone(&resource_log);
        marker.register(to_disposable(move || {
            late_log.borrow_mut().push("appended")
        }));
    }));
    let second_log = Rc::clone(&log);
    marker.register(to_disposable(move || {
        second_log.borrow_mut().push("second")
    }));
    marker.dispose().unwrap();
    assert_eq!(&*log.borrow(), &["first", "second", "appended"]);
}

#[test]
fn marker_listener_failure_stops_cleanup_and_does_not_retry() {
    let marker = Marker::new(2);
    let reached = Rc::new(Cell::new(false));
    marker
        .on_dispose()
        .try_subscribe(|_| Err(EmitError::ListenerArrayCleared));
    let reached_event = Rc::clone(&reached);
    marker
        .on_dispose()
        .subscribe(move |_| reached_event.set(true));
    let reached_resource = Rc::clone(&reached);
    marker.register(to_disposable(move || reached_resource.set(true)));
    assert_eq!(marker.dispose(), Err(EmitError::ListenerArrayCleared));
    assert!(marker.is_disposed());
    assert_eq!(marker.line_number(), -1.0);
    assert!(!reached.get());
    marker.dispose().unwrap();
    assert!(!reached.get());
}

#[test]
fn marker_propagates_original_callback_error_identity() {
    let marker = Marker::new(2);
    let original = EmitError::callback(std::io::Error::other("callback sentinel"));
    let callback_error = original.clone();
    marker
        .on_dispose()
        .try_subscribe(move |_| Err(callback_error.clone()));
    let returned = marker.dispose().unwrap_err();
    assert_eq!(returned, original);
    assert_eq!(
        returned
            .callback_ref::<std::io::Error>()
            .unwrap()
            .to_string(),
        "callback sentinel"
    );
    assert!(marker.is_disposed());
    assert_eq!(marker.line_number(), -1.0);
}

#[test]
fn marker_listener_and_resource_panics_preserve_invalidation_without_borrow_panics() {
    for event_panic in [false, true] {
        let marker = Marker::new(3);
        let weak = marker.downgrade();
        let reached = Rc::new(Cell::new(false));
        if event_panic {
            marker
                .on_dispose()
                .subscribe(|_| panic!("listener failure"));
        } else {
            marker.register(to_disposable(move || {
                weak.upgrade().unwrap().register(to_disposable(|| {}));
                panic!("resource failure");
            }));
        }
        let cleanup_reached = Rc::clone(&reached);
        marker.register(to_disposable(move || cleanup_reached.set(true)));
        assert!(catch_unwind(AssertUnwindSafe(|| marker.dispose())).is_err());
        assert!(marker.is_disposed());
        assert_eq!(marker.line_number(), -1.0);
        assert!(!reached.get());
        marker.dispose().unwrap();
        assert!(!reached.get());
    }
}

#[test]
fn marker_listener_removal_uses_event_snapshot() {
    let marker = Marker::new(4);
    let second = Rc::new(RefCell::new(None::<DisposableHandle>));
    let handle = Rc::clone(&second);
    marker
        .on_dispose()
        .subscribe(move |_| handle.borrow().as_ref().unwrap().dispose());
    let fired = Rc::new(Cell::new(false));
    let fired_callback = Rc::clone(&fired);
    *second.borrow_mut() = Some(
        marker
            .on_dispose()
            .subscribe(move |_| fired_callback.set(true)),
    );
    marker.dispose().unwrap();
    assert!(fired.get());
}

#[test]
fn marker_drop_releases_weak_resources_without_invoking_callbacks() {
    let called = Rc::new(Cell::new(false));
    let marker = Marker::new(5);
    let weak = marker.downgrade();
    let callback_weak = weak.clone();
    let callback_called = Rc::clone(&called);
    marker.register(to_disposable(move || {
        assert!(callback_weak.upgrade().is_some());
        callback_called.set(true);
    }));
    let clone = marker.clone();
    drop(marker);
    assert!(weak.upgrade().is_some());
    drop(clone);
    assert!(weak.upgrade().is_none());
    assert!(!called.get());
}
