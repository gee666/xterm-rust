// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    num::NonZeroU32,
    rc::Rc,
};
use xterm_core::{
    buffer::BufferService,
    event::EmitError,
    options::{OptionValue, OptionsService},
};
use xterm_web::selection::{
    model::{Dimensions, Position, SelectionModel},
    service::{SelectionError, SelectionPlatform, SelectionService},
};

fn setup() -> (Rc<SelectionService>, Rc<BufferService>, Dimensions) {
    let buffers = Rc::new(
        BufferService::new(Rc::new(OptionsService::new(
            false,
            [
                ("cols", OptionValue::Number(20.0)),
                ("rows", OptionValue::Number(4.0)),
            ],
        )))
        .unwrap(),
    );
    let service = Rc::new(SelectionService::new(SelectionPlatform::default()));
    service.update_model(|model| {
        model.selection_start = Some(Position::new(1, 1));
        model.selection_end = Some(Position::new(8, 2));
        model.selection_start_length = 4;
        model.is_select_all_active = true;
    });
    (
        service,
        buffers,
        Dimensions {
            columns: NonZeroU32::new(20).unwrap(),
            rows: NonZeroU32::new(4).unwrap(),
            scrollback: 0,
        },
    )
}

#[test]
fn set_selection_clears_before_teardown_then_refreshes_and_notifies() {
    let (service, buffers, dimensions) = setup();
    let events = Rc::new(RefCell::new(Vec::new()));
    let weak = Rc::downgrade(&service);
    let log = events.clone();
    service.on_stop_drag().subscribe(move |_| {
        let service = weak.upgrade().unwrap();
        assert_eq!(service.model(), SelectionModel::default());
        log.borrow_mut().push("stop");
    });
    for (event, label) in [
        (service.on_request_refresh(), "refresh"),
        (service.on_selection_change(), "change"),
    ] {
        let log = events.clone();
        let weak = Rc::downgrade(&service);
        event.subscribe(move |_| {
            assert_eq!(
                weak.upgrade().unwrap().model().selection_start,
                Some(Position::new(2, 0))
            );
            log.borrow_mut().push(label);
        });
    }
    service
        .set_selection(&buffers, dimensions, Position::new(2, 0), 3)
        .unwrap();
    assert_eq!(*events.borrow(), ["stop", "refresh", "change"]);
    assert_eq!(
        service.selection_end(dimensions).unwrap(),
        Some(Position::new(5, 0))
    );
}

#[test]
fn teardown_failure_retains_cleared_model_and_exact_error_without_events() {
    let (service, buffers, dimensions) = setup();
    let marker = EmitError::callback(std::io::Error::other("teardown marker"));
    let thrown = marker.clone();
    service
        .on_stop_drag()
        .try_subscribe(move |_| Err(thrown.clone()));
    let count = Rc::new(Cell::new(0));
    for event in [service.on_request_refresh(), service.on_selection_change()] {
        let count = count.clone();
        event.subscribe(move |_| count.set(count.get() + 1));
    }
    let error = service
        .set_selection(&buffers, dimensions, Position::new(2, 0), 3)
        .unwrap_err();
    assert!(matches!(error, SelectionError::Dispatch(error) if error == marker));
    assert_eq!(service.model(), SelectionModel::default());
    assert_eq!(count.get(), 0);
}

#[test]
fn teardown_mutation_survives_a_later_callback_error() {
    let (service, buffers, dimensions) = setup();
    let marker = EmitError::callback(std::io::Error::other("mutated teardown marker"));
    let thrown = marker.clone();
    let weak = Rc::downgrade(&service);
    let changed = SelectionModel {
        selection_start: Some(Position::new(7, 2)),
        selection_start_length: 8,
        selection_end: Some(Position::new(9, 1)),
        is_select_all_active: true,
    };
    let observed = changed.clone();
    service.on_stop_drag().try_subscribe(move |_| {
        let service = weak.upgrade().unwrap();
        assert_eq!(service.model(), SelectionModel::default());
        service.update_model(|model| *model = observed.clone());
        Err(thrown.clone())
    });
    let error = service
        .set_selection(&buffers, dimensions, Position::new(2, 0), 3)
        .unwrap_err();
    assert!(matches!(error, SelectionError::Dispatch(error) if error == marker));
    assert_eq!(service.model(), changed);
}

#[test]
fn late_assignment_preserves_reentrant_end_and_select_all_fields() {
    for all in [false, true] {
        for end in [None, Some(Position::new(9, 1))] {
            let (service, buffers, dimensions) = setup();
            let weak = Rc::downgrade(&service);
            service.on_stop_drag().subscribe(move |_| {
                let service = weak.upgrade().unwrap();
                assert_eq!(service.model(), SelectionModel::default());
                service.update_model(|model| {
                    model.selection_start = Some(Position::new(7, 2));
                    model.selection_start_length = 8;
                    model.selection_end = end;
                    model.is_select_all_active = all;
                });
            });
            service
                .set_selection(&buffers, dimensions, Position::new(2, 0), 3)
                .unwrap();
            assert_eq!(
                service.model(),
                SelectionModel {
                    selection_start: Some(Position::new(2, 0)),
                    selection_start_length: 3,
                    selection_end: end,
                    is_select_all_active: all,
                }
            );
        }
    }
}

#[test]
fn nested_set_selection_finishes_before_outer_target_assignment() {
    let (service, buffers, dimensions) = setup();
    let nested = Rc::new(Cell::new(false));
    let guard = nested.clone();
    let weak = Rc::downgrade(&service);
    let nested_buffers = buffers.clone();
    service.on_stop_drag().subscribe(move |_| {
        let service = weak.upgrade().unwrap();
        assert_eq!(service.model(), SelectionModel::default());
        if !guard.replace(true) {
            service
                .set_selection(&nested_buffers, dimensions, Position::new(7, 0), 2)
                .unwrap();
        }
    });
    let observations = Rc::new(RefCell::new(Vec::new()));
    let weak = Rc::downgrade(&service);
    let log = observations.clone();
    service.on_selection_change().subscribe(move |_| {
        log.borrow_mut()
            .push(weak.upgrade().unwrap().model().selection_start);
    });
    service
        .set_selection(&buffers, dimensions, Position::new(2, 0), 3)
        .unwrap();
    assert!(nested.get());
    assert_eq!(
        *observations.borrow(),
        [Some(Position::new(7, 0)), Some(Position::new(2, 0))]
    );
    assert_eq!(service.model().selection_start_length, 3);
}
