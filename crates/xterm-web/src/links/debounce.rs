// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Render coalescing with host-scheduled dispatch. Hosts must dispatch asynchronously.
use std::{cell::RefCell, rc::Rc};

pub trait FrameScheduler {
    type Handle: Clone;
    type Error;
    fn request_frame(&mut self) -> Result<Self::Handle, Self::Error>;
    fn cancel_frame(&mut self, handle: Self::Handle) -> Result<(), Self::Error>;
    /// Drop cannot return errors. The host must report failed cleanup, not ignore it.
    fn cleanup_error(&mut self, error: Self::Error);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderRange {
    pub start: f64,
    pub end: f64,
}

#[derive(Default)]
pub(super) struct Rows {
    start: Option<f64>,
    end: Option<f64>,
    count: Option<f64>,
}
impl Rows {
    pub(super) fn add(&mut self, start: Option<f64>, end: Option<f64>, count: f64) {
        self.count = Some(count);
        let start = start.unwrap_or(0.0);
        let end = end.unwrap_or(count - 1.0);
        self.start = Some(self.start.map_or(start, |old| js_min(old, start)));
        self.end = Some(self.end.map_or(end, |old| js_max(old, end)));
    }
    pub(super) fn take(&mut self) -> Option<RenderRange> {
        Some(RenderRange {
            start: js_max(self.start.take()?, 0.0),
            end: js_min(self.end.take()?, self.count? - 1.0),
        })
    }
}
pub(super) fn js_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_negative() || b.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else {
        a.min(b)
    }
}
pub(super) fn js_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_negative() && b.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else {
        a.max(b)
    }
}

type Render<S> =
    Rc<dyn Fn(&RenderDebouncer<S>, RenderRange) -> Result<(), <S as FrameScheduler>::Error>>;
type Refresh<S> = Rc<dyn Fn(&RenderDebouncer<S>, f64) -> Result<(), <S as FrameScheduler>::Error>>;
struct FrameState<S: FrameScheduler> {
    scheduler: RefCell<S>,
    rows: RefCell<Rows>,
    pending: RefCell<Option<S::Handle>>,
    callbacks: RefCell<Vec<Refresh<S>>>,
    render: Render<S>,
}
impl<S: FrameScheduler> Drop for FrameState<S> {
    fn drop(&mut self) {
        if let Some(handle) = self.pending.get_mut().take() {
            let scheduler = self.scheduler.get_mut();
            if let Err(error) = scheduler.cancel_frame(handle) {
                scheduler.cleanup_error(error);
            }
        }
    }
}

/// Clones share one debouncer. Callbacks receive this handle, avoiding captures
/// that form reference cycles. Last-handle drop cancels the outstanding frame.
pub struct RenderDebouncer<S: FrameScheduler> {
    state: Rc<FrameState<S>>,
}
impl<S: FrameScheduler> Clone for RenderDebouncer<S> {
    fn clone(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
        }
    }
}
impl<S: FrameScheduler> RenderDebouncer<S> {
    pub fn new(scheduler: S, render: impl Fn(&Self, RenderRange) + 'static) -> Self {
        Self::try_new(scheduler, move |debouncer, range| {
            render(debouncer, range);
            Ok(())
        })
    }
    pub fn try_new(
        scheduler: S,
        render: impl Fn(&Self, RenderRange) -> Result<(), S::Error> + 'static,
    ) -> Self {
        Self {
            state: Rc::new(FrameState {
                scheduler: RefCell::new(scheduler),
                rows: RefCell::new(Rows::default()),
                pending: RefCell::new(None),
                callbacks: RefCell::new(Vec::new()),
                render: Rc::new(render),
            }),
        }
    }
    pub fn refresh(
        &self,
        start: Option<f64>,
        end: Option<f64>,
        count: f64,
    ) -> Result<(), S::Error> {
        self.state.rows.borrow_mut().add(start, end, count);
        self.schedule().map(|_| ())
    }
    pub fn add_refresh_callback(
        &self,
        callback: impl Fn(&Self, f64) + 'static,
    ) -> Result<S::Handle, S::Error> {
        self.add_try_refresh_callback(move |debouncer, time| {
            callback(debouncer, time);
            Ok(())
        })
    }
    pub fn add_try_refresh_callback(
        &self,
        callback: impl Fn(&Self, f64) -> Result<(), S::Error> + 'static,
    ) -> Result<S::Handle, S::Error> {
        self.state.callbacks.borrow_mut().push(Rc::new(callback));
        self.schedule()
    }
    pub fn dispose(&self) -> Result<(), S::Error> {
        // Source retains row bounds and callbacks after cancellation.
        let pending = self.state.pending.borrow().clone();
        if let Some(handle) = pending {
            self.state.scheduler.borrow_mut().cancel_frame(handle)?;
            self.state.pending.borrow_mut().take();
        }
        Ok(())
    }
    pub fn has_pending_frame(&self) -> bool {
        self.state.pending.borrow().is_some()
    }
    /// An event-loop callback that does not keep this debouncer alive.
    pub fn weak_dispatcher(&self) -> impl Fn() -> Result<(), S::Error> + 'static
    where
        S: 'static,
    {
        let state = Rc::downgrade(&self.state);
        move || {
            if let Some(state) = state.upgrade() {
                Self { state }.on_frame()?;
            }
            Ok(())
        }
    }
    /// Called once by the host when its requested animation frame fires.
    pub fn on_frame(&self) -> Result<(), S::Error> {
        self.state.pending.borrow_mut().take();
        let range = self.state.rows.borrow_mut().take();
        if let Some(range) = range {
            (self.state.render)(self, range)?;
        }
        // JS array iteration sees callbacks appended while iteration is active.
        let mut index = 0;
        loop {
            let callback = self.state.callbacks.borrow().get(index).cloned();
            let Some(callback) = callback else { break };
            callback(self, 0.0)?;
            index += 1;
        }
        let callbacks = std::mem::take(&mut *self.state.callbacks.borrow_mut());
        drop(callbacks);
        Ok(())
    }
    fn schedule(&self) -> Result<S::Handle, S::Error> {
        if let Some(handle) = self.state.pending.borrow().as_ref() {
            return Ok(handle.clone());
        }
        let handle = self.state.scheduler.borrow_mut().request_frame()?;
        *self.state.pending.borrow_mut() = Some(handle.clone());
        Ok(handle)
    }
}
