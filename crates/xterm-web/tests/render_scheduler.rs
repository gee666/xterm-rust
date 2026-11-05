use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_web::links::debounce::{FrameScheduler, RenderDebouncer};

type Action = Rc<dyn Fn()>;
#[derive(Clone, Default)]
struct Scheduler {
    next: Rc<Cell<u32>>,
    log: Rc<RefCell<Vec<String>>>,
    request: Rc<RefCell<Option<Action>>>,
    cancel: Rc<RefCell<Option<Action>>>,
    fail_request: Rc<Cell<bool>>,
    fail_cancel: Rc<Cell<bool>>,
}
impl Scheduler {
    fn request(&self) -> Result<u32, &'static str> {
        let id = self.next.get() + 1;
        self.next.set(id);
        self.log.borrow_mut().push(format!("request:{id}"));
        let action = self.request.borrow_mut().take();
        if let Some(action) = action {
            action();
        }
        if self.fail_request.replace(false) {
            return Err("request token");
        }
        Ok(id)
    }
    fn cancel(&self, id: u32) -> Result<(), &'static str> {
        self.log.borrow_mut().push(format!("cancel:{id}"));
        let action = self.cancel.borrow_mut().take();
        if let Some(action) = action {
            action();
        }
        if self.fail_cancel.replace(false) {
            return Err("cancel token");
        }
        Ok(())
    }
}
impl FrameScheduler for Scheduler {
    type Handle = u32;
    type Error = &'static str;
    fn request_frame(&mut self) -> Result<u32, Self::Error> {
        self.request()
    }
    fn cancel_frame(&mut self, id: u32) -> Result<(), Self::Error> {
        self.cancel(id)
    }
    fn request_frame_shared(scheduler: &RefCell<Self>) -> Result<u32, Self::Error> {
        let scheduler = scheduler.borrow().clone();
        scheduler.request()
    }
    fn cancel_frame_shared(scheduler: &RefCell<Self>, id: u32) -> Result<(), Self::Error> {
        let scheduler = scheduler.borrow().clone();
        scheduler.cancel(id)
    }
    fn cleanup_error(&mut self, error: Self::Error) {
        panic!("{error}");
    }
}
#[test]
fn nested_request_retains_outer_handle_and_combined_rows() {
    let scheduler = Scheduler::default();
    let rows = Rc::new(RefCell::new(Vec::new()));
    let out = rows.clone();
    let model = RenderDebouncer::new(scheduler.clone(), move |_, r| out.borrow_mut().push(r));
    let nested = model.clone();
    *scheduler.request.borrow_mut() = Some(Rc::new(move || {
        nested.refresh(Some(4.0), Some(6.0), 10.0).unwrap();
    }));
    assert_eq!(model.add_refresh_callback(|_, _| {}).unwrap(), 1);
    assert_eq!(scheduler.next.get(), 2);
    model.dispose().unwrap();
    assert_eq!(
        *scheduler.log.borrow(),
        ["request:1", "request:2", "cancel:1"]
    );
    model.on_frame().unwrap();
    assert_eq!(rows.borrow()[0].start, 4.0);
    assert_eq!(rows.borrow()[0].end, 6.0);
    model.dispose().unwrap();
}
#[test]
fn nested_cancel_observes_old_handle_until_outer_call_returns() {
    let scheduler = Scheduler::default();
    let model = RenderDebouncer::new(scheduler.clone(), |_, _| {});
    model.refresh(None, None, 10.0).unwrap();
    let nested = model.clone();
    *scheduler.cancel.borrow_mut() = Some(Rc::new(move || nested.dispose().unwrap()));
    model.dispose().unwrap();
    assert!(!model.has_pending_frame());
    assert_eq!(
        *scheduler.log.borrow(),
        ["request:1", "cancel:1", "cancel:1"]
    );
}
#[test]
fn failed_cancel_preserves_reentrant_replacement_handle() {
    let scheduler = Scheduler::default();
    let model = RenderDebouncer::new(scheduler.clone(), |_, _| {});
    model.refresh(None, None, 10.0).unwrap();
    let nested = model.clone();
    *scheduler.cancel.borrow_mut() = Some(Rc::new(move || {
        nested.on_frame().unwrap();
        nested.refresh(None, None, 10.0).unwrap();
    }));
    scheduler.fail_cancel.set(true);
    assert_eq!(model.dispose(), Err("cancel token"));
    assert!(model.has_pending_frame());
    model.dispose().unwrap();
    assert_eq!(
        *scheduler.log.borrow(),
        ["request:1", "cancel:1", "request:2", "cancel:2"]
    );
}
#[test]
fn successful_cancel_clears_reentrant_replacement_handle() {
    let scheduler = Scheduler::default();
    let model = RenderDebouncer::new(scheduler.clone(), |_, _| {});
    model.refresh(None, None, 10.0).unwrap();
    let nested = model.clone();
    *scheduler.cancel.borrow_mut() = Some(Rc::new(move || {
        nested.on_frame().unwrap();
        nested.refresh(None, None, 10.0).unwrap();
    }));
    model.dispose().unwrap();
    assert!(!model.has_pending_frame());
    assert_eq!(
        *scheduler.log.borrow(),
        ["request:1", "cancel:1", "request:2"]
    );
}
#[test]
fn failed_outer_request_keeps_nested_pending_and_callbacks() {
    let scheduler = Scheduler::default();
    let callbacks = Rc::new(Cell::new(0));
    let model = RenderDebouncer::new(scheduler.clone(), |_, _| {});
    let nested = model.clone();
    let fail = scheduler.fail_request.clone();
    *scheduler.request.borrow_mut() = Some(Rc::new(move || {
        nested.refresh(None, None, 10.0).unwrap();
        fail.set(true);
    }));
    let count = callbacks.clone();
    assert_eq!(
        model.add_refresh_callback(move |_, _| count.set(count.get() + 1)),
        Err("request token")
    );
    assert!(model.has_pending_frame());
    model.on_frame().unwrap();
    assert_eq!(callbacks.get(), 1);
    assert!(!model.has_pending_frame());
}
