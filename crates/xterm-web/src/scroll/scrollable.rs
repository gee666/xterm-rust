// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! One host-driven Scrollable for numeric and callback-capable raw values.
mod updates;
use super::{
    animation::SmoothOperation,
    animation_live,
    state::{Position, PositionUpdate, ScrollEvent, ScrollState},
    value::ScrollValue,
};
use crate::links::debounce::FrameScheduler;
use std::{
    cell::RefCell,
    convert::Infallible,
    error::Error,
    fmt,
    rc::{Rc, Weak},
};
use xterm_core::{
    event::{EmitError, Emitter, Event},
    lifecycle::Disposable,
};

#[derive(Debug)]
pub enum ScrollError<E, V = Infallible> {
    Scheduler(E),
    Dispatch(EmitError),
    Value(V),
    /// A coercion removed the operation before a source-required dereference.
    MissingAnimation,
}
impl<E: fmt::Display, V: fmt::Display> fmt::Display for ScrollError<E, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scheduler(error) => error.fmt(f),
            Self::Dispatch(error) => error.fmt(f),
            Self::Value(error) => error.fmt(f),
            Self::MissingAnimation => f.write_str("scroll animation removed during operation"),
        }
    }
}
impl<E: Error + 'static, V: Error + 'static> Error for ScrollError<E, V> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Scheduler(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::Value(error) => Some(error),
            Self::MissingAnimation => None,
        }
    }
}
struct Animation<S: FrameScheduler, V> {
    operation: RefCell<SmoothOperation<V>>,
    pending: RefCell<Option<S::Handle>>,
}
type SharedAnimation<S, V> = Rc<Animation<S, V>>;
struct State<S: FrameScheduler, V> {
    scheduler: RefCell<S>,
    now: Rc<dyn Fn() -> f64>,
    scroll: RefCell<ScrollState<V>>,
    smooth: RefCell<Option<SharedAnimation<S, V>>>,
    duration: RefCell<V>,
    event: Emitter<ScrollEvent<V>>,
}
impl<S: FrameScheduler, V> Drop for State<S, V> {
    fn drop(&mut self) {
        if let Some(animation) = self.smooth.get_mut().take() {
            if let Some(handle) = animation.pending.borrow_mut().take() {
                let scheduler = self.scheduler.get_mut();
                if let Err(error) = scheduler.cancel_frame(handle) {
                    scheduler.cleanup_error(error);
                }
            }
        }
    }
}
pub struct Scrollable<S: FrameScheduler, V = f64> {
    state: Rc<State<S, V>>,
}
pub struct WeakScrollable<S: FrameScheduler, V = f64> {
    state: Weak<State<S, V>>,
}
impl<S: FrameScheduler, V> Clone for WeakScrollable<S, V> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}
impl<S: FrameScheduler, V> WeakScrollable<S, V> {
    pub fn upgrade(&self) -> Option<Scrollable<S, V>> {
        self.state.upgrade().map(|state| Scrollable { state })
    }
}
impl<S: FrameScheduler, V> Clone for Scrollable<S, V> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}
impl<S: FrameScheduler, V: ScrollValue + 'static> Scrollable<S, V> {
    /// The clock shares the frame timeline. Schedulers deliver asynchronously
    /// and must not reenter their own request/cancel methods while borrowed.
    pub fn try_new(
        scheduler: S,
        integer_values: bool,
        duration: V,
        now: impl Fn() -> f64 + 'static,
    ) -> Result<Self, V::Error> {
        Ok(Self {
            state: Rc::new(State {
                scheduler: RefCell::new(scheduler),
                now: Rc::new(now),
                scroll: RefCell::new(ScrollState::try_new(
                    integer_values,
                    V::number(0.0),
                    V::number(0.0),
                    V::number(0.0),
                    V::number(0.0),
                    V::number(0.0),
                    V::number(0.0),
                )?),
                smooth: RefCell::new(None),
                duration: RefCell::new(duration),
                event: Emitter::new(),
            }),
        })
    }
    pub fn downgrade(&self) -> WeakScrollable<S, V> {
        WeakScrollable {
            state: Rc::downgrade(&self.state),
        }
    }
    pub fn on_scroll(&self) -> Event<ScrollEvent<V>> {
        self.state.event.event()
    }
    pub fn set_smooth_scroll_duration(&self, duration: V) {
        *self.state.duration.borrow_mut() = duration;
    }
    pub fn scroll_state(&self) -> ScrollState<V> {
        self.state.scroll.borrow().clone()
    }
    pub fn try_validate_scroll_position(
        &self,
        position: PositionUpdate<V>,
    ) -> Result<Position<V>, V::Error> {
        self.try_validate_scroll_position_resolving(|| Ok(position))
    }
    /// Validation retains the state selected before reading update properties.
    pub fn try_validate_scroll_position_resolving(
        &self,
        resolve: impl FnOnce() -> Result<PositionUpdate<V>, V::Error>,
    ) -> Result<Position<V>, V::Error> {
        let state = self.scroll_state();
        state
            .try_with_position(resolve()?)
            .map(|state| state.position())
    }
    pub fn future_scroll_position(&self) -> Position<V> {
        self.animation().map_or_else(
            || self.current_scroll_position(),
            |animation| {
                animation_live::read(&animation.operation, |operation| operation.to.position())
            },
        )
    }
    pub fn current_scroll_position(&self) -> Position<V> {
        self.scroll_state().position()
    }
    pub fn has_pending_scroll_animation(&self) -> bool {
        self.animation().is_some()
    }
    /// Delivery clears the current operation's handle, as in the source RAF
    /// callback. A tick retains its receiver across clock/coercion reentry.
    pub fn frame(&self) -> Result<(), ScrollError<S::Error, V::Error>> {
        let Some(animation) = self.animation() else {
            return Ok(());
        };
        animation.pending.borrow_mut().take();
        let update = animation_live::tick(&animation.operation, || (self.state.now)())
            .map_err(ScrollError::Value)?;
        let next = self
            .scroll_state()
            .try_with_position(PositionUpdate {
                left: Some(update.position.left),
                top: Some(update.position.top),
            })
            .map_err(ScrollError::Value)?;
        self.set_state(next, true)?;
        if self.animation().is_none() {
            return Ok(());
        }
        if update.done {
            self.stop_animation()
        } else {
            self.request_frame()
        }
    }
    pub fn dispose(&self) -> Result<(), ScrollError<S::Error, V::Error>> {
        self.stop_animation()?;
        self.state.event.dispose();
        Ok(())
    }
    fn animation(&self) -> Option<SharedAnimation<S, V>> {
        self.state.smooth.borrow().clone()
    }
    fn required_animation(&self) -> Result<SharedAnimation<S, V>, ScrollError<S::Error, V::Error>> {
        self.animation().ok_or(ScrollError::MissingAnimation)
    }
    fn set_state(
        &self,
        next: ScrollState<V>,
        smooth: bool,
    ) -> Result<(), ScrollError<S::Error, V::Error>> {
        let previous = self.scroll_state();
        if previous.equals(&next) {
            return Ok(());
        }
        *self.state.scroll.borrow_mut() = next.clone();
        self.state
            .event
            .fire(&next.event(previous, smooth))
            .map_err(ScrollError::Dispatch)
    }
    fn request_frame(&self) -> Result<(), ScrollError<S::Error, V::Error>> {
        // Evaluate the assignment receiver before calling the scheduler.
        let animation = self.required_animation()?;
        let handle = self
            .state
            .scheduler
            .borrow_mut()
            .request_frame()
            .map_err(ScrollError::Scheduler)?;
        *animation.pending.borrow_mut() = Some(handle);
        Ok(())
    }
    fn cancel(&self, animation: &Animation<S, V>) -> Result<(), ScrollError<S::Error, V::Error>> {
        let handle = animation.pending.borrow().clone();
        if let Some(handle) = handle {
            self.state
                .scheduler
                .borrow_mut()
                .cancel_frame(handle)
                .map_err(ScrollError::Scheduler)?;
            animation.pending.borrow_mut().take();
        }
        Ok(())
    }
    fn stop_animation(&self) -> Result<(), ScrollError<S::Error, V::Error>> {
        if let Some(animation) = self.animation() {
            self.cancel(&animation)?;
            self.state.smooth.borrow_mut().take();
        }
        Ok(())
    }
}
impl<S: FrameScheduler> Scrollable<S> {
    pub fn new(
        scheduler: S,
        integer_values: bool,
        duration: f64,
        now: impl Fn() -> f64 + 'static,
    ) -> Self {
        super::value::infallible(Self::try_new(scheduler, integer_values, duration, now))
    }
    pub fn validate_scroll_position(&self, position: PositionUpdate) -> Position {
        super::value::infallible(self.try_validate_scroll_position(position))
    }
}
