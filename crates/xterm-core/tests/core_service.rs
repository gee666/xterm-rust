// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Includes all CoreService.test.ts assertions and added edges.
mod service_support;
use service_support::Harness;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::core_service::{
    BufferPosition, CoreError, CursorStyle, DecPrivateModes, KittyKeyboardState, Modes,
};
use xterm_core::event::EmitError;
use xterm_core::lifecycle::Disposable;
use xterm_core::log_service::{LogLevel, LogValue};
use xterm_core::options::OptionValue;

#[test]
fn upstream_cursor_initialization_defaults_and_option() {
    let h = Harness::new([]);
    assert!(!h.core.is_cursor_initialized.get());
    let h = Harness::new([("showCursorImmediately", true.into())]);
    assert!(h.core.is_cursor_initialized.get());
}

#[test]
fn cursor_initialization_uses_nullish_fallback_without_losing_raw_identity() {
    for raw in [OptionValue::Null, OptionValue::Undefined] {
        let h = Harness::new([("showCursorImmediately", raw)]);
        assert_eq!(h.core.is_cursor_initialized.value(), false.into());
    }
    for raw in [
        0.0.into(),
        "".into(),
        "false".into(),
        OptionValue::String(vec![0xd800]),
        OptionValue::object(()),
    ] {
        let h = Harness::new([("showCursorImmediately", raw.clone())]);
        assert_eq!(h.core.is_cursor_initialized.value(), raw);
        assert_eq!(h.core.is_cursor_initialized.get(), raw.is_truthy());
        h.options
            .set("showCursorImmediately", false.into())
            .unwrap();
        h.core.reset();
        assert_eq!(h.core.is_cursor_initialized.value(), raw);
        h.core.is_cursor_initialized.set(true);
        assert_eq!(h.core.is_cursor_initialized.value(), true.into());
    }
}

#[test]
fn upstream_reset_preserves_cursor_initialized_both_values() {
    let h = Harness::new([]);
    h.core.is_cursor_initialized.set(true);
    h.core.reset();
    assert!(h.core.is_cursor_initialized.get());
    h.core.is_cursor_initialized.set(false);
    h.core.reset();
    assert!(!h.core.is_cursor_initialized.get());
}

#[test]
fn defaults_reset_all_modes_and_independent_instances() {
    let h = Harness::new([]);
    assert_eq!(h.logger.level(), Some(LogLevel::Info));
    assert!(!h.core.is_cursor_hidden.get());
    assert_eq!(*h.core.modes.borrow(), Modes { insert_mode: false });
    let defaults = DecPrivateModes {
        application_cursor_keys: false,
        application_keypad: false,
        bracketed_paste_mode: false,
        color_scheme_updates: false,
        cursor_blink: None,
        cursor_style: None,
        origin: false,
        reverse_wraparound: false,
        send_focus: false,
        synchronized_output: false,
        win32_input_mode: false,
        wraparound: true,
    };
    assert_eq!(*h.core.dec_private_modes.borrow(), defaults);
    assert_eq!(
        *h.core.kitty_keyboard.borrow(),
        KittyKeyboardState::default()
    );
    h.core.modes.borrow_mut().insert_mode = true;
    *h.core.dec_private_modes.borrow_mut() = DecPrivateModes {
        application_cursor_keys: true,
        application_keypad: true,
        bracketed_paste_mode: true,
        color_scheme_updates: true,
        cursor_blink: Some(true),
        cursor_style: Some(CursorStyle::Underline),
        origin: true,
        reverse_wraparound: true,
        send_focus: true,
        synchronized_output: true,
        win32_input_mode: true,
        wraparound: false,
    };
    *h.core.kitty_keyboard.borrow_mut() = KittyKeyboardState {
        flags: u32::MAX,
        main_flags: 4,
        alt_flags: 5,
        main_stack: vec![6],
        alt_stack: vec![7],
    };
    h.core.is_cursor_hidden.set(true);
    let other = Harness::new([]);
    assert_eq!(*other.core.dec_private_modes.borrow(), defaults);
    assert_eq!(
        *other.core.kitty_keyboard.borrow(),
        KittyKeyboardState::default()
    );
    h.listen();
    h.core.reset();
    assert!(h.core.is_cursor_hidden.get());
    assert_eq!(*h.core.modes.borrow(), Modes::default());
    assert_eq!(*h.core.dec_private_modes.borrow(), defaults);
    assert_eq!(
        *h.core.kitty_keyboard.borrow(),
        KittyKeyboardState::default()
    );
    h.core.trigger_data_event(&[], false).unwrap();
    assert_eq!(*h.records.order.borrow(), ["data"]);
}

#[test]
fn all_disable_user_scroll_viewport_branches_and_live_updates() {
    let h = Harness::new([("logLevel", "trace".into())]);
    h.listen();
    for disabled in [false, true] {
        for user in [false, true] {
            for scroll in [false, true] {
                for (base, display) in [(0, 0), (5, 5), (5, 2), (2, 5)] {
                    h.options.set("disableStdin", disabled.into()).unwrap();
                    h.options.set("scrollOnUserInput", scroll.into()).unwrap();
                    h.position.set(BufferPosition {
                        ybase: base,
                        ydisp: display,
                    });
                    h.records.order.borrow_mut().clear();
                    h.core
                        .trigger_data_event(&[0, 0xd800, 0xd83d, 0xde00], user)
                        .unwrap();
                    let mut expected = vec![];
                    if !disabled {
                        if user && scroll && base != display {
                            expected.push("scroll");
                        }
                        if user {
                            expected.push("user");
                        }
                        expected.extend(["Debug", "Trace", "data"]);
                    }
                    assert_eq!(*h.records.order.borrow(), expected);
                    h.records.order.borrow_mut().clear();
                    h.core.trigger_binary_event(&[]).unwrap();
                    assert_eq!(
                        *h.records.order.borrow(),
                        if disabled {
                            vec![]
                        } else {
                            vec!["Debug", "Trace", "binary"]
                        }
                    );
                }
            }
        }
    }
}

#[test]
fn strings_and_lazy_code_units_are_exact_utf16() {
    let h = Harness::new([("logLevel", "trace".into())]);
    let units = vec![0, 0xd800, 0xdfff, 0xd83d, 0xde00, 0xfffe];
    let received = Rc::new(RefCell::new(Vec::new()));
    let capture = Rc::clone(&received);
    h.core
        .on_data()
        .subscribe(move |data| capture.borrow_mut().push(data.clone()));
    let capture = Rc::clone(&received);
    h.core
        .on_binary()
        .subscribe(move |data| capture.borrow_mut().push(data.clone()));
    h.core.trigger_data_event(&units, false).unwrap();
    h.core.trigger_binary_event(&units).unwrap();
    h.core.trigger_data_event(&[], false).unwrap();
    assert_eq!(*received.borrow(), [units.clone(), units.clone(), vec![]]);
    let logs = h.records.logs.borrow();
    assert_eq!(logs[1].parameters, [LogValue::CodeUnits(units.clone())]);
    assert_eq!(logs[3].parameters, [LogValue::CodeUnits(units.clone())]);
    let mut expected: Vec<_> = "xterm.js: sending data \"".encode_utf16().collect();
    expected.extend(units);
    expected.push(u16::from(b'"'));
    assert_eq!(logs[0].message, expected);
}

#[test]
fn reentrant_send_and_option_changes_preserve_entry_check() {
    let h = Harness::new([("logLevel", "trace".into())]);
    h.listen();
    h.position.set(BufferPosition { ybase: 5, ydisp: 0 });
    let options = Rc::clone(&h.options);
    let weak = Rc::downgrade(&h.core);
    h.core.on_request_scroll_to_bottom().subscribe(move |_| {
        options.set("disableStdin", true.into()).unwrap();
        weak.upgrade().unwrap().trigger_binary_event(&[42]).unwrap();
    });
    h.core.trigger_data_event(&[1], true).unwrap();
    assert_eq!(
        *h.records.order.borrow(),
        ["scroll", "user", "Debug", "Trace", "data"]
    );
    h.options.set("disableStdin", false.into()).unwrap();
    h.records.order.borrow_mut().clear();
    let once = Rc::new(Cell::new(false));
    let weak = Rc::downgrade(&h.core);
    h.core.on_data().subscribe(move |_| {
        if !once.replace(true) {
            weak.upgrade()
                .unwrap()
                .trigger_data_event(&[2], false)
                .unwrap();
        }
    });
    h.core.trigger_data_event(&[1], false).unwrap();
    assert_eq!(
        *h.records.order.borrow(),
        ["Debug", "Trace", "data", "Debug", "Trace", "data"]
    );
}

#[test]
fn failures_stop_later_events_and_disposal_clears_all_emitters() {
    let h = Harness::new([("logLevel", "trace".into())]);
    h.position.set(BufferPosition { ybase: 1, ydisp: 0 });
    let fail = h.core.on_user_input().try_subscribe(|_| Err(EmitError));
    h.listen();
    assert!(matches!(
        h.core.trigger_data_event(&[], true),
        Err(CoreError::Dispatch(_))
    ));
    assert_eq!(*h.records.order.borrow(), ["scroll"]);
    fail.dispose();
    h.records.order.borrow_mut().clear();
    h.core.dispose();
    h.core.trigger_data_event(&[], true).unwrap();
    h.core.trigger_binary_event(&[]).unwrap();
    assert_eq!(
        *h.records.order.borrow(),
        ["Debug", "Trace", "Debug", "Trace"]
    );
}

#[test]
fn disposing_owner_during_event_and_dropping_owner_clears_listeners() {
    let h = Harness::new([]);
    let weak = Rc::downgrade(&h.core);
    h.core
        .on_user_input()
        .subscribe(move |_| weak.upgrade().unwrap().dispose());
    h.listen();
    assert!(matches!(
        h.core.trigger_data_event(&[], true),
        Err(CoreError::Dispatch(_))
    ));
    let data = h.core.on_data();
    drop(h);
    let received = Rc::new(Cell::new(false));
    let capture = Rc::clone(&received);
    data.subscribe(move |_| capture.set(true));
    assert!(!received.get());
}

#[test]
fn option_truthiness_matches_js_conditions() {
    let h = Harness::new([]);
    h.listen();
    for false_value in [
        OptionValue::Undefined,
        OptionValue::Null,
        false.into(),
        0.0.into(),
        f64::NAN.into(),
        "".into(),
    ] {
        h.options.set_raw("disableStdin", false_value).unwrap();
        h.records.order.borrow_mut().clear();
        h.core.trigger_data_event(&[], true).unwrap();
        assert_eq!(*h.records.order.borrow(), ["user", "data"]);
    }
    for true_value in [
        true.into(),
        1.0.into(),
        "false".into(),
        OptionValue::object(()),
    ] {
        h.options.set_raw("disableStdin", true_value).unwrap();
        h.records.order.borrow_mut().clear();
        h.core.trigger_binary_event(&[]).unwrap();
        assert!(h.records.order.borrow().is_empty());
    }
}
