use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_web::{
    links::debounce::FrameScheduler,
    scroll::{
        scrollable::Scrollable,
        state::{DimensionUpdate, PositionUpdate},
    },
};

#[derive(Default)]
struct Frames {
    next: Cell<u32>,
    requested: RefCell<Vec<u32>>,
    canceled: RefCell<Vec<u32>>,
    fail_request: Cell<bool>,
    fail_cancel: Cell<bool>,
    cleanup_errors: RefCell<Vec<&'static str>>,
}
struct Scheduler(Rc<Frames>);
impl FrameScheduler for Scheduler {
    type Handle = u32;
    type Error = &'static str;
    fn request_frame(&mut self) -> Result<u32, &'static str> {
        if self.0.fail_request.get() {
            return Err("request");
        }
        let id = self.0.next.get() + 1;
        self.0.next.set(id);
        self.0.requested.borrow_mut().push(id);
        Ok(id)
    }
    fn cancel_frame(&mut self, id: u32) -> Result<(), &'static str> {
        if self.0.fail_cancel.get() {
            return Err("cancel");
        }
        self.0.canceled.borrow_mut().push(id);
        Ok(())
    }
    fn cleanup_error(&mut self, error: &'static str) {
        self.0.cleanup_errors.borrow_mut().push(error);
    }
}
fn model(duration: f64) -> (Scrollable<Scheduler>, Rc<Cell<f64>>, Rc<Frames>) {
    let now = Rc::new(Cell::new(1000.0));
    let read = now.clone();
    let frames = Rc::new(Frames::default());
    let model = Scrollable::new(Scheduler(frames.clone()), false, duration, move || {
        read.get()
    });
    model
        .set_scroll_dimensions(
            DimensionUpdate {
                width: Some(100.0),
                scroll_width: Some(1000.0),
                height: Some(100.0),
                scroll_height: Some(1000.0),
            },
            false,
        )
        .unwrap();
    (model, now, frames)
}
fn top(value: f64) -> PositionUpdate {
    PositionUpdate {
        top: Some(value),
        ..Default::default()
    }
}

#[test]
fn dimensions_keep_raw_request_and_emit_precise_change_fields() {
    let (model, _, _) = model(0.0);
    model.set_scroll_position_now(top(2000.5)).unwrap();
    assert_eq!(model.current_scroll_position().top, 900.0);
    let observed = Rc::new(RefCell::new(Vec::new()));
    let output = observed.clone();
    model
        .on_scroll()
        .subscribe(move |event| output.borrow_mut().push(*event));
    model
        .set_scroll_dimensions(
            DimensionUpdate {
                scroll_height: Some(4000.0),
                ..Default::default()
            },
            true,
        )
        .unwrap();
    assert_eq!(model.current_scroll_position().top, 2000.5);
    let events = observed.borrow();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].previous.top, 900.0);
    assert!(events[0].changed.scroll_height);
    assert!(events[0].changed.top);
    assert!(!events[0].changed.height);
    assert!(!events[0].in_smooth_scrolling);
}
#[test]
fn smooth_tick_reuse_and_external_position_cancel() {
    let (model, now, frames) = model(90.0);
    model.set_scroll_position_smooth(top(100.0), false).unwrap();
    assert_eq!(model.future_scroll_position().top, 100.0);
    now.set(1040.0);
    model.frame().unwrap();
    assert_eq!(model.current_scroll_position().top, 87.5);
    model.set_scroll_position_smooth(top(200.0), true).unwrap();
    now.set(1090.0);
    model.frame().unwrap();
    assert_eq!(model.current_scroll_position().top, 200.0);
    assert!(!model.has_pending_scroll_animation());
    model.set_scroll_position_smooth(top(400.0), false).unwrap();
    model.set_scroll_position_now(top(5.5)).unwrap();
    assert_eq!(model.current_scroll_position().top, 5.5);
    assert!(!model.has_pending_scroll_animation());
    assert!(!frames.canceled.borrow().is_empty());
}
#[test]
fn identical_smooth_target_does_not_restart_or_cancel() {
    let (model, _, frames) = model(100.0);
    model.set_scroll_position_smooth(top(100.0), false).unwrap();
    model.set_scroll_position_smooth(top(100.0), true).unwrap();
    assert_eq!(frames.requested.borrow().len(), 1);
    assert!(frames.canceled.borrow().is_empty());
    model.dispose().unwrap();
    assert_eq!(*frames.canceled.borrow(), vec![1]);
}
#[test]
fn event_callback_can_cancel_the_running_animation() {
    let (model, _, frames) = model(100.0);
    let weak = model.downgrade();
    let listener = model.on_scroll().subscribe(move |event| {
        if event.in_smooth_scrolling {
            weak.upgrade()
                .unwrap()
                .set_scroll_position_now(top(8.0))
                .unwrap();
        }
    });
    model.set_scroll_position_smooth(top(100.0), false).unwrap();
    model.frame().unwrap();
    assert_eq!(model.current_scroll_position().top, 8.0);
    assert!(!model.has_pending_scroll_animation());
    assert_eq!(frames.requested.borrow().len(), 1);
    listener.dispose();
}
#[test]
fn scheduler_failures_retain_source_partial_state_and_cleanup_is_observable() {
    let (model, _, frames) = model(100.0);
    frames.fail_request.set(true);
    assert!(model.set_scroll_position_smooth(top(100.0), false).is_err());
    assert!(model.has_pending_scroll_animation());
    assert_eq!(model.future_scroll_position().top, 100.0);
    frames.fail_request.set(false);
    model.set_scroll_position_smooth(top(200.0), false).unwrap();
    frames.fail_cancel.set(true);
    assert!(model.set_scroll_position_now(top(1.0)).is_err());
    assert!(model.has_pending_scroll_animation());
    assert_eq!(model.current_scroll_position().top, 0.0);
    drop(model);
    assert_eq!(*frames.cleanup_errors.borrow(), vec!["cancel"]);
}
#[test]
fn dimensions_resolve_against_state_selected_before_getters() {
    let (model, _, _) = model(90.0);
    model.set_scroll_position_now(top(40.0)).unwrap();
    model
        .set_scroll_dimensions_resolving(
            || {
                model.set_scroll_position_now(top(75.0)).unwrap();
                Ok(DimensionUpdate {
                    width: Some(120.0),
                    ..Default::default()
                })
            },
            true,
        )
        .unwrap();
    assert_eq!(model.scroll_state().width, 120.0);
    assert_eq!(model.scroll_state().top, 40.0);
}
#[test]
fn immediate_and_validation_resolve_before_reentrant_state_replacement() {
    let (model, _, _) = model(90.0);
    let change_dimensions = || {
        model
            .set_scroll_dimensions(
                DimensionUpdate {
                    width: Some(200.0),
                    ..Default::default()
                },
                false,
            )
            .unwrap();
        Ok(PositionUpdate {
            left: Some(950.0),
            top: Some(50.0),
        })
    };
    let position = model
        .try_validate_scroll_position_resolving(change_dimensions)
        .unwrap();
    assert_eq!(position.top, 50.0);
    assert_eq!(position.left, 900.0);
    assert_eq!(model.scroll_state().width, 200.0);
    model
        .set_scroll_position_now_resolving(|| {
            model
                .set_scroll_dimensions(
                    DimensionUpdate {
                        width: Some(300.0),
                        ..Default::default()
                    },
                    false,
                )
                .unwrap();
            Ok(top(75.0))
        })
        .unwrap();
    assert_eq!(model.scroll_state().width, 200.0);
    assert_eq!(model.scroll_state().top, 75.0);
}
#[test]
fn smooth_missing_axes_read_each_current_operation_after_getters() {
    let (model, _, _) = model(90.0);
    model.set_scroll_position_smooth(top(100.0), false).unwrap();
    model
        .set_scroll_position_smooth_resolving(
            || {
                model
                    .set_scroll_position_smooth(
                        PositionUpdate {
                            left: Some(150.0),
                            top: Some(200.0),
                        },
                        false,
                    )
                    .unwrap();
                Ok(None)
            },
            || {
                model
                    .set_scroll_position_smooth(
                        PositionUpdate {
                            left: Some(250.0),
                            top: Some(300.0),
                        },
                        false,
                    )
                    .unwrap();
                Ok(None)
            },
            false,
        )
        .unwrap();
    assert_eq!(model.future_scroll_position().left, 150.0);
    assert_eq!(model.future_scroll_position().top, 300.0);
}
#[test]
fn cancelled_smooth_fallback_fails_before_later_getter() {
    let (model, _, _) = model(90.0);
    model.set_scroll_position_smooth(top(100.0), false).unwrap();
    let read_top = Cell::new(false);
    let result = model.set_scroll_position_smooth_resolving(
        || {
            model.set_scroll_position_now(top(7.0)).unwrap();
            Ok(None)
        },
        || {
            read_top.set(true);
            Ok(Some(25.0))
        },
        false,
    );
    assert!(matches!(
        result,
        Err(xterm_web::scroll::scrollable::ScrollError::MissingAnimation)
    ));
    assert!(!read_top.get());
    assert_eq!(model.current_scroll_position().top, 7.0);
}
