//! Numeric callers use the same fallible clock phases as raw browser values.
use std::{
    cell::{Cell, RefCell},
    convert::Infallible,
    rc::Rc,
};
use xterm_web::{
    links::debounce::FrameScheduler,
    scroll::{
        animation::SmoothOperation,
        scrollable::Scrollable,
        state::{DimensionUpdate, PositionUpdate, ScrollState},
    },
};

#[derive(Default)]
struct Scheduler(u32);
impl FrameScheduler for Scheduler {
    type Handle = u32;
    type Error = Infallible;
    fn request_frame(&mut self) -> Result<u32, Infallible> {
        self.0 += 1;
        Ok(self.0)
    }
    fn cancel_frame(&mut self, _: u32) -> Result<(), Infallible> {
        Ok(())
    }
    fn cleanup_error(&mut self, never: Infallible) {
        match never {}
    }
}
fn top(value: f64) -> PositionUpdate {
    PositionUpdate {
        top: Some(value),
        ..Default::default()
    }
}
type Reaction = Rc<RefCell<Option<Box<dyn FnOnce()>>>>;
fn model(now: Rc<Cell<f64>>, reaction: Reaction) -> Scrollable<Scheduler> {
    let model = Scrollable::try_new_with_clock(Scheduler::default(), false, 90.0, move || {
        let callback = reaction.borrow_mut().take();
        if let Some(callback) = callback {
            callback();
        }
        Ok(now.get())
    })
    .unwrap();
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
    model
}
#[test]
fn standalone_raw_clock_entrypoints_keep_numeric_animation() {
    let state = |top| ScrollState::new(false, 100.0, 1000.0, 0.0, 100.0, 1000.0, top);
    let calls = Cell::new(0);
    let operation = SmoothOperation::try_start_with_clock(state(0.0), state(100.0), 90.0, || {
        calls.set(calls.get() + 1);
        Ok(1000.0)
    })
    .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(operation.start_time, 990.0);
    assert_eq!(operation.duration, 100.0);
    let update = operation.try_tick_with_clock(|| Ok(1040.0)).unwrap();
    assert_eq!(update.position.top, 87.5);
    assert!(!update.done);
}
#[test]
fn construction_selects_from_before_clock_reentry() {
    let now = Rc::new(Cell::new(1000.0));
    let reaction: Reaction = Rc::default();
    let model = model(now.clone(), reaction.clone());
    let weak = model.downgrade();
    *reaction.borrow_mut() = Some(Box::new(move || {
        weak.upgrade()
            .unwrap()
            .set_scroll_position_now(top(50.0))
            .unwrap();
    }));
    model.set_scroll_position_smooth(top(100.0), false).unwrap();
    assert_eq!(model.current_scroll_position().top, 50.0);
    now.set(1040.0);
    model.frame().unwrap();
    assert_eq!(model.current_scroll_position().top, 87.5);
}
#[test]
fn tick_keeps_receiver_when_clock_stops_current_animation() {
    let now = Rc::new(Cell::new(1000.0));
    let reaction: Reaction = Rc::default();
    let model = model(now.clone(), reaction.clone());
    model.set_scroll_position_smooth(top(100.0), false).unwrap();
    let weak = model.downgrade();
    *reaction.borrow_mut() = Some(Box::new(move || {
        weak.upgrade()
            .unwrap()
            .set_scroll_position_now(top(5.0))
            .unwrap();
    }));
    now.set(1040.0);
    model.frame().unwrap();
    assert_eq!(model.current_scroll_position().top, 87.5);
    assert!(!model.has_pending_scroll_animation());
}
