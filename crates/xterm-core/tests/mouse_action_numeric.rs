// Copyright (c) 2019, 2026 The xterm.js authors. MIT license.
use std::{cell::RefCell, rc::Rc};
use xterm_core::mouse_state::{
    MouseAction, MouseButton, MouseEvent, MouseEventTypes, MouseProtocol, MouseStateService,
};

fn event(action: MouseAction) -> MouseEvent {
    MouseEvent {
        col: 1.0,
        row: 1.0,
        x: 0.0,
        y: 0.0,
        button: MouseButton::Wheel,
        action,
        ctrl: None,
        alt: None,
        shift: None,
    }
}

#[test]
fn strict_action_equality_does_not_perform_integer_coercion() {
    for (number, named) in [
        (0.0, MouseAction::Up),
        (1.0, MouseAction::Down),
        (2.0, MouseAction::Left),
        (3.0, MouseAction::Right),
        (32.0, MouseAction::Move),
    ] {
        assert_eq!(MouseAction::RawNumber(number), named);
        assert_eq!(named.numeric_value(), number);
        assert_ne!(MouseAction::RawNumber(number + 0.5), named);
        assert_ne!(MouseAction::RawNumber(number + 4_294_967_296.0), named);
    }
    assert_eq!(MouseAction::RawNumber(-0.0), MouseAction::Up);
    assert_eq!(
        MouseAction::RawNumber(-0.0).numeric_value().to_bits(),
        (-0.0f64).to_bits()
    );
    let nan = MouseAction::RawNumber(f64::NAN);
    assert_ne!(nan, nan);
    assert_eq!(
        MouseAction::RawNumber(f64::INFINITY),
        MouseAction::RawNumber(f64::INFINITY)
    );
    // The validated constant conversion stays strict. Raw inputs opt in explicitly.
    assert!(MouseAction::try_from(5).is_err());
    assert!(MouseAction::try_from(255).is_err());
    assert_eq!(MouseAction::try_from(32).unwrap(), MouseAction::Move);
}

#[test]
fn custom_callbacks_observe_and_mutate_original_action_bits() {
    let service = MouseStateService::<()>::new();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let restriction_seen = seen.clone();
    service.add_protocol(
        "RAW",
        MouseProtocol {
            events: MouseEventTypes::DOWN,
            restrict: Rc::new(move |event| {
                restriction_seen
                    .borrow_mut()
                    .push(event.action.numeric_value().to_bits());
                event.action = MouseAction::RawNumber(-0.0);
                Ok(true)
            }),
        },
    );
    let encoding_seen = seen.clone();
    service.add_encoding(
        "RAW",
        Rc::new(move |event| {
            encoding_seen
                .borrow_mut()
                .push(event.action.numeric_value().to_bits());
            event.action = MouseAction::RawNumber(1.5);
            Ok(vec![0xd800])
        }),
    );
    service.set_active_protocol("RAW").unwrap();
    service.set_active_encoding("RAW").unwrap();
    let original = f64::from_bits(0x7ff8_0000_0000_0052);
    let mut event = event(MouseAction::RawNumber(original));
    assert!(service.restrict_mouse_event(&mut event).unwrap());
    assert_eq!(service.encode_mouse_event(&mut event).unwrap(), [0xd800]);
    assert_eq!(*seen.borrow(), [original.to_bits(), (-0.0f64).to_bits()]);
    assert_eq!(event.action.numeric_value().to_bits(), 1.5f64.to_bits());
}
