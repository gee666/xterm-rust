use xterm_web::input::touch::{EndGesture, TouchData};

#[test]
fn initial_and_first_rolling_getter_values_remain_distinct() {
    let data = TouchData::from_snapshots(7, 1000.0, (0.0, 0.0), (29.0, 29.0));
    assert_eq!(data.last(), (29.0, 29.0));
    assert!(matches!(data.end(1699.0, 1700.0), EndGesture::Tap));
    assert!(matches!(data.end(1700.0, 1700.0), EndGesture::ContextMenu));
    let data = TouchData::from_snapshots(7, 1000.0, (0.0, 0.0), (30.0, 0.0));
    assert!(matches!(data.end(1001.0, 1001.0), EndGesture::Inertia(_)));
}

#[test]
fn nested_commit_can_grow_history_beyond_four_source_samples() {
    let mut data = TouchData::new(7, 0.0, 0.0, 0.0);
    data.commit_move(1.0, 10.0, 0.0);
    data.commit_move(2.0, 20.0, 0.0);
    data.begin_commit();
    // A coordinate getter runs a nested complete move after the outer length
    // check but before its push, so source retains five entries rather than four.
    data.commit_move(3.0, 30.0, 0.0);
    data.push_x(100.0);
    data.push_y(0.0);
    data.push_time(4.0);
    let EndGesture::Inertia(mut inertia) = data.end(5.0, 5.0) else {
        panic!("movement became stationary")
    };
    assert_eq!(inertia.step(6.0).translation_x, 25.0 - 0.005);
    // Later normal commits shift once, not until the vector fits a guessed cap.
    data.commit_move(6.0, 130.0, 0.0);
    let EndGesture::Inertia(mut inertia) = data.end(7.0, 7.0) else {
        panic!("movement became stationary")
    };
    assert_eq!(inertia.step(8.0).translation_x, 120.0 / 5.0 - 0.005);
}

#[test]
fn partial_pushes_preserve_unequal_and_empty_array_tails() {
    let mut data = TouchData::new(7, 0.0, 0.0, 0.0);
    for time in 1..=3 {
        data.commit_move(f64::from(time), f64::from(time), f64::from(time));
    }
    for x in 10..14 {
        data.begin_commit();
        data.push_x(f64::from(x));
    }
    assert_eq!(data.last_x(), Some(13.0));
    assert_eq!(data.last_y(), None);
    assert!(data.last().1.is_nan());
    let EndGesture::Inertia(mut inertia) = data.end(10.0, 10.0) else {
        panic!("undefined tail became a tap")
    };
    let frame = inertia.step(11.0);
    assert_eq!((frame.translation_x, frame.translation_y), (0.0, 0.0));
    assert!(!frame.continues);
    data.push_y(99.0);
    data.push_time(12.0);
    assert_eq!(data.last_y(), Some(99.0));
}
