use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_web::links::time_debounce::{TimeBasedDebouncer, TimerScheduler};

type Action = Rc<dyn Fn()>;
#[derive(Clone, Default)]
struct Host {
    time: Rc<Cell<f64>>,
    next: Rc<Cell<u32>>,
    clock: Rc<RefCell<Option<Action>>>,
    request: Rc<RefCell<Option<Action>>>,
    cancel: Rc<RefCell<Option<Action>>>,
    ids: Rc<RefCell<Vec<Option<u32>>>>,
    fail: Rc<Cell<bool>>,
}
fn trigger(slot: &RefCell<Option<Action>>) {
    let action = slot.borrow_mut().take();
    if let Some(action) = action {
        action();
    }
}
impl TimerScheduler for Host {
    type Handle = u32;
    type Error = &'static str;
    fn now(&self) -> f64 {
        self.time.get()
    }
    fn request_timeout(&mut self, _: f64) -> Result<u32, Self::Error> {
        let id = self.next.get() + 1;
        self.next.set(id);
        trigger(&self.request);
        Ok(id)
    }
    fn cancel_timeout(&mut self, id: u32) -> Result<(), Self::Error> {
        self.ids.borrow_mut().push(Some(id));
        Ok(())
    }
    fn cleanup_error(&mut self, error: Self::Error) {
        panic!("{error}");
    }
    fn now_shared(host: &RefCell<Self>) -> Result<f64, Self::Error> {
        let host = host.borrow().clone();
        trigger(&host.clock);
        Ok(host.time.get())
    }
    fn request_timeout_shared(host: &RefCell<Self>, delay: f64) -> Result<u32, Self::Error> {
        let mut host = host.borrow().clone();
        host.request_timeout(delay)
    }
    fn cancel_timeout_resolved(
        host: &RefCell<Self>,
        resolve: impl FnOnce() -> Option<u32>,
    ) -> Result<(), Self::Error> {
        let host = host.borrow().clone();
        trigger(&host.cancel);
        host.ids.borrow_mut().push(resolve());
        if host.fail.replace(false) {
            Err("cancel token")
        } else {
            Ok(())
        }
    }
}
#[test]
fn clock_can_refresh_before_outer_clock_read_completes() {
    let host = Host::default();
    let rows = Rc::new(RefCell::new(Vec::new()));
    let out = rows.clone();
    let model = TimeBasedDebouncer::new(host.clone(), 100.0, move |_, r| out.borrow_mut().push(r));
    let nested = model.clone();
    *host.clock.borrow_mut() = Some(Rc::new(move || {
        nested.refresh(Some(1.0), Some(8.0), 10.0).unwrap()
    }));
    model.refresh(Some(3.0), Some(5.0), 10.0).unwrap();
    assert_eq!(host.next.get(), 1);
    model.on_timeout().unwrap();
    assert_eq!((rows.borrow()[0].start, rows.borrow()[0].end), (1.0, 8.0));
}
#[test]
fn request_can_dispose_and_install_nested_timeout_before_outer_assignment() {
    let host = Host::default();
    let model = TimeBasedDebouncer::new(host.clone(), 100.0, |_, _| {});
    let nested = model.clone();
    *host.request.borrow_mut() = Some(Rc::new(move || {
        nested.dispose().unwrap();
        nested.refresh(None, None, 10.0).unwrap();
    }));
    model.refresh(None, None, 10.0).unwrap();
    assert_eq!(host.next.get(), 2);
    model.dispose().unwrap();
    assert_eq!(*host.ids.borrow(), [Some(1)]);
}
fn cancellation(replace: bool, fail: bool) {
    let host = Host::default();
    let model = TimeBasedDebouncer::new(host.clone(), 100.0, |_, _| {});
    model.refresh(None, None, 10.0).unwrap();
    let nested = model.clone();
    *host.cancel.borrow_mut() = Some(Rc::new(move || {
        nested.dispose().unwrap();
        if replace {
            nested.refresh(None, None, 10.0).unwrap();
        }
    }));
    host.fail.set(fail);
    // The inner cancellation consumes fail if set, so arm it only after reentry.
    if fail {
        host.fail.set(false);
        let action = host.cancel.borrow_mut().take().unwrap();
        let flag = host.fail.clone();
        *host.cancel.borrow_mut() = Some(Rc::new(move || {
            action();
            flag.set(true);
        }));
    }
    assert_eq!(
        model.dispose(),
        if fail { Err("cancel token") } else { Ok(()) }
    );
    assert_eq!(
        *host.ids.borrow(),
        [Some(1), if replace { Some(2) } else { None }]
    );
    assert_eq!(model.has_pending_timeout(), fail && replace);
    model.dispose().unwrap();
}
#[test]
fn cancellation_resolves_cleared_id_after_reentrant_getter() {
    cancellation(false, false);
}
#[test]
fn cancellation_resolves_replacement_id_then_clears_it() {
    cancellation(true, false);
}
#[test]
fn failed_cancellation_retains_reentrant_replacement() {
    cancellation(true, true);
}
#[test]
fn immediate_refresh_cancellation_can_reenter_dispose() {
    let host = Host::default();
    let model = TimeBasedDebouncer::new(host.clone(), 100.0, |_, _| {});
    model.refresh(None, None, 10.0).unwrap();
    let nested = model.clone();
    *host.cancel.borrow_mut() = Some(Rc::new(move || nested.dispose().unwrap()));
    host.time.set(101.0);
    model.refresh(None, None, 10.0).unwrap();
    assert_eq!(*host.ids.borrow(), [Some(1), None]);
    assert!(!model.has_pending_timeout());
}
