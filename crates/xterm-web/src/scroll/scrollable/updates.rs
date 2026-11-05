//! Update-object evaluation phases, shared by typed and raw browser callers.
use super::{Animation, ScrollError, Scrollable};
use crate::{
    links::debounce::FrameScheduler,
    scroll::{
        animation::SmoothOperation,
        animation_live,
        state::{DimensionUpdate, Position, PositionUpdate, ScrollState},
        value::ScrollValue,
    },
};
use std::{cell::RefCell, rc::Rc};

impl<S: FrameScheduler, V: ScrollValue + 'static> Scrollable<S, V> {
    pub fn set_scroll_dimensions(
        &self,
        update: DimensionUpdate<V>,
        raw_positions: bool,
    ) -> Result<(), ScrollError<S::Error, V::Error>> {
        self.set_scroll_dimensions_resolving(|| Ok(update), raw_positions)
    }
    /// Capture the receiver state before evaluating an update object's getters.
    pub fn set_scroll_dimensions_resolving(
        &self,
        resolve: impl FnOnce() -> Result<DimensionUpdate<V>, V::Error>,
        raw_positions: bool,
    ) -> Result<(), ScrollError<S::Error, V::Error>> {
        let state = self.scroll_state();
        let next = state
            .try_with_dimensions(resolve().map_err(ScrollError::Value)?, raw_positions)
            .map_err(ScrollError::Value)?;
        self.set_state(next, self.has_pending_scroll_animation())?;
        // Select after events, but retain that selected operation's identity
        // throughout coercion, even if a callback cancels/replaces it.
        if let Some(animation) = self.animation() {
            animation_live::accept_dimensions(&animation.operation, self.scroll_state())
                .map_err(ScrollError::Value)?;
        }
        Ok(())
    }
    pub fn set_scroll_position_now(
        &self,
        update: PositionUpdate<V>,
    ) -> Result<(), ScrollError<S::Error, V::Error>> {
        self.set_scroll_position_now_resolving(|| Ok(update))
    }
    /// Getters may reenter, but this operation still updates its selected state.
    pub fn set_scroll_position_now_resolving(
        &self,
        resolve: impl FnOnce() -> Result<PositionUpdate<V>, V::Error>,
    ) -> Result<(), ScrollError<S::Error, V::Error>> {
        let state = self.scroll_state();
        let next = state
            .try_with_position(resolve().map_err(ScrollError::Value)?)
            .map_err(ScrollError::Value)?;
        self.stop_animation()?;
        self.set_state(next, false)
    }
    pub fn set_scroll_position_smooth(
        &self,
        update: PositionUpdate<V>,
        reuse: bool,
    ) -> Result<(), ScrollError<S::Error, V::Error>> {
        self.set_scroll_position_smooth_resolving(|| Ok(update.left), || Ok(update.top), reuse)
    }
    /// Resolve axes separately. A missing axis reads the operation that is
    /// current after that getter, not the one selected before either getter.
    pub fn set_scroll_position_smooth_resolving(
        &self,
        left: impl FnOnce() -> Result<Option<V>, V::Error>,
        top: impl FnOnce() -> Result<Option<V>, V::Error>,
        reuse: bool,
    ) -> Result<(), ScrollError<S::Error, V::Error>> {
        if self.state.duration.borrow().strict_eq(&V::number(0.0)) {
            return self.set_scroll_position_now_resolving(|| {
                Ok(PositionUpdate {
                    left: left()?,
                    top: top()?,
                })
            });
        }
        let replace = self.has_pending_scroll_animation();
        let target = if replace {
            let left = self.resolve_smooth_axis(left, |position| position.left)?;
            let top = self.resolve_smooth_axis(top, |position| position.top)?;
            // The expanded plain object is read by withScrollPosition after
            // both getters. Its undefined values use the selected state's raw
            // requests, even when a getter's second read produced undefined.
            self.scroll_state()
                .try_with_position(PositionUpdate { left, top })
        } else {
            let state = self.scroll_state();
            let update = PositionUpdate {
                left: left().map_err(ScrollError::Value)?,
                top: top().map_err(ScrollError::Value)?,
            };
            state.try_with_position(update)
        }
        .map_err(ScrollError::Value)?;
        let operation = if replace {
            let previous = self.required_animation()?;
            let previous = animation_live::read(&previous.operation, Clone::clone);
            if previous.to.left.strict_eq(&target.left) && previous.to.top.strict_eq(&target.top) {
                return Ok(());
            }
            let next = if reuse {
                SmoothOperation::try_new(
                    previous.from,
                    target,
                    previous.start_time,
                    previous.duration,
                )
            } else {
                self.start(target)
            }
            .map_err(ScrollError::Value)?;
            // Construction can change which operation is current. Source
            // disposes that current operation, not the pre-coercion snapshot.
            let current = self.required_animation()?;
            self.cancel(&current)?;
            next
        } else {
            self.start(target).map_err(ScrollError::Value)?
        };
        *self.state.smooth.borrow_mut() = Some(Rc::new(Animation {
            operation: RefCell::new(operation),
            pending: RefCell::new(None),
        }));
        self.request_frame()
    }
    fn start(&self, target: ScrollState<V>) -> Result<SmoothOperation<V>, V::Error> {
        let from = self.scroll_state();
        let duration = self.state.duration.borrow().clone();
        SmoothOperation::try_start_with_clock(from, target, duration, || (self.state.now)())
    }
    fn resolve_smooth_axis(
        &self,
        read: impl FnOnce() -> Result<Option<V>, V::Error>,
        select: impl FnOnce(Position<V>) -> V,
    ) -> Result<Option<V>, ScrollError<S::Error, V::Error>> {
        let value = match read().map_err(ScrollError::Value)? {
            Some(value) => value,
            None => {
                let animation = self.required_animation()?;
                animation_live::read(&animation.operation, |operation| {
                    select(operation.to.position())
                })
            }
        };
        Ok((!value.is_undefined()).then_some(value))
    }
}
