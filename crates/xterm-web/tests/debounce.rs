use std::{
    cell::{Cell, RefCell},
    convert::Infallible,
    rc::Rc,
};
use xterm_web::links::{
    debounce::{FrameScheduler, RenderDebouncer, RenderRange},
    time_debounce::{TimeBasedDebouncer, TimerScheduler},
};
#[derive(Clone, Default)]
struct Clock {
    now: Rc<Cell<f64>>,
    next: Rc<Cell<u32>>,
    operations: Rc<RefCell<Vec<String>>>,
}
impl Clock {
    fn request(&self, label: &str) -> u32 {
        let id = self.next.get() + 1;
        self.next.set(id);
        self.operations.borrow_mut().push(format!("{label}:{id}"));
        id
    }
}
impl FrameScheduler for Clock {
    type Handle = u32;
    type Error = Infallible;
    fn request_frame(&mut self) -> Result<u32, Infallible> {
        Ok(self.request("frame"))
    }
    fn cancel_frame(&mut self, id: u32) -> Result<(), Infallible> {
        self.operations.borrow_mut().push(format!("cancel:{id}"));
        Ok(())
    }
    fn cleanup_error(&mut self, error: Infallible) {
        match error {}
    }
}
impl TimerScheduler for Clock {
    type Handle = u32;
    type Error = Infallible;
    fn now(&self) -> f64 {
        self.now.get()
    }
    fn request_timeout(&mut self, delay: f64) -> Result<u32, Infallible> {
        Ok(self.request(&format!("timeout:{delay}")))
    }
    fn cancel_timeout(&mut self, id: u32) -> Result<(), Infallible> {
        self.operations.borrow_mut().push(format!("cancel:{id}"));
        Ok(())
    }
    fn cleanup_error(&mut self, error: Infallible) {
        match error {}
    }
}
#[test]
fn frame_coalesces_clamps_and_uses_latest_count() {
    let clock = Clock::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let output = log.clone();
    let d = RenderDebouncer::new(clock.clone(), move |_, r| output.borrow_mut().push(r));
    d.refresh(Some(-3.0), Some(50.0), 80.0).unwrap();
    d.refresh(Some(2.0), Some(6.0), 10.0).unwrap();
    assert_eq!(clock.next.get(), 1);
    d.on_frame().unwrap();
    assert_eq!(
        *log.borrow(),
        vec![RenderRange {
            start: 0.0,
            end: 9.0
        }]
    );
    assert!(!d.has_pending_frame());
}
#[test]
fn callback_only_frame_is_ordered_and_receives_zero() {
    let clock = Clock::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let d = RenderDebouncer::new(clock.clone(), |_, _| {
        panic!("callback-only must not render")
    });
    for n in 0..3 {
        let log = log.clone();
        d.add_refresh_callback(move |_, time| log.borrow_mut().push((n, time)))
            .unwrap();
    }
    assert_eq!(clock.next.get(), 1);
    d.on_frame().unwrap();
    assert_eq!(*log.borrow(), vec![(0, 0.0), (1, 0.0), (2, 0.0)]);
}
#[test]
fn reentrant_render_preserves_next_frame_rows() {
    let clock = Clock::default();
    let count = Rc::new(Cell::new(0));
    let calls = count.clone();
    let d = RenderDebouncer::new(clock.clone(), move |d, r| {
        calls.set(calls.get() + 1);
        if calls.get() == 1 {
            d.refresh(Some(4.0), Some(5.0), 10.0).unwrap();
        } else {
            assert_eq!(
                r,
                RenderRange {
                    start: 4.0,
                    end: 5.0
                }
            );
        }
    });
    d.refresh(None, None, 10.0).unwrap();
    d.on_frame().unwrap();
    assert!(d.has_pending_frame());
    assert_eq!(clock.next.get(), 2);
    d.on_frame().unwrap();
    assert_eq!(count.get(), 2);
}
#[test]
fn callbacks_added_during_callback_are_visited_in_same_frame() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let first = log.clone();
    let d = RenderDebouncer::new(Clock::default(), |_, _| {});
    d.add_refresh_callback(move |d, _| {
        first.borrow_mut().push(1);
        let second = first.clone();
        d.add_refresh_callback(move |_, _| second.borrow_mut().push(2))
            .unwrap();
    })
    .unwrap();
    d.on_frame().unwrap();
    assert_eq!(*log.borrow(), vec![1, 2]);
    // The reentrant registration requested a new frame before being visited.
    assert!(d.has_pending_frame());
    d.on_frame().unwrap();
    assert_eq!(log.borrow().len(), 2);
}
#[test]
fn disposal_retains_callbacks_and_rows_for_reuse() {
    let clock = Clock::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let output = log.clone();
    let d = RenderDebouncer::new(clock.clone(), move |_, r| output.borrow_mut().push(r));
    d.refresh(Some(2.0), Some(3.0), 10.0).unwrap();
    d.dispose().unwrap();
    d.dispose().unwrap();
    d.refresh(Some(5.0), Some(6.0), 10.0).unwrap();
    d.on_frame().unwrap();
    assert_eq!(
        *log.borrow(),
        vec![RenderRange {
            start: 2.0,
            end: 6.0
        }]
    );
    assert_eq!(
        *clock.operations.borrow(),
        vec!["frame:1", "cancel:1", "frame:2"]
    );
}
#[test]
fn last_handle_drop_cancels_but_clone_drop_does_not() {
    let clock = Clock::default();
    let d = RenderDebouncer::new(clock.clone(), |_, _| {});
    d.refresh(None, None, 10.0).unwrap();
    let cloned = d.clone();
    drop(d);
    assert_eq!(clock.operations.borrow().len(), 1);
    drop(cloned);
    assert_eq!(clock.operations.borrow().last().unwrap(), "cancel:1");
}
#[test]
fn time_threshold_first_wait_trailing_coalescence_and_equality() {
    let clock = Clock::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let output = log.clone();
    let d = TimeBasedDebouncer::new(clock.clone(), 1000.0, move |_, r| {
        output.borrow_mut().push(r)
    });
    clock.now.set(100.0);
    d.refresh(Some(2.0), Some(3.0), 10.0).unwrap();
    clock.now.set(200.0);
    d.refresh(Some(1.0), Some(8.0), 6.0).unwrap();
    assert_eq!(*clock.operations.borrow(), vec!["timeout:900:1"]);
    clock.now.set(1000.0);
    d.on_timeout().unwrap();
    assert_eq!(
        *log.borrow(),
        vec![RenderRange {
            start: 1.0,
            end: 5.0
        }]
    );
    clock.now.set(2000.0);
    d.refresh(None, None, 4.0).unwrap();
    assert_eq!(
        log.borrow()[1],
        RenderRange {
            start: 0.0,
            end: 3.0
        }
    );
    assert!(!d.has_pending_timeout());
}
#[test]
fn immediate_refresh_cancels_pending_timeout() {
    let clock = Clock::default();
    let d = TimeBasedDebouncer::new(clock.clone(), 100.0, |_, _| {});
    d.refresh(None, None, 10.0).unwrap();
    clock.now.set(100.0);
    d.refresh(None, None, 10.0).unwrap();
    assert!(!d.has_pending_timeout());
    assert_eq!(
        *clock.operations.borrow(),
        vec!["timeout:100:1", "cancel:1"]
    );
}
#[test]
fn trailing_callback_reentrancy_does_not_schedule_until_next_request() {
    let clock = Clock::default();
    let count = Rc::new(Cell::new(0));
    let calls = count.clone();
    let d = TimeBasedDebouncer::new(clock.clone(), 100.0, move |d, _| {
        calls.set(calls.get() + 1);
        if calls.get() == 1 {
            d.refresh(Some(4.0), Some(5.0), 10.0).unwrap();
        }
    });
    d.refresh(None, None, 10.0).unwrap();
    clock.now.set(100.0);
    d.on_timeout().unwrap();
    assert_eq!(clock.next.get(), 1);
    assert!(!d.has_pending_timeout());
    clock.now.set(101.0);
    d.refresh(Some(6.0), Some(7.0), 10.0).unwrap();
    assert_eq!(clock.next.get(), 2);
    clock.now.set(200.0);
    d.on_timeout().unwrap();
    assert_eq!(count.get(), 2);
}
