// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
use std::cell::RefCell;
use std::convert::Infallible;
use std::rc::Rc;
use xterm_core::mouse_state::{MouseAction, MouseButton, MouseEvent, MouseStateService};
use xterm_web::input::mouse_coords::ReportCoords;
use xterm_web::input::mouse_policy::{
    MouseInput, MouseOutput, MousePolicy, MouseRouting, PointerEventKind, PointerInput,
};
use xterm_web::input::mouse_wheel::WheelOptions;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn routing() -> MouseRouting {
    MouseRouting {
        cols: 500.0,
        rows: 500.0,
        require_alt: false,
        device_cell_height: Some(20.0),
        dpr: Some(2.0),
        wheel_options: WheelOptions::default(),
    }
}
fn pointer(kind: PointerEventKind) -> PointerInput {
    PointerInput {
        kind,
        buttons: Some(0),
        button: Some(0),
        ctrl: false,
        alt: false,
        shift: false,
        delta_y: 50.0,
        delta_mode: 0.0,
    }
}
fn input(pointer: PointerInput) -> MouseInput<'static, ()> {
    MouseInput {
        pointer,
        coords: Some(ReportCoords {
            col: 0.0,
            row: 0.0,
            x: 0.0,
            y: 0.0,
        }),
        wheel_payload: Some(&()),
    }
}
fn service() -> MouseStateService<()> {
    let service = MouseStateService::new();
    service.set_active_protocol("ANY").unwrap();
    service.set_active_encoding("SGR").unwrap();
    service
}
fn event() -> MouseEvent {
    MouseEvent {
        col: 0.0,
        row: 0.0,
        x: 0.0,
        y: 0.0,
        button: MouseButton::None,
        action: MouseAction::Move,
        ctrl: None,
        alt: None,
        shift: None,
    }
}
fn payload(text: &str) -> MouseOutput {
    MouseOutput::UserData(text.encode_utf16().collect())
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn upstream_alt_required_report_strips_only_nonwheel_alt() {
    let service = service();
    let policy = MousePolicy::new();
    let mut config = routing();
    config.require_alt = true;
    let mut raw = pointer(PointerEventKind::Down);
    let mut reports = Vec::new();
    assert!(!policy
        .route(&service, input(raw), config, |out| {
            reports.push(out);
            Ok::<_, Infallible>(())
        })
        .unwrap());
    raw.alt = true;
    assert!(policy
        .route(&service, input(raw), config, |out| {
            reports.push(out);
            Ok::<_, Infallible>(())
        })
        .unwrap());
    assert_eq!(reports, [payload("\x1b[<0;1;1M")]);
    raw.kind = PointerEventKind::Wheel;
    assert!(policy
        .route(&service, input(raw), config, |out| {
            reports.push(out);
            Ok::<_, Infallible>(())
        })
        .unwrap());
    assert_eq!(reports[1], payload("\x1b[<73;1;1M"));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn move_buttons_priority_fallback_override_and_invalid_button_pairs() {
    let service = service();
    for (buttons, code) in [
        (0, 35),
        (1, 32),
        (2, 34),
        (3, 32),
        (4, 33),
        (6, 33),
        (7, 32),
        (8, 35),
    ] {
        let policy = MousePolicy::new();
        let mut raw = pointer(PointerEventKind::Move);
        raw.buttons = Some(buttons);
        let mut reports = Vec::new();
        assert!(policy
            .route(&service, input(raw), routing(), |out| {
                reports.push(out);
                Ok::<_, Infallible>(())
            })
            .unwrap());
        assert_eq!(reports, [payload(&format!("\x1b[<{code};1;1M"))]);
    }
    let policy = MousePolicy::new();
    let mut raw = pointer(PointerEventKind::Move);
    raw.buttons = None;
    raw.button = Some(-1);
    let mut reports = Vec::new();
    assert!(policy
        .route(&service, input(raw), routing(), |out| {
            reports.push(out);
            Ok::<_, Infallible>(())
        })
        .unwrap());
    assert_eq!(reports, [payload("\x1b[<227;1;1M")]);
    raw.kind = PointerEventKind::Down;
    raw.button = Some(3);
    assert!(!policy
        .route(&service, input(raw), routing(), |_| Ok::<_, Infallible>(()))
        .unwrap());
    assert_eq!(
        PointerEventKind::from_names("mousedown", Some("mousemove")),
        PointerEventKind::Move
    );
    assert_eq!(
        PointerEventKind::from_names("mousedown", Some("")),
        PointerEventKind::Down
    );
    assert_eq!(
        PointerEventKind::from_names("click", None),
        PointerEventKind::Other
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn wheel_veto_zero_shift_and_missing_coordinates_do_not_consume_state() {
    let service = service();
    let policy = MousePolicy::new();
    service.set_custom_wheel_event_handler(Some(Rc::new(|_| false)));
    let raw = pointer(PointerEventKind::Wheel);
    assert!(!policy
        .route(&service, input(raw), routing(), |_| Ok::<_, Infallible>(()))
        .unwrap());
    assert_eq!(policy.wheel_state().partial_scroll(), 0.0);
    service.set_custom_wheel_event_handler(None);
    for (delta_y, shift) in [(0.0, false), (50.0, true)] {
        let raw = PointerInput {
            delta_y,
            shift,
            ..raw
        };
        assert!(!policy
            .route(&service, input(raw), routing(), |_| Ok::<_, Infallible>(()))
            .unwrap());
    }
    let mut missing = input(raw);
    missing.coords = None;
    assert!(!policy
        .route(&service, missing, routing(), |_| Ok::<_, Infallible>(()))
        .unwrap());
    assert_eq!(policy.wheel_state().partial_scroll(), 0.0);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn empty_encoding_still_commits_last_event_and_reset_clears_it() {
    let service = service();
    service.set_active_encoding("DEFAULT").unwrap();
    let policy = MousePolicy::new();
    let mut input = event();
    input.col = 300.0;
    assert!(policy
        .trigger(
            &service,
            &mut input,
            500.0,
            500.0,
            |_| -> Result<(), Infallible> { panic!("empty report was emitted") }
        )
        .unwrap());
    let mut input = event();
    input.col = 300.0;
    assert!(!policy
        .trigger(&service, &mut input, 500.0, 500.0, |_| Ok::<_, Infallible>(
            ()
        ))
        .unwrap());
    policy.reset();
    let mut input = event();
    input.col = 300.0;
    assert!(policy
        .trigger(&service, &mut input, 500.0, 500.0, |_| Ok::<_, Infallible>(
            ()
        ))
        .unwrap());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn output_failure_does_not_commit_cache_and_nested_output_sees_old_cache() {
    let service = service();
    let policy = MousePolicy::new();
    let error = policy.trigger(&service, &mut event(), 500.0, 500.0, |_| {
        Err("original output failure")
    });
    assert_eq!(error, Err("original output failure"));
    let reports = RefCell::new(Vec::new());
    assert!(policy
        .trigger(&service, &mut event(), 500.0, 500.0, |out| {
            reports.borrow_mut().push(out);
            assert!(policy
                .trigger(&service, &mut event(), 500.0, 500.0, |out| {
                    reports.borrow_mut().push(out);
                    Ok::<_, Infallible>(())
                })
                .unwrap());
            Ok::<_, Infallible>(())
        })
        .unwrap());
    assert_eq!(reports.borrow().len(), 2);
    assert!(!policy
        .trigger(
            &service,
            &mut event(),
            500.0,
            500.0,
            |_| Ok::<_, Infallible>(())
        )
        .unwrap());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn touch_assignment_commits_captured_accumulator_after_reentrant_reset() {
    let policy = MousePolicy::new();
    assert_eq!(policy.consume_touch(-5.0, Some(20.0)), 0.0);
    let before = policy.wheel_state().touch_accumulator();
    assert_eq!(before, 5.0);
    // A translation getter dispatches gesturestart before returning its value.
    policy.touch_start();
    assert_eq!(policy.consume_touch_from(before, 2.0, Some(20.0)), 0.0);
    assert_eq!(policy.wheel_state().touch_accumulator(), 3.0);
    // A throwing getter keeps its reset but never calls the outer commit.
    policy.touch_start();
    assert_eq!(policy.wheel_state().touch_accumulator(), 0.0);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn source_mutable_encoder_updates_event_before_cache_commit() {
    let service = service();
    service.add_encoding(
        "CUSTOM",
        Rc::new(|event| {
            event.col = 9.0;
            event.shift = Some(true);
            vec![u16::from(b'x')]
        }),
    );
    service.set_active_encoding("CUSTOM").unwrap();
    let policy = MousePolicy::new();
    let mut input = event();
    let mut reports = Vec::new();
    assert!(policy
        .trigger(&service, &mut input, 500.0, 500.0, |out| {
            reports.push(out);
            Ok::<_, Infallible>(())
        })
        .unwrap());
    assert_eq!(input.col, 9.0);
    assert_eq!(input.shift, Some(true));
    assert_eq!(reports, [payload("x")]);
    let mut next = event();
    next.col = 8.0;
    next.shift = Some(true);
    assert!(!policy
        .trigger(&service, &mut next, 500.0, 500.0, |_| Ok::<_, Infallible>(
            ()
        ))
        .unwrap());
}
