// Copyright (c) 2020 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Assertions ported from OptionsService.test.ts.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::lifecycle::Disposable;
use xterm_core::options::{default_options, OptionObject, OptionValue as V, OptionsService};

fn number(service: &OptionsService, key: &str) -> f64 {
    match service.get(key).unwrap() {
        V::Number(n) => n,
        other => panic!("not a number: {other:?}"),
    }
}

#[test]
fn constructor_undefined_dimensions_use_defaults() {
    let service = OptionsService::new(false, [("cols", V::Undefined), ("rows", V::Undefined)]);
    assert_eq!(number(&service, "cols"), 80.0);
    assert_eq!(number(&service, "rows"), 24.0);
    assert_eq!(service.diagnostics().len(), 2);
}
#[test]
fn constructor_valid_dimensions() {
    let service = OptionsService::new(
        false,
        [("cols", V::Number(80.0)), ("rows", V::Number(25.0))],
    );
    assert_eq!(number(&service, "cols"), 80.0);
    assert_eq!(number(&service, "rows"), 25.0);
}
#[test]
fn constructor_invalid_value_uses_default() {
    let service = OptionsService::new(false, [("tabStopWidth", V::Number(0.0))]);
    assert_eq!(number(&service, "tabStopWidth"), 8.0);
}
#[test]
fn object_keys_are_nonempty() {
    assert!(!OptionsService::default().keys().is_empty());
}
#[test]
fn valid_font_weights() {
    let service = OptionsService::default();
    for value in [
        V::from("bold"),
        V::from("normal"),
        V::from("600"),
        V::Number(350.0),
        V::Number(1.0),
        V::Number(1000.0),
    ] {
        service.set("fontWeight", value.clone()).unwrap();
        assert_eq!(service.get("fontWeight").unwrap(), value);
    }
}
#[test]
fn invalid_font_weights_normalize() {
    let service = OptionsService::default();
    for value in [V::Number(10000.0), V::Number(-10.0), V::from("bold700")] {
        service.set("fontWeight", V::Number(350.0)).unwrap();
        service.set("fontWeight", value).unwrap();
        assert_eq!(service.get("fontWeight").unwrap(), V::from("normal"));
    }
}
#[test]
fn on_option_change() {
    let service = OptionsService::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let output = log.clone();
    let disposable = service
        .on_option_change()
        .subscribe(move |key| output.borrow_mut().push(*key));
    service.set("cursorWidth", V::Number(10.0)).unwrap();
    assert_eq!(*log.borrow(), ["cursorWidth"]);
    disposable.dispose();
    let output = log.clone();
    service
        .on_option_change()
        .subscribe(move |key| output.borrow_mut().push(*key));
    service.set("scrollback", V::Number(20.0)).unwrap();
    assert_eq!(*log.borrow(), ["cursorWidth", "scrollback"]);
}
fn specific_case() {
    let service = OptionsService::default();
    let values = Rc::new(RefCell::new(Vec::new()));
    let output = values.clone();
    service
        .on_specific_option_change("scrollback", move |value| output.borrow_mut().push(value))
        .unwrap();
    service.set("cursorWidth", V::Number(10.0)).unwrap();
    assert!(values.borrow().is_empty());
    service.set("scrollback", V::Number(20.0)).unwrap();
    assert_eq!(*values.borrow(), [V::Number(20.0)]);
}
#[test]
fn specific_option_change_first_upstream_case() {
    specific_case();
}
#[test]
fn specific_option_change_duplicate_upstream_case() {
    specific_case();
}
#[test]
fn multiple_option_change() {
    let service = OptionsService::default();
    let called = Rc::new(Cell::new(false));
    let output = called.clone();
    service
        .on_multiple_option_change(&["scrollback"], move || output.set(true))
        .unwrap();
    service.set("cursorWidth", V::Number(10.0)).unwrap();
    assert!(!called.get());
    service.set("scrollback", V::Number(20.0)).unwrap();
    assert!(called.get());
}

#[test]
fn validation_boundaries_and_coercion() {
    let service = OptionsService::default();
    for key in ["lineHeight", "tabStopWidth", "cursorWidth"] {
        assert!(service.set(key, V::Number(0.9)).is_err());
    }
    for key in ["fastScrollSensitivity", "scrollSensitivity"] {
        assert!(service.set(key, V::Number(0.0)).is_err());
    }
    service.set("cursorWidth", V::from("2.9")).unwrap();
    assert_eq!(number(&service, "cursorWidth"), 2.0);
    service.set("lineHeight", V::from("2.9")).unwrap();
    assert_eq!(service.get("lineHeight").unwrap(), V::from("2.9"));
    service.set("scrollback", V::Number(f64::INFINITY)).unwrap();
    assert_eq!(number(&service, "scrollback"), 4_294_967_295.0);
    service
        .set("minimumContrastRatio", V::Number(2.25))
        .unwrap();
    assert_eq!(number(&service, "minimumContrastRatio"), 2.3);
    service.set("minimumContrastRatio", V::Undefined).unwrap();
    assert!(number(&service, "minimumContrastRatio").is_nan());
    service.set("scrollback", V::Undefined).unwrap();
    assert!(number(&service, "scrollback").is_nan());
    service.set("cols", V::Number(0.0)).unwrap();
    service.set("cols", V::from("not really numeric")).unwrap();
    assert!(service.set("rows", V::Bool(false)).is_err());
    assert!(service.set("cursorStyle", V::from("outline")).is_err());
    service.set("cursorStyle", V::Null).unwrap();
    assert_eq!(service.get("cursorStyle").unwrap(), V::from("block"));
}

#[test]
fn strict_equality_and_raw_write() {
    let service = OptionsService::default();
    let count = Rc::new(Cell::new(0));
    let output = count.clone();
    service
        .on_option_change()
        .subscribe(move |_| output.set(output.get() + 1));
    service.set("cols", V::Number(80.0)).unwrap();
    assert_eq!(count.get(), 0);
    for _ in 0..2 {
        service.set("fontSize", V::Number(f64::NAN)).unwrap();
    }
    assert_eq!(count.get(), 2);
    let object = V::object(());
    service.set("theme", object.clone()).unwrap();
    service.set("theme", object).unwrap();
    assert_eq!(count.get(), 3);
    service.set("theme", V::object(())).unwrap();
    assert_eq!(count.get(), 4);
    service.set_raw("rows", V::Number(-50.0)).unwrap();
    assert_eq!(number(&service, "rows"), -50.0);
    assert_eq!(count.get(), 4);
}

#[test]
fn defaults_share_objects_but_not_platform_policy() {
    let first = OptionsService::default();
    let second = OptionsService::new(true, std::iter::empty::<(&str, V)>());
    assert_eq!(first.get("theme").unwrap(), second.get("theme").unwrap());
    assert_eq!(first.get("rightClickSelectsWord").unwrap(), V::Bool(false));
    assert_eq!(second.get("rightClickSelectsWord").unwrap(), V::Bool(true));
    assert_eq!(default_options(false).len(), 48);
    assert_eq!(first.keys()[0..2], ["cols", "rows"]);
}

#[test]
fn utf16_and_object_coercion() {
    let service = OptionsService::default();
    let lone = V::String(vec![0xd800]);
    service.set("wordSeparator", lone.clone()).unwrap();
    assert_eq!(service.get("wordSeparator").unwrap(), lone);
    for (value, expected) in [("\u{feff}0x10\u{a0}", 16.0), ("0b11", 3.0), ("0o17", 15.0)] {
        service.set("cursorWidth", V::from(value)).unwrap();
        assert_eq!(number(&service, "cursorWidth"), expected);
    }
    let object = V::Object(OptionObject::new("adapter", |_| Ok(V::Number(3.9))));
    service.set("cursorWidth", object).unwrap();
    assert_eq!(number(&service, "cursorWidth"), 3.0);
}

#[test]
fn long_radix_coercion_rounds_once() {
    let service = OptionsService::default();
    service
        .set("blinkIntervalDuration", V::from("0x2000000000000101"))
        .unwrap();
    assert_eq!(
        number(&service, "blinkIntervalDuration"),
        ((1_u64 << 61) + 512) as f64
    );
    service
        .set("blinkIntervalDuration", V::from("0x2000000000000100"))
        .unwrap();
    assert_eq!(
        number(&service, "blinkIntervalDuration"),
        (1_u64 << 61) as f64
    );
    service
        .set("blinkIntervalDuration", V::from("0x2000000000000300"))
        .unwrap();
    assert_eq!(
        number(&service, "blinkIntervalDuration"),
        ((1_u64 << 61) + 1024) as f64
    );
}

#[test]
fn invalid_utf16_error_is_lossless() {
    let service = OptionsService::default();
    let error = service
        .set("cursorStyle", V::String(vec![0xd800]))
        .unwrap_err();
    let mut expected = vec![u16::from(b'\"'), 0xd800, u16::from(b'\"')];
    expected.extend(" is not a valid value for cursorStyle".encode_utf16());
    assert_eq!(error.message_utf16(), expected);
    assert_eq!(service.get("cursorStyle").unwrap(), V::from("block"));
}

#[test]
fn construction_unknown_keys_and_diagnostic_order() {
    let service = OptionsService::new(
        false,
        [
            ("unknown", V::Null),
            ("rows", V::Undefined),
            ("tabStopWidth", V::Number(0.0)),
            ("cols", V::Number(99.0)),
            ("rows", V::Number(30.0)),
        ],
    );
    assert_eq!(service.diagnostics().len(), 2);
    assert_eq!(
        service.diagnostics()[0].to_string(),
        "rows must be numeric, value: undefined"
    );
    assert_eq!(
        service.diagnostics()[1].to_string(),
        "tabStopWidth cannot be less than 1, value: 0"
    );
    assert_eq!(number(&service, "cols"), 99.0);
    assert_eq!(number(&service, "rows"), 30.0);
    assert!(service.get("unknown").is_err());
}

#[test]
fn listener_reentrancy_reads_current_value_and_mutates() {
    let service = Rc::new(OptionsService::default());
    let weak = Rc::downgrade(&service);
    service.on_option_change().subscribe(move |key| {
        if *key == "cols" {
            weak.upgrade()
                .unwrap()
                .set("rows", V::Number(30.0))
                .unwrap();
        }
    });
    let weak = Rc::downgrade(&service);
    service
        .on_specific_option_change("cols", move |value| {
            assert_eq!(value, V::Number(100.0));
            assert_eq!(number(&weak.upgrade().unwrap(), "rows"), 30.0);
        })
        .unwrap();
    service.set("cols", V::Number(100.0)).unwrap();
}

#[test]
fn disposal_clears_external_references_without_events() {
    let service = OptionsService::default();
    let object = OptionObject::plain(());
    let weak = Rc::downgrade(&object);
    service.set("documentOverride", V::Object(object)).unwrap();
    service.set("linkHandler", V::object(())).unwrap();
    service
        .on_option_change()
        .subscribe(|_| panic!("disposed emitter must not fire"));
    service.dispose();
    assert!(weak.upgrade().is_none());
    assert_eq!(service.get("linkHandler").unwrap(), V::Null);
    service.set("rows", V::Number(31.0)).unwrap();
    assert_eq!(number(&service, "rows"), 31.0);
}
