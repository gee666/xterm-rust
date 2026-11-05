// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
//! Callback failures must not become a rejection, an empty report or a debounce hit.
use std::{cell::Cell, convert::Infallible, error::Error, fmt, rc::Rc};
use xterm_core::{
    event::EmitError,
    mouse_state::{
        MouseAction, MouseButton, MouseEvent, MouseEventTypes, MouseProtocol, MouseStateService,
    },
};
use xterm_web::input::mouse_policy::{MouseOutput, MousePolicy, MouseRouteError};

#[derive(Debug)]
struct Failure(Rc<()>);
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("original mouse callback failure")
    }
}
impl Error for Failure {}

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
fn service() -> Rc<MouseStateService> {
    let service = Rc::new(MouseStateService::new());
    service.set_active_protocol("ANY").unwrap();
    service.set_active_encoding("SGR").unwrap();
    service
}

#[test]
fn restriction_and_encoding_failures_keep_error_identity_and_do_not_commit_cache() {
    for restriction in [true, false] {
        let service = service();
        let policy = MousePolicy::new();
        let sentinel = Rc::new(());
        let thrown = sentinel.clone();
        if restriction {
            service.add_protocol(
                "ERROR",
                MouseProtocol {
                    events: MouseEventTypes::MOVE,
                    restrict: Rc::new(move |event| {
                        event.ctrl = Some(true);
                        Err(EmitError::callback(Failure(thrown.clone())))
                    }),
                },
            );
            service.set_active_protocol("ERROR").unwrap();
        } else {
            service.add_encoding(
                "ERROR",
                Rc::new(move |event| {
                    event.ctrl = Some(true);
                    Err(EmitError::callback(Failure(thrown.clone())))
                }),
            );
            service.set_active_encoding("ERROR").unwrap();
        }
        let mut input = event();
        let result = policy.trigger(
            &service,
            &mut input,
            10.0,
            4.0,
            |_| -> Result<(), Infallible> { panic!("failed callback emitted a report") },
        );
        let error = result.unwrap_err();
        let MouseRouteError::Callback(callback) = &error else {
            panic!("wrong error: {error}")
        };
        assert!(Rc::ptr_eq(
            &callback.callback_ref::<Failure>().unwrap().0,
            &sentinel
        ));
        assert!(error.source().unwrap().source().unwrap().is::<Failure>());
        assert_eq!((input.col, input.row, input.ctrl), (1.0, 1.0, Some(true)));
        service.set_active_protocol("ANY").unwrap();
        service.set_active_encoding("SGR").unwrap();
        let mut retry = event();
        retry.ctrl = Some(true);
        let mut outputs = Vec::new();
        assert!(
            policy
                .trigger(&service, &mut retry, 10.0, 4.0, |output| {
                    outputs.push(output);
                    Ok::<_, Infallible>(())
                })
                .unwrap(),
            "failure must not suppress an identical later move"
        );
        assert_eq!(
            outputs,
            [MouseOutput::UserData(
                "\x1b[<51;1;1M".encode_utf16().collect()
            )]
        );
    }
}

#[test]
fn callback_can_replace_itself_reenter_and_then_fail_without_rolling_back_inner_cache() {
    for restriction in [true, false] {
        let service = service();
        let policy = Rc::new(MousePolicy::new());
        let sentinel = EmitError::callback(Failure(Rc::new(())));
        let thrown = sentinel.clone();
        let weak_service = Rc::downgrade(&service);
        let weak_policy = Rc::downgrade(&policy);
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let nested = move || {
            let service = weak_service.upgrade().unwrap();
            service.set_active_protocol("ANY").unwrap();
            service.set_active_encoding("SGR").unwrap();
            assert!(weak_policy
                .upgrade()
                .unwrap()
                .trigger(&service, &mut event(), 10.0, 4.0, |_| {
                    observed.set(observed.get() + 1);
                    Ok::<_, Infallible>(())
                })
                .unwrap());
        };
        if restriction {
            service.add_protocol(
                "ERROR",
                MouseProtocol {
                    events: MouseEventTypes::MOVE,
                    restrict: Rc::new(move |_| {
                        nested();
                        Err(thrown.clone())
                    }),
                },
            );
            service.set_active_protocol("ERROR").unwrap();
        } else {
            service.add_encoding(
                "ERROR",
                Rc::new(move |_| {
                    nested();
                    Err(thrown.clone())
                }),
            );
            service.set_active_encoding("ERROR").unwrap();
        }
        let result = policy.trigger(
            &service,
            &mut event(),
            10.0,
            4.0,
            |_| -> Result<(), Infallible> { panic!("outer failed report must not emit") },
        );
        assert_eq!(result, Err(MouseRouteError::Callback(sentinel)));
        assert_eq!(calls.get(), 1);
        assert!(!policy
            .trigger(
                &service,
                &mut event(),
                10.0,
                4.0,
                |_| -> Result<(), Infallible> { panic!("successful nested event remains cached") }
            )
            .unwrap());
    }
}
