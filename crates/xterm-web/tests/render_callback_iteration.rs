use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_web::links::debounce::{FrameScheduler, RenderDebouncer};

#[derive(Clone, Default)]
struct Scheduler(Rc<Cell<u32>>);
impl FrameScheduler for Scheduler {
    type Handle = u32;
    type Error = &'static str;
    fn request_frame(&mut self) -> Result<u32, Self::Error> {
        self.0.set(self.0.get() + 1);
        Ok(self.0.get())
    }
    fn cancel_frame(&mut self, _: u32) -> Result<(), Self::Error> {
        Ok(())
    }
    fn cleanup_error(&mut self, error: Self::Error) {
        panic!("unexpected cleanup error: {error}");
    }
}

type Log = Rc<RefCell<Vec<&'static str>>>;
fn append(d: &RenderDebouncer<Scheduler>, log: &Log, label: &'static str) {
    let log = log.clone();
    d.add_refresh_callback(move |_, time| {
        assert_eq!(time, 0.0);
        log.borrow_mut().push(label);
    })
    .unwrap();
}
fn model() -> (RenderDebouncer<Scheduler>, Log) {
    (
        RenderDebouncer::new(Scheduler::default(), |_, _| {}),
        Rc::default(),
    )
}

#[test]
fn nested_completion_keeps_outer_array_and_live_appends() {
    let (d, log) = model();
    let out = log.clone();
    let entered = Cell::new(false);
    d.add_try_refresh_callback(move |d, _| {
        out.borrow_mut().push("A");
        if !entered.replace(true) {
            append(d, &out, "C");
            d.on_frame()?;
        }
        Ok(())
    })
    .unwrap();
    append(&d, &log, "B");
    d.on_frame().unwrap();
    assert_eq!(*log.borrow(), ["A", "A", "B", "C", "B", "C"]);
    d.on_frame().unwrap();
    assert_eq!(log.borrow().len(), 6);
}

#[test]
fn outer_success_discards_new_list_appended_after_nested_completion() {
    let (d, log) = model();
    let out = log.clone();
    let entered = Cell::new(false);
    d.add_try_refresh_callback(move |d, _| {
        out.borrow_mut().push("A");
        if !entered.replace(true) {
            append(d, &out, "C");
            d.on_frame()?;
            append(d, &out, "D");
        }
        Ok(())
    })
    .unwrap();
    append(&d, &log, "B");
    d.on_frame().unwrap();
    assert!(d.has_pending_frame());
    d.on_frame().unwrap();
    assert_eq!(*log.borrow(), ["A", "A", "B", "C", "B", "C"]);
}

#[test]
fn outer_error_preserves_new_list_after_nested_completion() {
    let (d, log) = model();
    let out = log.clone();
    let entered = Cell::new(false);
    d.add_try_refresh_callback(move |d, _| {
        out.borrow_mut().push("A");
        if !entered.replace(true) {
            append(d, &out, "C");
            d.on_frame()?;
            append(d, &out, "D");
            return Err("outer");
        }
        Ok(())
    })
    .unwrap();
    append(&d, &log, "B");
    assert_eq!(d.on_frame(), Err("outer"));
    d.on_frame().unwrap();
    assert_eq!(*log.borrow(), ["A", "A", "B", "C", "D"]);
}

#[test]
fn caught_inner_error_keeps_original_array_active_for_outer_iteration() {
    let (d, log) = model();
    let out = log.clone();
    let entered = Cell::new(false);
    d.add_try_refresh_callback(move |d, _| {
        out.borrow_mut().push("A");
        if !entered.replace(true) {
            append(d, &out, "C");
            assert_eq!(d.on_frame(), Err("inner"));
            append(d, &out, "D");
        }
        Ok(())
    })
    .unwrap();
    let out = log.clone();
    let thrown = Cell::new(false);
    d.add_try_refresh_callback(move |_, _| {
        out.borrow_mut().push("B");
        if !thrown.replace(true) {
            return Err("inner");
        }
        Ok(())
    })
    .unwrap();
    d.on_frame().unwrap();
    assert_eq!(*log.borrow(), ["A", "A", "B", "B", "C", "D"]);
    d.on_frame().unwrap();
    assert_eq!(log.borrow().len(), 6);
}

#[test]
fn propagated_inner_error_retains_callbacks_for_next_frame() {
    let (d, log) = model();
    let out = log.clone();
    let entered = Cell::new(false);
    d.add_try_refresh_callback(move |d, _| {
        out.borrow_mut().push("A");
        if !entered.replace(true) {
            append(d, &out, "C");
            d.on_frame()?;
        }
        Ok(())
    })
    .unwrap();
    let out = log.clone();
    let thrown = Cell::new(false);
    d.add_try_refresh_callback(move |_, _| {
        out.borrow_mut().push("B");
        if !thrown.replace(true) {
            return Err("inner");
        }
        Ok(())
    })
    .unwrap();
    assert_eq!(d.on_frame(), Err("inner"));
    append(&d, &log, "D");
    d.on_frame().unwrap();
    assert_eq!(*log.borrow(), ["A", "A", "B", "A", "B", "C", "D"]);
}

#[test]
fn dispose_after_nested_completion_does_not_clear_outer_iterator() {
    let (d, log) = model();
    let out = log.clone();
    let entered = Cell::new(false);
    d.add_try_refresh_callback(move |d, _| {
        out.borrow_mut().push("A");
        if !entered.replace(true) {
            append(d, &out, "C");
            d.on_frame()?;
            append(d, &out, "D");
            d.dispose()?;
        }
        Ok(())
    })
    .unwrap();
    append(&d, &log, "B");
    d.on_frame().unwrap();
    assert!(!d.has_pending_frame());
    assert_eq!(*log.borrow(), ["A", "A", "B", "C", "B", "C"]);
    d.on_frame().unwrap();
    assert_eq!(log.borrow().len(), 6);
}
