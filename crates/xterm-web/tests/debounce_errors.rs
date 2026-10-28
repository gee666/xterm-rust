use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_web::links::{
    debounce::{FrameScheduler, RenderDebouncer},
    time_debounce::{TimeBasedDebouncer, TimerScheduler},
};
#[derive(Clone, Default)]
struct Clock {
    now: Rc<Cell<f64>>,
    next: Rc<Cell<u32>>,
    ops: Rc<RefCell<Vec<String>>>,
}
impl FrameScheduler for Clock {
    type Handle = u32;
    type Error = &'static str;
    fn request_frame(&mut self) -> Result<u32, Self::Error> {
        let id = self.next.get() + 1;
        self.next.set(id);
        Ok(id)
    }
    fn cancel_frame(&mut self, id: u32) -> Result<(), Self::Error> {
        self.ops.borrow_mut().push(format!("cancel:{id}"));
        Ok(())
    }
    fn cleanup_error(&mut self, error: Self::Error) {
        self.ops.borrow_mut().push(error.into());
    }
}
impl TimerScheduler for Clock {
    type Handle = u32;
    type Error = &'static str;
    fn now(&self) -> f64 {
        self.now.get()
    }
    fn request_timeout(&mut self, _: f64) -> Result<u32, Self::Error> {
        let id = self.next.get() + 1;
        self.next.set(id);
        Ok(id)
    }
    fn cancel_timeout(&mut self, id: u32) -> Result<(), Self::Error> {
        self.ops.borrow_mut().push(format!("cancel:{id}"));
        Ok(())
    }
    fn cleanup_error(&mut self, error: Self::Error) {
        self.ops.borrow_mut().push(error.into());
    }
}
#[test]
fn failed_frame_render_leaves_callbacks_but_consumes_rows_and_frame() {
    let calls = Rc::new(Cell::new(0));
    let out = calls.clone();
    let d = RenderDebouncer::try_new(Clock::default(), move |_, _| {
        out.set(out.get() + 1);
        Err("render")
    });
    let callback = Rc::new(Cell::new(0));
    let output = callback.clone();
    d.add_refresh_callback(move |_, _| output.set(output.get() + 1))
        .unwrap();
    d.refresh(None, None, 10.0).unwrap();
    assert_eq!(d.on_frame(), Err("render"));
    assert!(!d.has_pending_frame());
    assert_eq!(callback.get(), 0);
    d.on_frame().unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(callback.get(), 1);
}
#[test]
fn failed_refresh_callback_retains_original_array_and_stops_later_callbacks() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let first = log.clone();
    let second = log.clone();
    let fail = Rc::new(Cell::new(true));
    let error = fail.clone();
    let d = RenderDebouncer::new(Clock::default(), |_, _| {});
    d.add_try_refresh_callback(move |_, _| {
        first.borrow_mut().push(1);
        if error.replace(false) {
            Err("callback")
        } else {
            Ok(())
        }
    })
    .unwrap();
    d.add_refresh_callback(move |_, _| second.borrow_mut().push(2))
        .unwrap();
    assert_eq!(d.on_frame(), Err("callback"));
    assert_eq!(*log.borrow(), vec![1]);
    d.on_frame().unwrap();
    assert_eq!(*log.borrow(), vec![1, 1, 2]);
}
#[test]
fn failed_trailing_render_preserves_pending_flags_until_immediate_refresh() {
    let clock = Clock::default();
    let fail = Rc::new(Cell::new(true));
    let error = fail.clone();
    let d = TimeBasedDebouncer::try_new(clock.clone(), 100.0, move |_, _| {
        if error.replace(false) {
            Err("render")
        } else {
            Ok(())
        }
    });
    d.refresh(None, None, 10.0).unwrap();
    clock.now.set(100.0);
    assert_eq!(d.on_timeout(), Err("render"));
    assert!(d.has_pending_timeout());
    clock.now.set(101.0);
    d.refresh(None, None, 10.0).unwrap();
    assert_eq!(clock.next.get(), 1);
    clock.now.set(200.0);
    d.refresh(None, None, 10.0).unwrap();
    assert!(!d.has_pending_timeout());
    assert_eq!(*clock.ops.borrow(), vec!["cancel:1"]);
}
