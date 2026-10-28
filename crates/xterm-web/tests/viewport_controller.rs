// Copyright (c) 2024 The xterm.js authors. All rights reserved. Licensed under MIT.
// Source Viewport.ts method/flag checkpoints, not a DOM scroller substitute.
use std::{
    cell::{Cell, RefCell},
    error::Error,
    fmt,
    rc::Rc,
};
use xterm_core::event::EmitError;
use xterm_web::viewport::{SyncCallback, ViewportController, ViewportError, ViewportHost};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Failure(&'static str);
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for Failure {}
type Hook = Rc<dyn Fn(&str)>;
struct Host {
    row: Cell<f64>,
    cell: Cell<f64>,
    canvas: Cell<f64>,
    count: Cell<f64>,
    synchronized: Cell<bool>,
    top: Cell<f64>,
    log: RefCell<Vec<&'static str>>,
    writes: RefCell<Vec<(f64, Option<bool>)>>,
    dims: RefCell<Vec<(f64, f64)>>,
    callbacks: RefCell<Vec<SyncCallback<Failure>>>,
    hook: RefCell<Option<Hook>>,
    fail: Cell<Option<&'static str>>,
    immediate: Cell<bool>,
}
impl Default for Host {
    fn default() -> Self {
        Self {
            row: Cell::new(2.0),
            cell: Cell::new(10.0),
            canvas: Cell::new(40.0),
            count: Cell::new(20.0),
            synchronized: Cell::new(false),
            top: Cell::new(12.0),
            log: RefCell::new(Vec::new()),
            writes: RefCell::new(Vec::new()),
            dims: RefCell::new(Vec::new()),
            callbacks: RefCell::new(Vec::new()),
            hook: RefCell::new(None),
            fail: Cell::new(None),
            immediate: Cell::new(false),
        }
    }
}
impl Host {
    fn call(&self, name: &'static str) -> Result<(), Failure> {
        self.log.borrow_mut().push(name);
        let hook = self.hook.borrow().clone();
        if let Some(hook) = hook {
            hook(name);
        }
        if self.fail.get() == Some(name) {
            Err(Failure(name))
        } else {
            Ok(())
        }
    }
    fn flush(&self) -> Result<(), ViewportError<Failure>> {
        let callbacks: Vec<_> = self.callbacks.borrow_mut().drain(..).collect();
        for callback in callbacks {
            callback()?;
        }
        Ok(())
    }
}
impl ViewportHost for Host {
    type Error = Failure;
    fn display_row(&self) -> Result<f64, Failure> {
        self.call("row")?;
        Ok(self.row.get())
    }
    fn cell_height(&self) -> Result<f64, Failure> {
        self.call("cell")?;
        Ok(self.cell.get())
    }
    fn canvas_height(&self) -> Result<f64, Failure> {
        self.call("canvas")?;
        Ok(self.canvas.get())
    }
    fn buffer_line_count(&self) -> Result<f64, Failure> {
        self.call("count")?;
        Ok(self.count.get())
    }
    fn synchronized_output(&self) -> Result<bool, Failure> {
        self.call("mode")?;
        Ok(self.synchronized.get())
    }
    fn scroll_top(&self) -> Result<f64, Failure> {
        self.call("top")?;
        Ok(self.top.get())
    }
    fn set_dimensions(&self, height: f64, scroll: f64) -> Result<(), Failure> {
        self.dims.borrow_mut().push((height, scroll));
        self.call("dimensions")
    }
    fn set_scroll_top(&self, top: f64, reuse: Option<bool>) -> Result<(), Failure> {
        self.top.set(top);
        self.writes.borrow_mut().push((top, reuse));
        self.call("position")
    }
    fn add_refresh_callback(&self, callback: SyncCallback<Failure>) -> Result<i32, Failure> {
        self.call("queue")?;
        let id = i32::try_from(self.callbacks.borrow().len()).unwrap();
        if self.immediate.get() {
            callback().map_err(|_| Failure("callback"))?;
        } else {
            self.callbacks.borrow_mut().push(callback);
        }
        Ok(id)
    }
}
fn setup() -> (Rc<Host>, Rc<ViewportController<Host>>) {
    let host = Rc::new(Host::default());
    let viewport = ViewportController::new(host.clone());
    (host, viewport)
}

#[test]
fn queue_coalesces_and_sync_reads_live_metrics_before_real_position_write() {
    let (host, viewport) = setup();
    viewport.queue_sync(None).unwrap();
    viewport.queue_sync(None).unwrap();
    assert_eq!(host.callbacks.borrow().len(), 1);
    assert_eq!(viewport.queued_refresh(), Some(0));
    host.cell.set(12.5);
    host.flush().unwrap();
    assert_eq!(*host.dims.borrow(), [(40.0, 250.0)]);
    assert_eq!(*host.writes.borrow(), [(25.0, None)]);
    assert_eq!(
        *host.log.borrow(),
        [
            "queue",
            "row",
            "mode",
            "canvas",
            "cell",
            "count",
            "dimensions",
            "cell",
            "position"
        ]
    );
    assert_eq!(viewport.queued_refresh(), None);
}
#[test]
fn queued_latest_row_is_updated_before_coalescing_and_activation_clears_it() {
    let (host, viewport) = setup();
    viewport.queue_sync(Some(4.0)).unwrap();
    viewport.queue_sync(Some(9.0)).unwrap();
    host.flush().unwrap();
    assert_eq!(viewport.latest_display_row(), Some(9.0));
    assert!(host.writes.borrow().is_empty());
    host.row.set(5.0);
    viewport.buffer_activated().unwrap();
    host.flush().unwrap();
    assert_eq!(viewport.latest_display_row(), None);
    assert_eq!(*host.writes.borrow(), [(50.0, None)]);
}
#[test]
fn synchronous_refresh_callback_still_assigns_returned_id_afterward() {
    let (host, viewport) = setup();
    host.immediate.set(true);
    viewport.queue_sync(None).unwrap();
    assert_eq!(viewport.queued_refresh(), Some(0));
    let calls = host.log.borrow().len();
    viewport.queue_sync(Some(3.0)).unwrap();
    assert_eq!(host.log.borrow().len(), calls);
}
#[test]
fn synchronized_output_defers_until_render_and_rechecks_source_flag() {
    let (host, viewport) = setup();
    host.synchronized.set(true);
    viewport.sync(None).unwrap();
    assert!(viewport.needs_sync_on_render());
    assert!(host.dims.borrow().is_empty());
    viewport.on_render().unwrap();
    assert!(viewport.needs_sync_on_render());
    host.synchronized.set(false);
    viewport.on_render().unwrap();
    assert!(!viewport.needs_sync_on_render());
    assert_eq!(host.dims.borrow().len(), 1);
}
#[test]
fn dimension_feedback_and_nested_sync_are_suppressed_without_borrow_panics() {
    let (host, viewport) = setup();
    let weak = Rc::downgrade(&viewport);
    *host.hook.borrow_mut() = Some(Rc::new(move |name| {
        if let Some(viewport) = weak.upgrade() {
            if name == "dimensions" {
                viewport.handle_scroll(99.0).unwrap();
            }
            if name == "canvas" {
                viewport.sync(None).unwrap();
            }
        }
    }));
    let requests = Rc::new(Cell::new(0));
    let out = requests.clone();
    viewport
        .on_request_scroll_lines()
        .subscribe(move |_| out.set(out.get() + 1));
    viewport.sync(None).unwrap();
    assert_eq!(requests.get(), 0);
    assert_eq!(
        host.log
            .borrow()
            .iter()
            .filter(|name| **name == "row")
            .count(),
        2
    );
}
#[test]
fn line_absolute_and_touch_scroll_use_source_reuse_property_presence() {
    let (host, viewport) = setup();
    viewport.scroll_lines(2.5).unwrap();
    viewport.scroll_to_line(4.5, false).unwrap();
    viewport.scroll_to_line(6.5, true).unwrap();
    viewport.handle_touch_scroll(7.5).unwrap();
    assert_eq!(
        *host.writes.borrow(),
        [
            (37.0, Some(true)),
            (45.0, Some(true)),
            (65.0, Some(false)),
            (57.5, None)
        ]
    );
    assert_eq!(viewport.latest_display_row(), Some(6.5));
}
#[test]
fn scroll_requests_commit_latest_before_callbacks_and_prevent_nested_handling() {
    let (host, viewport) = setup();
    let weak = Rc::downgrade(&viewport);
    let requests = Rc::new(RefCell::new(Vec::new()));
    let out = requests.clone();
    viewport
        .on_request_scroll_lines()
        .subscribe(move |difference| {
            let viewport = weak.upgrade().unwrap();
            assert_eq!(viewport.latest_display_row(), Some(3.0));
            out.borrow_mut().push(*difference);
            viewport.handle_scroll(100.0).unwrap();
        });
    viewport.handle_scroll(29.0).unwrap();
    assert_eq!(*requests.borrow(), [1.0]);
    assert_eq!(
        host.log
            .borrow()
            .iter()
            .filter(|name| **name == "cell")
            .count(),
        1
    );
}
#[test]
fn javascript_round_retains_negative_zero_ties_and_nonfinite_requests() {
    let (host, viewport) = setup();
    host.row.set(1.0);
    viewport.handle_scroll(-5.0).unwrap();
    assert_eq!(
        viewport.latest_display_row().unwrap().to_bits(),
        (-0.0f64).to_bits()
    );
    viewport.handle_scroll(f64::NAN).unwrap();
    assert!(viewport.latest_display_row().unwrap().is_nan());
    viewport.handle_scroll(f64::INFINITY).unwrap();
    assert_eq!(viewport.latest_display_row(), Some(f64::INFINITY));
}
#[test]
fn failed_dimension_call_preserves_sync_and_suppression_partial_state() {
    let (host, viewport) = setup();
    host.fail.set(Some("dimensions"));
    assert!(matches!(
        viewport.sync(None),
        Err(ViewportError::Host(Failure("dimensions")))
    ));
    host.fail.set(None);
    let before = host.log.borrow().len();
    viewport.sync(None).unwrap();
    viewport.handle_scroll(100.0).unwrap();
    assert_eq!(&host.log.borrow()[before..], ["row"]);
}
#[test]
fn thrown_scroll_listener_keeps_handling_flag_and_actual_error_identity() {
    let (host, viewport) = setup();
    let error = EmitError::callback(Failure("listener"));
    let thrown = error.clone();
    viewport
        .on_request_scroll_lines()
        .try_subscribe(move |_| Err(thrown.clone()));
    assert!(
        matches!(viewport.handle_scroll(30.0), Err(ViewportError::Dispatch(actual)) if actual == error)
    );
    let before = host.log.borrow().len();
    viewport.handle_scroll(100.0).unwrap();
    assert_eq!(host.log.borrow().len(), before);
}
#[test]
fn deferred_flag_clears_before_failed_render_sync() {
    let (host, viewport) = setup();
    host.synchronized.set(true);
    viewport.sync(None).unwrap();
    host.synchronized.set(false);
    host.fail.set(Some("row"));
    assert!(viewport.on_render().is_err());
    assert!(!viewport.needs_sync_on_render());
}
