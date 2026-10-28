// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    num::NonZeroU32,
    rc::Rc,
};
use xterm_core::{
    buffer::BufferService,
    options::{OptionValue, OptionsService},
};
use xterm_web::selection::{
    model::{Dimensions, Position},
    pointer::{self, Modifiers, SelectionMouseOptions},
    service::{self, SelectionPlatform, SelectionService},
};

fn dimensions() -> Dimensions {
    Dimensions {
        columns: NonZeroU32::new(20).unwrap(),
        rows: NonZeroU32::new(20).unwrap(),
        scrollback: 0,
    }
}
fn buffers() -> Rc<BufferService> {
    Rc::new(
        BufferService::new(Rc::new(OptionsService::new(
            false,
            [
                ("cols", OptionValue::Number(20.0)),
                ("rows", OptionValue::Number(20.0)),
            ],
        )))
        .unwrap(),
    )
}
#[test]
fn source_cell_membership_assertions() {
    let start = Position::new(2, 0);
    let end = Position::new(2, 1);
    for (point, expected) in [
        (Position::new(0, 0), false),
        (Position::new(1, 0), false),
        (Position::new(2, 0), true),
        (Position::new(10, 0), true),
        (Position::new(0, 1), true),
        (Position::new(1, 1), true),
        (Position::new(2, 1), false),
    ] {
        assert_eq!(
            service::are_coords_in_selection(point, start, end),
            expected
        );
    }
}
#[test]
fn source_force_alt_requirement_precedes_mac_option() {
    let mut options = SelectionMouseOptions {
        mouse_events_require_alt: true,
        mouse_events_active: true,
        ..SelectionMouseOptions::default()
    };
    let platform = SelectionPlatform::default();
    assert!(pointer::should_force_selection(
        platform,
        options,
        Modifiers::default()
    ));
    assert!(!pointer::should_force_selection(
        platform,
        options,
        Modifiers {
            alt: true,
            shift: false
        }
    ));
    options.mac_option_click_forces_selection = true;
    assert!(!pointer::should_force_selection(
        SelectionPlatform {
            mac: true,
            ..platform
        },
        options,
        Modifiers {
            alt: true,
            shift: false
        }
    ));
}
#[test]
fn source_force_modifier_matrix_and_column_mode() {
    for mac in [false, true] {
        for forces in [false, true] {
            for active in [false, true] {
                for require_alt in [false, true] {
                    for alt in [false, true] {
                        for shift in [false, true] {
                            let platform = SelectionPlatform {
                                mac,
                                ..SelectionPlatform::default()
                            };
                            let options = SelectionMouseOptions {
                                mouse_events_require_alt: require_alt,
                                mac_option_click_forces_selection: forces,
                                mouse_events_active: active,
                            };
                            let expected = if require_alt && active {
                                !alt
                            } else if mac {
                                alt && forces
                            } else {
                                shift
                            };
                            assert_eq!(
                                pointer::should_force_selection(
                                    platform,
                                    options,
                                    Modifiers { alt, shift }
                                ),
                                expected
                            );
                            assert_eq!(
                                pointer::should_column_select(
                                    platform,
                                    options,
                                    Modifiers { alt, shift }
                                ),
                                !(require_alt && active) && alt && !(mac && forces)
                            );
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn drag_scroll_threshold_rounding_and_bounds() {
    for (offset, expected) in [
        (-100.0, -15.0),
        (-50.0, -15.0),
        (-25.0, -8.0),
        (-1.0, -1.0),
        (0.0, 0.0),
        (100.0, 0.0),
        (101.0, 1.0),
        (125.0, 8.0),
        (150.0, 15.0),
        (200.0, 15.0),
    ] {
        assert_eq!(pointer::drag_scroll_amount(offset, 100.0), expected);
    }
}
#[test]
fn rejected_frame_request_does_not_prevent_the_next_refresh() {
    let buffers = buffers();
    let service = SelectionService::new(SelectionPlatform::default());
    let count = Rc::new(Cell::new(0));
    let calls = Rc::clone(&count);
    service.on_request_refresh().try_subscribe(move |_| {
        calls.set(calls.get() + 1);
        if calls.get() == 1 {
            Err(xterm_core::event::EmitError::callback(
                std::io::Error::other("frame request rejected"),
            ))
        } else {
            Ok(())
        }
    });
    assert!(service.refresh(&buffers, dimensions(), false).is_err());
    service.refresh(&buffers, dimensions(), false).unwrap();
    assert_eq!(count.get(), 2);
}

#[test]
fn clear_order_refresh_coalescing_and_reentrant_state() {
    let buffers = buffers();
    let service = Rc::new(SelectionService::new(SelectionPlatform::default()));
    let order = Rc::new(RefCell::new(Vec::new()));
    for (event, label) in [
        (service.on_stop_drag(), "stop"),
        (service.on_request_refresh(), "schedule"),
        (service.on_selection_change(), "change"),
    ] {
        let order = Rc::clone(&order);
        event.subscribe(move |_| order.borrow_mut().push(label));
    }
    let weak = Rc::downgrade(&service);
    service.on_selection_change().subscribe(move |_| {
        let service = weak.upgrade().unwrap();
        assert_eq!(service.selection_start(), None);
        service.update_model(|model| model.selection_start_length = 2);
    });
    service.clear_selection(&buffers, dimensions()).unwrap();
    assert_eq!(*order.borrow(), ["stop", "schedule", "change"]);
    assert_eq!(service.model().selection_start_length, 2);
    order.borrow_mut().clear();
    service.clear_selection(&buffers, dimensions()).unwrap();
    assert_eq!(*order.borrow(), ["stop", "change"]);
    service.flush_refresh(dimensions()).unwrap();
    service.clear_selection(&buffers, dimensions()).unwrap();
    assert_eq!(
        *order.borrow(),
        ["stop", "change", "stop", "schedule", "change"]
    );
}
#[test]
fn set_selection_change_deduplicates_and_reentry_releases_model() {
    let buffers = buffers();
    let service = Rc::new(SelectionService::new(SelectionPlatform::default()));
    let count = Rc::new(Cell::new(0));
    let counter = Rc::clone(&count);
    let weak = Rc::downgrade(&service);
    service.on_selection_change().subscribe(move |_| {
        counter.set(counter.get() + 1);
        assert!(weak.upgrade().unwrap().has_selection(dimensions()).unwrap());
    });
    service
        .set_selection(&buffers, dimensions(), Position::new(1, 0), 2)
        .unwrap();
    service
        .set_selection(&buffers, dimensions(), Position::new(1, 0), 2)
        .unwrap();
    assert_eq!(count.get(), 1);
    service
        .set_selection(&buffers, dimensions(), Position::new(2, 0), 2)
        .unwrap();
    assert_eq!(count.get(), 2);
    service.dispose().unwrap();
    service
        .set_selection(&buffers, dimensions(), Position::new(3, 0), 2)
        .unwrap();
    assert_eq!(count.get(), 2);
}
