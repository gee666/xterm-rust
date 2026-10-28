// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. First eight cases port common/services/MouseStateService.test.ts.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::lifecycle::Disposable;
use xterm_core::mouse_state::{
    MouseAction, MouseButton, MouseEvent, MouseEventTypes as Events, MouseProtocol,
    MouseStateService,
};

type Service = MouseStateService<()>;
fn event() -> MouseEvent {
    MouseEvent {
        col: 1.0,
        row: 1.0,
        x: 0.0,
        y: 0.0,
        button: MouseButton::Left,
        action: MouseAction::Down,
        ctrl: Some(false),
        alt: Some(false),
        shift: Some(false),
    }
}

#[test]
fn upstream_init() {
    let service = Service::new();
    assert_eq!(service.active_encoding(), "DEFAULT");
    assert_eq!(service.active_protocol(), "NONE");
}

#[test]
fn upstream_default_protocols() {
    assert_eq!(
        Service::new().protocol_names(),
        ["NONE", "X10", "VT200", "DRAG", "ANY"]
    );
}

#[test]
fn upstream_default_encodings() {
    assert_eq!(
        Service::new().encoding_names(),
        ["DEFAULT", "SGR", "SGR_PIXELS"]
    );
}

#[test]
fn upstream_setters_reset_and_errors() {
    let service = Service::new();
    service.set_active_encoding("SGR").unwrap();
    service.set_active_protocol("ANY").unwrap();
    assert_eq!(service.active_encoding(), "SGR");
    assert_eq!(service.active_protocol(), "ANY");
    service.reset().unwrap();
    assert_eq!(service.active_encoding(), "DEFAULT");
    assert_eq!(service.active_protocol(), "NONE");
    assert_eq!(
        service.set_active_encoding("xyz").unwrap_err().to_string(),
        "unknown encoding \"xyz\""
    );
    assert_eq!(
        service.set_active_protocol("xyz").unwrap_err().to_string(),
        "unknown protocol \"xyz\""
    );
}

#[test]
fn upstream_add_encoding() {
    let service = Service::new();
    service.add_encoding("XYZ", Rc::new(|_| Vec::new()));
    service.set_active_encoding("XYZ").unwrap();
    assert_eq!(service.active_encoding(), "XYZ");
}

#[test]
fn upstream_add_protocol() {
    let service = Service::new();
    service.add_protocol(
        "XYZ",
        MouseProtocol {
            events: Events::NONE,
            restrict: Rc::new(|_| false),
        },
    );
    service.set_active_protocol("XYZ").unwrap();
    assert_eq!(service.active_protocol(), "XYZ");
}

#[test]
fn upstream_protocol_change_event() {
    let service = Service::new();
    let events = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&events);
    service
        .on_protocol_change()
        .subscribe(move |value| output.borrow_mut().push(*value));
    service.set_active_protocol("NONE").unwrap();
    assert_eq!(*events.borrow(), [Events::NONE]);
    service.set_active_protocol("ANY").unwrap();
    assert_eq!(
        *events.borrow(),
        [
            Events::NONE,
            Events::DOWN | Events::UP | Events::WHEEL | Events::DRAG | Events::MOVE
        ]
    );
}

#[test]
fn upstream_restrict_and_encode_mouse_event() {
    let service = Service::new();
    service.set_active_protocol("ANY").unwrap();
    service.set_active_encoding("DEFAULT").unwrap();
    let mut input = event();
    assert!(service.restrict_mouse_event(&mut input));
    assert_eq!(
        service.encode_mouse_event(&mut input),
        [0x1b, 0x5b, 0x4d, 0x20, 0x21, 0x21]
    );
}

#[test]
fn flags_and_unknown_setters_preserve_active_state_and_events() {
    let service = Service::new();
    let count = Rc::new(Cell::new(0));
    let output = Rc::clone(&count);
    service
        .on_protocol_change()
        .subscribe(move |_| output.set(output.get() + 1));
    assert!(!service.are_mouse_events_active());
    assert!(service.is_default_encoding());
    assert!(!service.is_pixel_encoding());
    service.set_active_protocol("ANY").unwrap();
    service.set_active_protocol("ANY").unwrap();
    service.set_active_encoding("SGR_PIXELS").unwrap();
    assert!(service.are_mouse_events_active());
    assert!(!service.is_default_encoding());
    assert!(service.is_pixel_encoding());
    assert!(service.set_active_protocol("missing").is_err());
    assert!(service.set_active_encoding("missing").is_err());
    assert_eq!(service.active_protocol(), "ANY");
    assert_eq!(service.active_encoding(), "SGR_PIXELS");
    assert_eq!(count.get(), 2);
}

#[test]
fn optional_modifiers_are_observable_and_x10_only_mutates_accepted_events() {
    let service = Service::new();
    let mut input = event();
    input.ctrl = None;
    input.alt = None;
    input.shift = None;
    service.add_protocol(
        "OPTIONAL",
        MouseProtocol {
            events: Events::DOWN,
            restrict: Rc::new(|event| {
                event.ctrl.is_none() && event.alt.is_none() && event.shift.is_none()
            }),
        },
    );
    service.set_active_protocol("OPTIONAL").unwrap();
    assert!(service.restrict_mouse_event(&mut input));
    assert_eq!(
        service.encode_mouse_event(&mut input),
        [0x1b, 0x5b, 0x4d, 0x20, 0x21, 0x21]
    );
    service.set_active_protocol("X10").unwrap();
    input.button = MouseButton::Wheel;
    let original = input.clone();
    assert!(!service.restrict_mouse_event(&mut input));
    assert_eq!(input, original);
    input.button = MouseButton::Left;
    assert!(service.restrict_mouse_event(&mut input));
    assert_eq!(
        (input.ctrl, input.alt, input.shift),
        (Some(false), Some(false), Some(false))
    );
}

#[test]
fn active_overwrites_take_effect_without_notification_and_reset_retains_registry() {
    let service = Service::new();
    let events = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&events);
    service
        .on_protocol_change()
        .subscribe(move |value| output.borrow_mut().push(*value));
    service.add_protocol(
        "NONE",
        MouseProtocol {
            events: Events::UP,
            restrict: Rc::new(|_| true),
        },
    );
    service.add_encoding("DEFAULT", Rc::new(|_| vec![0xd800, 0xdfff]));
    service.add_encoding("CUSTOM", Rc::new(|_| vec![1]));
    assert!(events.borrow().is_empty());
    assert!(service.are_mouse_events_active());
    assert!(service.restrict_mouse_event(&mut event()));
    assert_eq!(service.encode_mouse_event(&mut event()), [0xd800, 0xdfff]);
    service.set_active_protocol("X10").unwrap();
    service.set_active_encoding("CUSTOM").unwrap();
    service.reset().unwrap();
    assert_eq!(*events.borrow(), [Events::DOWN, Events::UP]);
    assert_eq!(service.active_encoding(), "DEFAULT");
    assert_eq!(service.encode_mouse_event(&mut event()), [0xd800, 0xdfff]);
    service.set_active_encoding("CUSTOM").unwrap();
    assert_eq!(service.encode_mouse_event(&mut event()), [1]);
}

#[test]
fn custom_callbacks_invoked_once_and_mutations_are_not_hidden() {
    let service = Service::new();
    let count = Rc::new(Cell::new(0));
    let calls = Rc::clone(&count);
    service.add_protocol(
        "CUSTOM",
        MouseProtocol {
            events: Events::NONE,
            restrict: Rc::new(move |event| {
                calls.set(calls.get() + 1);
                event.x = -10.5;
                event.shift = None;
                false
            }),
        },
    );
    service.set_active_protocol("CUSTOM").unwrap();
    let calls = Rc::clone(&count);
    service.add_encoding(
        "CUSTOM",
        Rc::new(move |event| {
            calls.set(calls.get() + 1);
            assert_eq!(event.x, -10.5);
            assert_eq!(event.shift, None);
            vec![0xd800]
        }),
    );
    service.set_active_encoding("CUSTOM").unwrap();
    let mut input = event();
    assert!(!service.restrict_mouse_event(&mut input));
    assert_eq!(count.get(), 1);
    assert_eq!(service.encode_mouse_event(&mut input), [0xd800]);
    assert_eq!(count.get(), 2);
}

#[test]
fn wheel_handler_identity_replace_clear_reset_and_dispose() {
    struct Wheel {
        value: Cell<u32>,
    }
    let service = MouseStateService::<Wheel>::new();
    let input = Rc::new(Wheel {
        value: Cell::new(0),
    });
    assert!(service.allow_custom_wheel_event(&input));
    let original = Rc::clone(&input);
    service.set_custom_wheel_event_handler(Some(Rc::new(move |actual| {
        assert!(std::ptr::eq(actual, original.as_ref()));
        actual.value.set(actual.value.get() + 1);
        false
    })));
    assert!(!service.allow_custom_wheel_event(&input));
    assert_eq!(input.value.get(), 1);
    service.reset().unwrap();
    service.dispose();
    assert!(!service.allow_custom_wheel_event(&input));
    assert_eq!(input.value.get(), 2);
    service.set_custom_wheel_event_handler(Some(Rc::new(|_| true)));
    assert!(service.allow_custom_wheel_event(&input));
    service.set_custom_wheel_event_handler(None);
    assert!(service.allow_custom_wheel_event(&input));
    assert_eq!(input.value.get(), 2);
}

#[test]
fn disposal_is_idempotent_stops_notifications_but_not_service_operations() {
    let service = Service::new();
    let count = Rc::new(Cell::new(0));
    let calls = Rc::clone(&count);
    service
        .on_protocol_change()
        .subscribe(move |_| calls.set(calls.get() + 1));
    service.dispose();
    service.dispose();
    let calls = Rc::clone(&count);
    service
        .on_protocol_change()
        .subscribe(move |_| calls.set(calls.get() + 100));
    service.set_active_protocol("ANY").unwrap();
    service.set_active_encoding("SGR").unwrap();
    assert_eq!(count.get(), 0);
    assert!(service.restrict_mouse_event(&mut event()));
    assert_eq!(
        service.encode_mouse_event(&mut event()),
        "\x1b[<0;1;1M".encode_utf16().collect::<Vec<_>>()
    );
    service.reset().unwrap();
    assert_eq!(count.get(), 0);
    assert_eq!(service.active_protocol(), "NONE");
}
