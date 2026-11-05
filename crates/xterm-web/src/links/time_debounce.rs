// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::debounce::{RenderRange, Rows};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub const DEFAULT_DEBOUNCE_MS: f64 = 1000.0;
pub trait TimerScheduler {
    type Handle: Clone;
    type Error;
    fn now(&self) -> f64;
    /// Timeout handles must be nonzero, as browser timer identifiers are.
    fn request_timeout(&mut self, delay_ms: f64) -> Result<Self::Handle, Self::Error>;
    fn cancel_timeout(&mut self, handle: Self::Handle) -> Result<(), Self::Error>;
    fn cleanup_error(&mut self, error: Self::Error);
    /// Reentrant hosts copy their clock handles and release this borrow before
    /// entering host code. Existing non-reentrant schedulers keep their API.
    fn now_shared(scheduler: &RefCell<Self>) -> Result<f64, Self::Error>
    where
        Self: Sized,
    {
        Ok(scheduler.borrow().now())
    }
    fn request_timeout_shared(
        scheduler: &RefCell<Self>,
        delay_ms: f64,
    ) -> Result<Self::Handle, Self::Error>
    where
        Self: Sized,
    {
        scheduler.borrow_mut().request_timeout(delay_ms)
    }
    /// Resolve the pending ID after the host's cancellation method lookup.
    /// A getter can reenter and replace or clear that ID before arguments run.
    fn cancel_timeout_resolved(
        scheduler: &RefCell<Self>,
        resolve: impl FnOnce() -> Option<Self::Handle>,
    ) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        match resolve() {
            Some(handle) => scheduler.borrow_mut().cancel_timeout(handle),
            None => Ok(()),
        }
    }
}

type Render<S> =
    Rc<dyn Fn(&TimeBasedDebouncer<S>, RenderRange) -> Result<(), <S as TimerScheduler>::Error>>;
struct State<S: TimerScheduler> {
    scheduler: RefCell<S>,
    rows: RefCell<Rows>,
    last_refresh: Cell<f64>,
    additional: Cell<bool>,
    pending: RefCell<Option<S::Handle>>,
    render: Render<S>,
    threshold: f64,
}
impl<S: TimerScheduler> Drop for State<S> {
    fn drop(&mut self) {
        if let Some(handle) = self.pending.get_mut().take() {
            let scheduler = self.scheduler.get_mut();
            if let Err(error) = scheduler.cancel_timeout(handle) {
                scheduler.cleanup_error(error);
            }
        }
    }
}

/// Source-compatible trailing throttle. Time comes from the host's performance
/// clock. No sleeps or background threads are part of the portable algorithm.
pub struct TimeBasedDebouncer<S: TimerScheduler> {
    state: Rc<State<S>>,
}
impl<S: TimerScheduler> Clone for TimeBasedDebouncer<S> {
    fn clone(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
        }
    }
}
impl<S: TimerScheduler> TimeBasedDebouncer<S> {
    pub fn new(
        scheduler: S,
        threshold_ms: f64,
        render: impl Fn(&Self, RenderRange) + 'static,
    ) -> Self {
        Self::try_new(scheduler, threshold_ms, move |debouncer, range| {
            render(debouncer, range);
            Ok(())
        })
    }
    pub fn try_new(
        scheduler: S,
        threshold_ms: f64,
        render: impl Fn(&Self, RenderRange) -> Result<(), S::Error> + 'static,
    ) -> Self {
        Self {
            state: Rc::new(State {
                scheduler: RefCell::new(scheduler),
                rows: RefCell::new(Rows::default()),
                last_refresh: Cell::new(0.0),
                additional: Cell::new(false),
                pending: RefCell::new(None),
                render: Rc::new(render),
                threshold: threshold_ms,
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
        let now = S::now_shared(&self.state.scheduler)?;
        let elapsed = now - self.state.last_refresh.get();
        if elapsed >= self.state.threshold {
            let has_pending = self.state.pending.borrow().is_some();
            if has_pending {
                self.cancel_pending()?;
                self.state.pending.borrow_mut().take();
                self.state.additional.set(false);
            }
            self.state.last_refresh.set(now);
            self.render()?;
        } else if !self.state.additional.get() {
            self.state.additional.set(true);
            let handle =
                S::request_timeout_shared(&self.state.scheduler, self.state.threshold - elapsed)?;
            *self.state.pending.borrow_mut() = Some(handle);
        }
        Ok(())
    }
    /// Consume one fired host timeout. Clearing trailing flags AFTER rendering is
    /// intentional: callback reentrancy sees the source's still-pending state.
    pub fn weak_dispatcher(&self) -> impl Fn() -> Result<(), S::Error> + 'static
    where
        S: 'static,
    {
        let state = Rc::downgrade(&self.state);
        move || {
            if let Some(state) = state.upgrade() {
                Self { state }.on_timeout()?;
            }
            Ok(())
        }
    }
    pub fn on_timeout(&self) -> Result<(), S::Error> {
        self.state
            .last_refresh
            .set(S::now_shared(&self.state.scheduler)?);
        self.render()?;
        self.state.additional.set(false);
        self.state.pending.borrow_mut().take();
        Ok(())
    }
    pub fn dispose(&self) -> Result<(), S::Error> {
        let has_pending = self.state.pending.borrow().is_some();
        if has_pending {
            self.cancel_pending()?;
            self.state.pending.borrow_mut().take();
        }
        self.state.additional.set(false);
        Ok(())
    }
    pub fn has_pending_timeout(&self) -> bool {
        self.state.pending.borrow().is_some()
    }
    fn cancel_pending(&self) -> Result<(), S::Error> {
        S::cancel_timeout_resolved(&self.state.scheduler, || {
            self.state.pending.borrow().clone()
        })
    }
    fn render(&self) -> Result<(), S::Error> {
        let range = self.state.rows.borrow_mut().take();
        if let Some(range) = range {
            (self.state.render)(self, range)?;
        }
        Ok(())
    }
}
