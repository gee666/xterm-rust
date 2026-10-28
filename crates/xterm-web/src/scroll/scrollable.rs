// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Host-driven Scrollable. FrameScheduler requests asynchronous frames; the
//! adapter invokes frame() at delivery. Clone handles share state and events.
use super::{
    animation::SmoothOperation,
    state::{DimensionUpdate, Position, PositionUpdate, ScrollEvent, ScrollState},
};
use crate::links::debounce::FrameScheduler;
use std::{
    cell::{Cell, RefCell},
    error::Error,
    fmt,
    rc::{Rc, Weak},
};
use xterm_core::{
    event::{EmitError, Emitter, Event},
    lifecycle::Disposable,
};

#[derive(Debug)]
pub enum ScrollError<E> {
    Scheduler(E),
    Dispatch(EmitError),
}
impl<E: fmt::Display> fmt::Display for ScrollError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scheduler(error) => error.fmt(f),
            Self::Dispatch(error) => error.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for ScrollError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(match self {
            Self::Scheduler(error) => error,
            Self::Dispatch(error) => error,
        })
    }
}
struct State<S: FrameScheduler> {
    scheduler: RefCell<S>,
    now: Rc<dyn Fn() -> f64>,
    scroll: Cell<ScrollState>,
    smooth: Cell<Option<SmoothOperation>>,
    duration: Cell<f64>,
    pending: RefCell<Option<S::Handle>>,
    event: Emitter<ScrollEvent>,
}
impl<S: FrameScheduler> Drop for State<S> {
    fn drop(&mut self) {
        if let Some(handle) = self.pending.get_mut().take() {
            let scheduler = self.scheduler.get_mut();
            if let Err(error) = scheduler.cancel_frame(handle) {
                scheduler.cleanup_error(error);
            }
        }
    }
}
pub struct Scrollable<S: FrameScheduler> {
    state: Rc<State<S>>,
}
pub struct WeakScrollable<S: FrameScheduler> {
    state: Weak<State<S>>,
}
impl<S: FrameScheduler> Clone for WeakScrollable<S> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}
impl<S: FrameScheduler> WeakScrollable<S> {
    pub fn upgrade(&self) -> Option<Scrollable<S>> {
        self.state.upgrade().map(|state| Scrollable { state })
    }
}
impl<S: FrameScheduler> Clone for Scrollable<S> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}
impl<S: FrameScheduler> Scrollable<S> {
    /// The clock must use the same millisecond timeline as future frame calls.
    /// Scheduler callbacks must be asynchronous and must not hold a scheduler
    /// borrow while dispatching frame(), matching requestAnimationFrame.
    pub fn new(
        scheduler: S,
        integer_values: bool,
        duration: f64,
        now: impl Fn() -> f64 + 'static,
    ) -> Self {
        Self {
            state: Rc::new(State {
                scheduler: RefCell::new(scheduler),
                now: Rc::new(now),
                scroll: Cell::new(ScrollState::new(
                    integer_values,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                )),
                smooth: Cell::new(None),
                duration: Cell::new(duration),
                pending: RefCell::new(None),
                event: Emitter::new(),
            }),
        }
    }
    pub fn downgrade(&self) -> WeakScrollable<S> {
        WeakScrollable {
            state: Rc::downgrade(&self.state),
        }
    }
    pub fn on_scroll(&self) -> Event<ScrollEvent> {
        self.state.event.event()
    }
    pub fn set_smooth_scroll_duration(&self, duration: f64) {
        self.state.duration.set(duration);
    }
    pub fn scroll_state(&self) -> ScrollState {
        self.state.scroll.get()
    }
    pub fn validate_scroll_position(&self, position: PositionUpdate) -> Position {
        self.state.scroll.get().with_position(position).position()
    }
    pub fn future_scroll_position(&self) -> Position {
        self.state.smooth.get().map_or_else(
            || self.state.scroll.get().position(),
            |operation| operation.to.position(),
        )
    }
    pub fn current_scroll_position(&self) -> Position {
        self.state.scroll.get().position()
    }
    pub fn has_pending_scroll_animation(&self) -> bool {
        self.state.smooth.get().is_some()
    }
    pub fn set_scroll_dimensions(
        &self,
        update: DimensionUpdate,
        raw_positions: bool,
    ) -> Result<(), ScrollError<S::Error>> {
        let next = self
            .state
            .scroll
            .get()
            .with_dimensions(update, raw_positions);
        self.set_state(next, self.state.smooth.get().is_some())?;
        // Event callbacks may replace the operation; source updates whichever
        // operation is current after those callbacks return.
        if let Some(mut operation) = self.state.smooth.get() {
            operation.accept_dimensions(self.state.scroll.get());
            self.state.smooth.set(Some(operation));
        }
        Ok(())
    }
    pub fn set_scroll_position_now(
        &self,
        update: PositionUpdate,
    ) -> Result<(), ScrollError<S::Error>> {
        let next = self.state.scroll.get().with_position(update);
        self.stop_animation()?;
        self.set_state(next, false)
    }
    pub fn set_scroll_position_smooth(
        &self,
        mut update: PositionUpdate,
        reuse: bool,
    ) -> Result<(), ScrollError<S::Error>> {
        if self.state.duration.get() == 0.0 {
            return self.set_scroll_position_now(update);
        }
        let current = self.state.scroll.get();
        let operation = if let Some(previous) = self.state.smooth.get() {
            update.left = Some(update.left.unwrap_or(previous.to.left));
            update.top = Some(update.top.unwrap_or(previous.to.top));
            let target = current.with_position(update);
            if previous.to.left == target.left && previous.to.top == target.top {
                return Ok(());
            }
            let next = if reuse {
                SmoothOperation::new(
                    previous.from,
                    target,
                    previous.start_time,
                    previous.duration,
                )
            } else {
                SmoothOperation::start(
                    current,
                    target,
                    self.state.duration.get(),
                    (self.state.now)(),
                )
            };
            self.cancel_pending()?;
            next
        } else {
            SmoothOperation::start(
                current,
                current.with_position(update),
                self.state.duration.get(),
                (self.state.now)(),
            )
        };
        self.state.smooth.set(Some(operation));
        self.request_frame()
    }
    /// Called once by the host for each scheduled animation frame. The source
    /// samples Date.now inside tick, rather than trusting the RAF timestamp.
    pub fn frame(&self) -> Result<(), ScrollError<S::Error>> {
        self.state.pending.borrow_mut().take();
        let Some(operation) = self.state.smooth.get() else {
            return Ok(());
        };
        let update = operation.tick((self.state.now)());
        let next = self.state.scroll.get().with_position(PositionUpdate {
            left: Some(update.position.left),
            top: Some(update.position.top),
        });
        self.set_state(next, true)?;
        if self.state.smooth.get().is_none() {
            return Ok(());
        }
        if update.done {
            self.stop_animation()
        } else {
            self.request_frame()
        }
    }
    pub fn dispose(&self) -> Result<(), ScrollError<S::Error>> {
        self.stop_animation()?;
        self.state.event.dispose();
        Ok(())
    }
    fn set_state(&self, next: ScrollState, smooth: bool) -> Result<(), ScrollError<S::Error>> {
        let previous = self.state.scroll.get();
        if previous.equals(&next) {
            return Ok(());
        }
        self.state.scroll.set(next);
        self.state
            .event
            .fire(&next.event(previous, smooth))
            .map_err(ScrollError::Dispatch)
    }
    fn request_frame(&self) -> Result<(), ScrollError<S::Error>> {
        let handle = self
            .state
            .scheduler
            .borrow_mut()
            .request_frame()
            .map_err(ScrollError::Scheduler)?;
        *self.state.pending.borrow_mut() = Some(handle);
        Ok(())
    }
    fn cancel_pending(&self) -> Result<(), ScrollError<S::Error>> {
        let handle = self.state.pending.borrow().clone();
        if let Some(handle) = handle {
            self.state
                .scheduler
                .borrow_mut()
                .cancel_frame(handle)
                .map_err(ScrollError::Scheduler)?;
            self.state.pending.borrow_mut().take();
        }
        Ok(())
    }
    fn stop_animation(&self) -> Result<(), ScrollError<S::Error>> {
        if self.state.smooth.get().is_some() {
            self.cancel_pending()?;
            self.state.smooth.set(None);
        }
        Ok(())
    }
}
