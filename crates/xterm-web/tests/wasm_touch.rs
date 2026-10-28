#![cfg(target_arch = "wasm32")]
#[path = "support/touch_dom.rs"]
mod support;
use support::{get, has, number, send_batch, Setup};
#[path = "cases/touch_reentry.rs"]
mod reentry;
use wasm_bindgen_test::*;
use xterm_web::dom_listener::DomListener;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn nested_duplicate_ignore_targets_and_exact_event_fields() {
    let mut setup = Setup::new();
    setup.track(false);
    setup.track(true);
    let outer = setup.context.add_target(&setup.parent).unwrap();
    let inner = setup.context.add_target(&setup.child).unwrap();
    let duplicate = setup.context.add_target(&setup.child).unwrap();
    let source = setup.send("touchstart", 1, 3.0, 4.0);
    assert!(source.default_prevented());
    {
        let log = setup.log.borrow();
        assert_eq!(
            log.iter().map(|entry| entry.0.as_str()).collect::<Vec<_>>(),
            ["child", "child", "parent"]
        );
        for (_, event) in log.iter() {
            assert!(!event.bubbles());
            assert!(event.cancelable());
            assert_eq!(number(event, "pageX"), 3.0);
            assert_eq!(number(event, "tapCount"), 0.0);
            assert!(!has(event, "translationX"));
            assert!(!has(event, "clientX"));
        }
    }
    setup.clear();
    setup.time(1010.0);
    setup.send("touchmove", 1, 13.5, 24.25);
    {
        let log = setup.log.borrow();
        assert_eq!(log.len(), 3);
        assert_eq!(number(&log[0].1, "translationX"), 10.5);
        assert_eq!(number(&log[0].1, "translationY"), 20.25);
        assert_eq!(number(&log[0].1, "clientX"), 14.5);
    }
    setup.clear();
    let ignore = setup.context.ignore_target(&setup.child).unwrap();
    assert!(!setup.send("touchmove", 1, 14.0, 25.0).default_prevented());
    assert!(setup.log.borrow().is_empty());
    ignore.dispose();
    ignore.dispose();
    inner.dispose();
    inner.dispose();
    setup.send("touchstart", 1, 2.0, 3.0);
    assert_eq!(
        setup
            .log
            .borrow()
            .iter()
            .map(|entry| entry.0.as_str())
            .collect::<Vec<_>>(),
        ["child", "parent"]
    );
    duplicate.dispose();
    outer.dispose();
    setup.clear();
    assert!(!setup.send("touchstart", 1, 0.0, 0.0).default_prevented());
    assert!(setup.log.borrow().is_empty());
}

#[wasm_bindgen_test]
fn tap_hold_thresholds_ignore_state_and_end_coordinates_match_source() {
    let mut setup = Setup::new();
    setup.track(true);
    let target = setup.context.add_target(&setup.child).unwrap();
    for (time, expected) in [(1000.0, 1.0), (1400.0, 2.0), (1801.0, 1.0)] {
        setup.clear();
        setup.time(time);
        setup.send("touchstart", 1, 1.0, 2.0);
        setup.send("touchend", 1, 999.0, 999.0);
        let log = setup.log.borrow();
        assert_eq!(log.len(), 3);
        assert_eq!(log[1].1.type_(), "-xterm-gesturetap");
        assert_eq!(number(&log[1].1, "tapCount"), expected);
        assert_eq!(number(&log[1].1, "pageX"), 1.0);
        assert!(!has(&log[2].1, "pageX"));
    }
    setup.clear();
    setup.time(2000.0);
    setup.send("touchstart", 1, 0.0, 0.0);
    setup.time(2700.0);
    setup.send("touchend", 1, 0.0, 0.0);
    assert_eq!(setup.log.borrow()[1].1.type_(), "-xterm-gesturecontextmenu");
    // Ignored taps still change the global tap timestamp before filtering.
    let ignored = setup.context.ignore_target(&setup.child).unwrap();
    setup.time(3000.0);
    setup.send("touchstart", 1, 0.0, 0.0);
    setup.send("touchend", 1, 0.0, 0.0);
    ignored.dispose();
    setup.clear();
    setup.time(3400.0);
    setup.send("touchstart", 1, 0.0, 0.0);
    setup.send("touchend", 1, 0.0, 0.0);
    assert_eq!(number(&setup.log.borrow()[1].1, "tapCount"), 2.0);
    target.dispose();
}

#[wasm_bindgen_test]
fn inertia_bypasses_ignore_and_depth_sort_and_emits_final_zero_frame() {
    let mut setup = Setup::new();
    setup.track(false);
    setup.track(true);
    let outer = setup.context.add_target(&setup.parent).unwrap();
    let inner = setup.context.add_target(&setup.child).unwrap();
    setup.send("touchstart", 1, 0.0, 0.0);
    setup.time(1010.0);
    setup.send("touchmove", 1, 100.0, 0.0);
    setup.time(1020.0);
    setup.send("touchend", 1, 100.0, 0.0);
    assert_eq!(setup.pending(), 1.0);
    let ignore = setup.context.ignore_target(&setup.child).unwrap();
    setup.clear();
    setup.time(1030.0);
    setup.flush();
    {
        let log = setup.log.borrow();
        assert_eq!(
            log.iter().map(|entry| entry.0.as_str()).collect::<Vec<_>>(),
            ["parent", "child"]
        );
        assert!((number(&log[0].1, "translationX") - 99.5).abs() < 1e-10);
        assert_eq!(number(&log[0].1, "translationY"), 0.0);
        assert!(get(&log[0].1, "initialTarget").is_undefined());
        assert!(!has(&log[0].1, "pageX"));
        assert!(!has(&log[0].1, "clientX"));
    }
    assert_eq!(setup.pending(), 1.0);
    setup.clear();
    setup.time(10000.0);
    setup.flush();
    assert_eq!(setup.pending(), 0.0);
    assert_eq!(number(&setup.log.borrow()[0].1, "translationX"), 0.0);
    ignore.dispose();
    inner.dispose();
    outer.dispose();
}

#[wasm_bindgen_test]
fn ending_multiple_touches_retains_the_initial_active_count_snapshot() {
    let mut setup = Setup::new();
    setup.track(true);
    let token = setup.context.add_target(&setup.child).unwrap();
    send_batch(&setup.leaf, "touchstart", &[(1, 0.0, 0.0), (2, 0.0, 0.0)]);
    setup.time(1010.0);
    send_batch(&setup.leaf, "touchmove", &[(1, 30.0, 0.0), (2, 40.0, 0.0)]);
    setup.clear();
    setup.time(1020.0);
    send_batch(&setup.leaf, "touchend", &[(1, 30.0, 0.0), (2, 40.0, 0.0)]);
    assert_eq!(setup.pending(), 0.0);
    assert_eq!(setup.log.borrow().len(), 2);
    assert!(setup
        .log
        .borrow()
        .iter()
        .all(|entry| entry.1.type_() == "-xterm-gesturesend"));
    token.dispose();
}

#[wasm_bindgen_test]
fn start_cancels_inertia_without_canceling_the_shared_frame() {
    let mut setup = Setup::new();
    setup.track(true);
    let target = setup.context.add_target(&setup.child).unwrap();
    setup.send("touchstart", 1, 0.0, 0.0);
    setup.time(1010.0);
    setup.send("touchmove", 1, 30.0, 0.0);
    setup.time(1020.0);
    setup.send("touchend", 1, 30.0, 0.0);
    assert_eq!(setup.pending(), 1.0);
    setup.send("touchstart", 2, 0.0, 0.0);
    setup.clear();
    setup.flush();
    assert!(setup.log.borrow().is_empty());
    assert_eq!(setup.pending(), 0.0);
    target.dispose();
}

#[wasm_bindgen_test]
fn target_disposal_during_dispatch_keeps_snapshot_and_context_disposal_removes_listeners() {
    let mut setup = Setup::new();
    setup.track(false);
    setup.track(true);
    let outer = setup.context.add_target(&setup.parent).unwrap();
    let inner = setup.context.add_target(&setup.child).unwrap();
    let captured = outer.clone();
    let listener = DomListener::new(
        setup.child.as_ref(),
        "-xterm-gesturestart",
        false,
        move |_| captured.dispose(),
    )
    .unwrap();
    setup.send("touchstart", 1, 0.0, 0.0);
    assert_eq!(
        setup
            .log
            .borrow()
            .iter()
            .map(|entry| entry.0.as_str())
            .collect::<Vec<_>>(),
        ["child", "parent"]
    );
    setup.clear();
    setup.send("touchstart", 2, 0.0, 0.0);
    assert_eq!(setup.log.borrow().len(), 1);
    setup.context.dispose().unwrap();
    setup.context.dispose().unwrap();
    setup.clear();
    assert!(!setup.send("touchstart", 3, 0.0, 0.0).default_prevented());
    assert!(setup.log.borrow().is_empty());
    drop(listener);
    inner.dispose();
    outer.dispose();
}
