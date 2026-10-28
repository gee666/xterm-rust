// Copyright (c) 2019, 2026 The xterm.js authors. All rights reserved. MIT license.
// Retained custom objects, mutable callbacks and concrete callback errors.
use std::{
    cell::Cell,
    error::Error,
    fmt,
    rc::{Rc, Weak},
};
use xterm_core::{
    event::EmitError,
    mouse_state::{
        MouseAction, MouseButton, MouseEvent, MouseEventTypes as Events, MouseProtocol,
        MouseStateService, SharedMouseProtocol,
    },
};
type Service = MouseStateService<()>;
fn input() -> MouseEvent {
    MouseEvent {
        col: 1.0,
        row: 1.0,
        x: 0.0,
        y: 0.0,
        button: MouseButton::Left,
        action: MouseAction::Down,
        ctrl: None,
        alt: None,
        shift: None,
    }
}

#[test]
fn custom_encoder_mutates_the_original_event() {
    let service = Service::new();
    service.add_encoding(
        "MUTABLE",
        Rc::new(|event| {
            event.col = 7.5;
            event.ctrl = Some(true);
            event.button = MouseButton::Raw(-1);
            vec![0xd800]
        }),
    );
    service.set_active_encoding("MUTABLE").unwrap();
    let mut event = input();
    assert_eq!(service.encode_mouse_event(&mut event), [0xd800]);
    assert_eq!(event.col, 7.5);
    assert_eq!(event.ctrl, Some(true));
    assert_eq!(event.button.raw_value(), -1);
}

#[test]
fn shared_protocol_external_edits_are_visible_in_multiple_services() {
    let first = Service::new();
    let second = Service::new();
    let protocol = Rc::new(SharedMouseProtocol::new(MouseProtocol {
        events: Events::DOWN,
        restrict: Rc::new(|_| true),
    }));
    for service in [&first, &second] {
        service.add_shared_protocol("SHARED", Rc::clone(&protocol));
        service.set_active_protocol("SHARED").unwrap();
        assert!(service.are_mouse_events_active());
        assert!(service.restrict_mouse_event(&mut input()));
    }
    protocol.set_events(Events::NONE);
    protocol.set_restriction(Rc::new(|event| {
        event.row = 9.0;
        false
    }));
    assert_eq!(protocol.events(), Events::NONE);
    for service in [&first, &second] {
        assert!(!service.are_mouse_events_active());
        let mut event = input();
        assert!(!service.restrict_mouse_event(&mut event));
        assert_eq!(event.row, 9.0);
    }
}

#[test]
fn protocol_replacement_during_callback_finishes_the_original_call() {
    let service = Service::new();
    let protocol = Rc::new(SharedMouseProtocol::new(MouseProtocol {
        events: Events::DOWN,
        restrict: Rc::new(|_| false),
    }));
    let weak = Rc::downgrade(&protocol);
    protocol.set_restriction(Rc::new(move |_| {
        weak.upgrade().unwrap().set_restriction(Rc::new(|_| false));
        true
    }));
    service.add_shared_protocol("SHARED", protocol);
    service.set_active_protocol("SHARED").unwrap();
    assert!(service.restrict_mouse_event(&mut input()));
    assert!(!service.restrict_mouse_event(&mut input()));
}

#[derive(Debug)]
struct HostFailure;
impl fmt::Display for HostFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("original host failure")
    }
}
impl Error for HostFailure {}
#[test]
fn activation_retains_the_original_callback_error_chain() {
    let service = Service::new();
    service
        .on_protocol_change()
        .try_subscribe(|_| Err(EmitError::callback(HostFailure)));
    let error = service.set_active_protocol("ANY").unwrap_err();
    assert_eq!(error.to_string(), "original host failure");
    let dispatch = error.source().unwrap().downcast_ref::<EmitError>().unwrap();
    assert!(dispatch.callback_ref::<HostFailure>().is_some());
    assert!(dispatch.source().unwrap().is::<HostFailure>());
    assert_eq!(service.active_protocol(), "ANY");
}

struct ReadOnDrop {
    service: Weak<Service>,
    drops: Rc<Cell<usize>>,
}
impl Drop for ReadOnDrop {
    fn drop(&mut self) {
        if let Some(service) = self.service.upgrade() {
            assert_eq!(service.active_protocol(), "NONE");
        }
        self.drops.set(self.drops.get() + 1);
    }
}
#[test]
fn replacing_callback_captures_releases_them_outside_service_borrows() {
    let service = Rc::new(Service::new());
    let drops = Rc::new(Cell::new(0));
    let guard = || ReadOnDrop {
        service: Rc::downgrade(&service),
        drops: Rc::clone(&drops),
    };
    let captured = guard();
    service.add_encoding(
        "CUSTOM",
        Rc::new(move |_| {
            let _retained = &captured;
            Vec::new()
        }),
    );
    service.add_encoding("CUSTOM", Rc::new(|_| Vec::new()));
    assert_eq!(drops.get(), 1);
    let captured = guard();
    service.add_protocol(
        "CUSTOM",
        MouseProtocol {
            events: Events::DOWN,
            restrict: Rc::new(move |_| {
                let _retained = &captured;
                true
            }),
        },
    );
    service.add_protocol(
        "CUSTOM",
        MouseProtocol {
            events: Events::DOWN,
            restrict: Rc::new(|_| true),
        },
    );
    assert_eq!(drops.get(), 2);
    let captured = guard();
    service.set_custom_wheel_event_handler(Some(Rc::new(move |_| {
        let _retained = &captured;
        true
    })));
    service.set_custom_wheel_event_handler(None);
    assert_eq!(drops.get(), 3);
}
