//! Shared scheduling callbacks can reenter without borrowing the scheduler.
use std::{cell::RefCell, rc::Rc};
use xterm_web::{
    links::debounce::FrameScheduler,
    scroll::{
        scrollable::Scrollable,
        state::{DimensionUpdate, PositionUpdate},
    },
};

type Hook = Rc<dyn Fn() -> Result<(), &'static str>>;
#[derive(Default)]
struct Frames {
    next: RefCell<u32>,
    requested: RefCell<Vec<u32>>,
    canceled: RefCell<Vec<u32>>,
    request: RefCell<Option<Hook>>,
    cancel: RefCell<Option<Hook>>,
}
#[derive(Clone)]
struct Scheduler(Rc<Frames>);
impl FrameScheduler for Scheduler {
    type Handle = u32;
    type Error = &'static str;
    fn request_frame(&mut self) -> Result<u32, Self::Error> {
        let hook = self.0.request.borrow().clone();
        if let Some(hook) = hook {
            hook()?;
        }
        let id = *self.0.next.borrow() + 1;
        *self.0.next.borrow_mut() = id;
        self.0.requested.borrow_mut().push(id);
        Ok(id)
    }
    fn cancel_frame(&mut self, handle: u32) -> Result<(), Self::Error> {
        let hook = self.0.cancel.borrow().clone();
        if let Some(hook) = hook {
            hook()?;
        }
        self.0.canceled.borrow_mut().push(handle);
        Ok(())
    }
    fn cleanup_error(&mut self, error: Self::Error) {
        panic!("unexpected cleanup: {error}");
    }
    fn request_frame_shared(scheduler: &RefCell<Self>) -> Result<u32, Self::Error> {
        let mut scheduler = scheduler.borrow().clone();
        scheduler.request_frame()
    }
    fn cancel_frame_shared(scheduler: &RefCell<Self>, handle: u32) -> Result<(), Self::Error> {
        let mut scheduler = scheduler.borrow().clone();
        scheduler.cancel_frame(handle)
    }
}
fn top(value: f64) -> PositionUpdate {
    PositionUpdate {
        top: Some(value),
        ..Default::default()
    }
}
fn model() -> (Scrollable<Scheduler>, Rc<Frames>) {
    let frames = Rc::new(Frames::default());
    let model = Scrollable::new(Scheduler(frames.clone()), false, 90.0, || 1000.0);
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
    (model, frames)
}
#[test]
fn request_retains_assignment_receiver_when_callback_replaces_animation() {
    let (model, frames) = model();
    let weak = model.downgrade();
    let state = Rc::downgrade(&frames);
    *frames.request.borrow_mut() = Some(Rc::new(move || {
        let frames = state.upgrade().unwrap();
        frames.request.borrow_mut().take();
        weak.upgrade()
            .unwrap()
            .set_scroll_position_smooth(top(300.0), false)
            .unwrap();
        Ok(())
    }));
    model.set_scroll_position_smooth(top(100.0), false).unwrap();
    assert_eq!(model.future_scroll_position().top, 300.0);
    assert_eq!(*frames.requested.borrow(), vec![1, 2]);
    model.dispose().unwrap();
    // The outer request belongs to its original operation, not the replacement.
    assert_eq!(*frames.canceled.borrow(), vec![1]);
}
#[test]
fn cancel_callback_can_cancel_again_and_start_a_replacement() {
    let (model, frames) = model();
    model.set_scroll_position_smooth(top(100.0), false).unwrap();
    let weak = model.downgrade();
    let state = Rc::downgrade(&frames);
    *frames.cancel.borrow_mut() = Some(Rc::new(move || {
        let frames = state.upgrade().unwrap();
        frames.cancel.borrow_mut().take();
        weak.upgrade()
            .unwrap()
            .set_scroll_position_smooth(top(300.0), false)
            .unwrap();
        Ok(())
    }));
    model.set_scroll_position_now(top(8.0)).unwrap();
    assert_eq!(model.current_scroll_position().top, 8.0);
    assert!(!model.has_pending_scroll_animation());
    assert_eq!(*frames.requested.borrow(), vec![1, 2]);
    assert_eq!(*frames.canceled.borrow(), vec![1, 1]);
}
#[test]
fn request_error_keeps_reentrant_replacement_and_its_handle() {
    let (model, frames) = model();
    let weak = model.downgrade();
    let state = Rc::downgrade(&frames);
    *frames.request.borrow_mut() = Some(Rc::new(move || {
        let frames = state.upgrade().unwrap();
        frames.request.borrow_mut().take();
        weak.upgrade()
            .unwrap()
            .set_scroll_position_smooth(top(300.0), false)
            .unwrap();
        Err("request-token")
    }));
    assert!(matches!(
        model.set_scroll_position_smooth(top(100.0), false),
        Err(xterm_web::scroll::scrollable::ScrollError::Scheduler(
            "request-token"
        ))
    ));
    assert_eq!(model.future_scroll_position().top, 300.0);
    model.dispose().unwrap();
    assert_eq!(*frames.canceled.borrow(), vec![1]);
}
#[test]
fn cancel_error_preserves_reentrant_state_and_retry_handle() {
    let (model, frames) = model();
    model.set_scroll_position_smooth(top(100.0), false).unwrap();
    let weak = model.downgrade();
    let state = Rc::downgrade(&frames);
    *frames.cancel.borrow_mut() = Some(Rc::new(move || {
        state.upgrade().unwrap().cancel.borrow_mut().take();
        weak.upgrade()
            .unwrap()
            .set_scroll_dimensions(
                DimensionUpdate {
                    height: Some(120.0),
                    ..Default::default()
                },
                false,
            )
            .unwrap();
        Err("cancel-token")
    }));
    assert!(matches!(
        model.set_scroll_position_now(top(8.0)),
        Err(xterm_web::scroll::scrollable::ScrollError::Scheduler(
            "cancel-token"
        ))
    ));
    assert_eq!(model.scroll_state().height, 120.0);
    assert_eq!(model.current_scroll_position().top, 0.0);
    assert!(model.has_pending_scroll_animation());
    model.dispose().unwrap();
    assert_eq!(*frames.canceled.borrow(), vec![1]);
}
