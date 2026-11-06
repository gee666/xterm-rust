// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::BufferService,
    options::{OptionValue, OptionsService},
};
use xterm_web::selection::{
    model::Position,
    service::{buffer_dimensions, SelectionPlatform, SelectionService},
};

#[test]
fn selection_dedup_reads_geometry_after_each_callback_phase() {
    for refresh in [false, true] {
        for columns in [10, 32] {
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
            let dimensions = buffer_dimensions(&buffers).unwrap();
            let resized = buffers.clone();
            let event = if refresh {
                service.on_request_refresh()
            } else {
                service.on_stop_drag()
            };
            let callback = event.subscribe(move |_| resized.resize(columns, 4).unwrap());
            let events = Rc::new(RefCell::new(Vec::new()));
            let observed = events.clone();
            let weak = Rc::downgrade(&service);
            let current = buffers.clone();
            service.on_selection_change().subscribe(move |_| {
                let service = weak.upgrade().unwrap();
                observed.borrow_mut().push((
                    service.selection_start(),
                    service
                        .selection_end(buffer_dimensions(&current).unwrap())
                        .unwrap(),
                ));
            });
            service
                .set_selection(&buffers, dimensions, Position::new(8, 0), 15)
                .unwrap();
            callback.dispose();
            let expected_end = if columns == 10 {
                Position::new(3, 2)
            } else {
                Position::new(23, 0)
            };
            assert_eq!(
                *events.borrow(),
                [(Some(Position::new(8, 0)), Some(expected_end))]
            );
            service
                .set_selection(
                    &buffers,
                    buffer_dimensions(&buffers).unwrap(),
                    Position::new(8, 0),
                    15,
                )
                .unwrap();
            assert_eq!(
                events.borrow().len(),
                1,
                "refresh={refresh}, columns={columns}"
            );
        }
    }
}
