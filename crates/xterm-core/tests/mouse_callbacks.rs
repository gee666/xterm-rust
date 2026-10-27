// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Mouse registration, callback snapshots and dispatch errors.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::lifecycle::Disposable;
use xterm_core::mouse_state::{
    MouseAction, MouseButton, MouseEvent, MouseEventTypes as Events, MouseProtocol,
    MouseStateError, MouseStateService,
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
fn advertised_masks_match_all_five_protocols() {
    let service = Service::new();
    let observed = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&observed);
    service
        .on_protocol_change()
        .subscribe(move |mask| output.borrow_mut().push(mask.0));
    for name in ["NONE", "X10", "VT200", "DRAG", "ANY"] {
        service.set_active_protocol(name).unwrap();
    }
    assert_eq!(*observed.borrow(), [0, 1, 19, 23, 31]);
}

#[test]
fn protocol_listener_observes_committed_state_and_can_reenter() {
    let service = Rc::new(Service::new());
    let observed = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&observed);
    let weak = Rc::downgrade(&service);
    service.on_protocol_change().subscribe(move |mask| {
        let service = weak.upgrade().unwrap();
        output
            .borrow_mut()
            .push((mask.0, service.active_protocol()));
        if mask.0 == 1 {
            service.set_active_protocol("ANY").unwrap();
        }
    });
    service.set_active_protocol("X10").unwrap();
    assert_eq!(*observed.borrow(), [(1, "X10".into()), (31, "ANY".into())]);
    assert_eq!(service.active_protocol(), "ANY");
}

#[test]
fn custom_restriction_and_encoder_can_replace_themselves_without_borrow_conflicts() {
    let service = Rc::new(Service::new());
    let weak = Rc::downgrade(&service);
    service.add_protocol(
        "CUSTOM",
        MouseProtocol {
            events: Events::DOWN,
            restrict: Rc::new(move |_| {
                weak.upgrade().unwrap().add_protocol(
                    "CUSTOM",
                    MouseProtocol {
                        events: Events::UP,
                        restrict: Rc::new(|_| false),
                    },
                );
                true
            }),
        },
    );
    service.set_active_protocol("CUSTOM").unwrap();
    assert!(service.restrict_mouse_event(&mut input()));
    assert!(!service.restrict_mouse_event(&mut input()));
    let weak = Rc::downgrade(&service);
    service.add_encoding(
        "CUSTOM",
        Rc::new(move |_| {
            weak.upgrade()
                .unwrap()
                .add_encoding("CUSTOM", Rc::new(|_| vec![2]));
            vec![1]
        }),
    );
    service.set_active_encoding("CUSTOM").unwrap();
    assert_eq!(service.encode_mouse_event(&input()), [1]);
    assert_eq!(service.encode_mouse_event(&input()), [2]);
}

#[test]
fn wheel_handler_can_clear_itself_and_its_current_result_is_retained() {
    let service = Rc::new(Service::new());
    let weak = Rc::downgrade(&service);
    service.set_custom_wheel_event_handler(Some(Rc::new(move |_| {
        weak.upgrade().unwrap().set_custom_wheel_event_handler(None);
        false
    })));
    assert!(!service.allow_custom_wheel_event(&()));
    assert!(service.allow_custom_wheel_event(&()));
}

#[test]
fn interrupted_reset_propagates_error_without_resetting_encoding() {
    let service = Rc::new(Service::new());
    service.set_active_encoding("SGR").unwrap();
    service.set_active_protocol("ANY").unwrap();
    let weak = Rc::downgrade(&service);
    service
        .on_protocol_change()
        .subscribe(move |_| weak.upgrade().unwrap().dispose());
    let calls = Rc::new(Cell::new(0));
    let output = Rc::clone(&calls);
    service
        .on_protocol_change()
        .subscribe(move |_| output.set(output.get() + 1));
    assert!(matches!(service.reset(), Err(MouseStateError::Dispatch(_))));
    assert_eq!(service.active_protocol(), "NONE");
    assert_eq!(service.active_encoding(), "SGR");
    assert_eq!(calls.get(), 0);
    // The emitter is now disposed, so a subsequent reset has no failing listener.
    service.reset().unwrap();
    assert_eq!(service.active_encoding(), "DEFAULT");
}
