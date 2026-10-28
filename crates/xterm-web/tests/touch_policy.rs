// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
// Scoped policy assertions and unchanged-source touch trace replay.
use xterm_web::input::touch::{EndGesture, Inertia, TapCount, TouchData};

fn number(value: &str) -> f64 {
    match value {
        "NaN" => f64::NAN,
        "Infinity" => f64::INFINITY,
        "-Infinity" => f64::NEG_INFINITY,
        _ => value.parse().unwrap(),
    }
}
fn equal_number(actual: f64, expected: &str, context: &str) {
    let expected = number(expected);
    assert!(
        actual.is_nan() && expected.is_nan() || actual.to_bits() == expected.to_bits(),
        "{context}: actual {actual:?}, expected {expected:?}"
    );
}

#[test]
fn start_and_translation_do_not_commit_before_dispatch() {
    let mut touch = TouchData::new("leaf", 1000.0, 10.0, 20.0);
    assert_eq!(touch.initial_target, "leaf");
    assert_eq!(touch.last(), (10.0, 20.0));
    assert_eq!(touch.translation(11.5, 17.0), (1.5, -3.0));
    assert_eq!(touch.last(), (10.0, 20.0));
    touch.commit_move(1001.0, 11.5, 17.0);
    assert_eq!(touch.last(), (11.5, 17.0));
}

#[test]
fn hold_and_distance_thresholds_are_strict_and_per_axis() {
    let mut touch = TouchData::new(1, 1000.0, 0.0, 0.0);
    touch.commit_move(1010.0, 29.999, -29.999);
    assert!(matches!(touch.end(1699.999, 2000.0), EndGesture::Tap));
    assert!(matches!(touch.end(1700.0, 2000.0), EndGesture::ContextMenu));
    assert!(matches!(
        touch.end(1700.001, 2000.0),
        EndGesture::ContextMenu
    ));
    for (x, y) in [(30.0, 0.0), (0.0, -30.0)] {
        touch.commit_move(1020.0, x, y);
        assert!(matches!(touch.end(1030.0, 2000.0), EndGesture::Inertia(_)));
    }
}

#[test]
fn rolling_velocity_uses_four_samples_but_hold_uses_initial_position() {
    let mut touch = TouchData::new(1, 0.0, 0.0, 0.0);
    for (time, x) in [
        (1.0, 10.0),
        (2.0, 20.0),
        (3.0, 30.0),
        (4.0, 40.0),
        (5.0, 55.0),
    ] {
        touch.commit_move(time, x, 0.0);
    }
    let EndGesture::Inertia(mut inertia) = touch.end(10.0, 10.0) else {
        panic!("movement classified as stationary")
    };
    let frame = inertia.step(11.0);
    assert_eq!(frame.translation_x, 35.0 / 3.0 - 0.005);
    assert_eq!(frame.translation_y, 0.0);
    assert!(frame.continues);
    let final_frame = inertia.step(10000.0);
    assert_eq!(
        (final_frame.translation_x, final_frame.translation_y),
        (0.0, 0.0)
    );
    assert!(!final_frame.continues);
}

#[test]
fn equal_and_nonfinite_clocks_retain_javascript_arithmetic() {
    let mut touch = TouchData::new(1, 0.0, 0.0, 0.0);
    touch.commit_move(0.0, 35.0, 0.0);
    let EndGesture::Inertia(mut inertia) = touch.end(10.0, 10.0) else {
        panic!("movement classified as stationary")
    };
    let frame = inertia.step(11.0);
    assert_eq!(frame.translation_x, f64::INFINITY);
    assert!(frame.continues);
    let frame = inertia.step(f64::INFINITY);
    assert_eq!((frame.translation_x, frame.translation_y), (0.0, 0.0));
    assert!(!frame.continues);
    let touch = TouchData::new(1, 0.0, 0.0, 0.0);
    let EndGesture::Inertia(mut inertia) = touch.end(f64::NAN, 0.0) else {
        panic!("NaN hold became a tap")
    };
    assert!(!inertia.step(16.0).continues);
}

#[test]
fn tap_counter_is_one_or_two_with_exact_source_reset_boundary() {
    let mut taps = TapCount::default();
    assert_eq!(taps.tap(0.0), 2);
    assert_eq!(taps.tap(400.0), 2);
    assert_eq!(taps.tap(801.0), 1);
    taps.reset();
    assert_eq!(taps.tap(400.0), 2);
    assert_eq!(taps.tap(f64::NAN), 2);
    assert_eq!(taps.tap(f64::INFINITY), 2);
    assert_eq!(taps.tap(f64::NEG_INFINITY), 2);
}

#[test]
fn captured_old_record_can_commit_after_active_record_replacement() {
    let mut old = TouchData::new(1, 0.0, 0.0, 0.0);
    let pending_delta = old.translation(35.0, 40.0);
    let replacement = TouchData::new(2, 1.0, 100.0, 200.0);
    old.commit_move(2.0, 35.0, 40.0);
    assert_eq!(pending_delta, (35.0, 40.0));
    assert_eq!(old.last(), (35.0, 40.0));
    assert_eq!(replacement.last(), (100.0, 200.0));
    assert_eq!(replacement.initial_target, 2);
}

#[test]
fn every_original_source_policy_snapshot_matches_rust() {
    let fixture = include_str!("fixtures/touch-policy.tsv");
    let mut snapshot_count = 0;
    let mut trace_count = 0;
    for (case, line) in fixture.lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 3);
        let start: Vec<_> = fields[0].split(',').map(number).collect();
        let mut touch = TouchData::new(7, start[0], start[1], start[2]);
        let mut taps = TapCount::default();
        let mut inertia: Option<Inertia> = None;
        let operations: Vec<_> = fields[1].split('|').collect();
        let expected: Vec<_> = fields[2].split('|').collect();
        assert_eq!(operations.len(), expected.len());
        for (step, (operation, expected)) in operations.iter().zip(expected).enumerate() {
            let context =
                format!("seed0x704c4b31 case{case} step{step} {operation}; fulltrace={line}");
            let (kind, args) = operation.split_once(':').unwrap_or((operation, ""));
            let args: Vec<_> = args
                .split(',')
                .filter(|s| !s.is_empty())
                .map(number)
                .collect();
            let values: Vec<_> = expected.split(':').collect();
            match kind {
                "m" => {
                    let delta = touch.translation(args[1], args[2]);
                    taps.reset();
                    touch.commit_move(args[0], args[1], args[2]);
                    assert_eq!(values[0], "M", "{context}");
                    let expected: Vec<_> = values[1].split(',').collect();
                    let last = touch.last();
                    for (actual, expected) in
                        [delta.0, delta.1, last.0, last.1].into_iter().zip(expected)
                    {
                        equal_number(actual, expected, &context);
                    }
                }
                "e" => {
                    let (classification, count) = match touch.end(args[0], args[1]) {
                        EndGesture::Tap => ("T", taps.tap(args[0])),
                        EndGesture::ContextMenu => {
                            taps.reset();
                            ("C", 0)
                        }
                        EndGesture::Inertia(next) => {
                            inertia = Some(next);
                            ("I", 0)
                        }
                    };
                    assert_eq!((values[0], values[1]), ("E", classification), "{context}");
                    let last = touch.last();
                    for (actual, expected) in [last.0, last.1].into_iter().zip(values[2].split(','))
                    {
                        equal_number(actual, expected, &context);
                    }
                    assert_eq!(count, values[3].parse::<u8>().unwrap(), "{context}");
                }
                "f" => {
                    assert_eq!(values[0], "F", "{context}");
                    if let Some(animation) = &mut inertia {
                        assert_ne!(values[1], "none", "{context}");
                        let frame = animation.step(args[0]);
                        for (actual, expected) in [frame.translation_x, frame.translation_y]
                            .into_iter()
                            .zip(values[1].split(','))
                        {
                            equal_number(actual, expected, &context);
                        }
                        assert_eq!(
                            u8::from(frame.continues),
                            values[2].parse::<u8>().unwrap(),
                            "{context}"
                        );
                        if !frame.continues {
                            inertia = None;
                        }
                    } else {
                        assert_eq!(values[1], "none", "{context}");
                    }
                }
                "t" => {
                    assert_eq!(values[0], "T");
                    assert_eq!(
                        taps.tap(args[0]),
                        values[1].parse::<u8>().unwrap(),
                        "{context}"
                    );
                }
                "r" => {
                    taps.reset();
                    assert_eq!(values, ["R"]);
                }
                _ => panic!("invalid fixture {context}"),
            }
            assert_eq!(touch.initial_target, 7);
            snapshot_count += 1;
        }
        trace_count += 1;
    }
    assert_eq!(trace_count, 639);
    assert_eq!(snapshot_count, 8061);
}
