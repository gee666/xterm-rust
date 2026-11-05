// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Shared SmoothScrollingOperation for numbers and raw browser values.
use super::{
    state::{Position, ScrollState},
    value::{infallible, ScrollValue},
};

const ADVANCE_MS: f64 = 10.0;
const LONG_DISTANCE_THRESHOLD: f64 = 2.5;
const EDGE_DISTANCE: f64 = 0.75;
const COMPOSITION_CUT: f64 = 0.33;

#[derive(Clone, Copy, Debug)]
pub(super) struct Cubic<V> {
    from: V,
    delta: V,
}
impl<V: ScrollValue> Cubic<V> {
    fn new(from: V, to: &V) -> Result<Self, V::Error> {
        // Source captures this subtraction once, not on each animation tick.
        let delta = to.subtract(&from)?;
        Ok(Self { from, delta })
    }
    fn value(&self, completion: &V) -> Result<V, V::Error> {
        let one = V::number(1.0);
        let ease = one.subtract(&one.subtract(completion)?.power(&V::number(3.0))?)?;
        self.from.add(&self.delta.multiply(&ease)?)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Curve<V> {
    Cubic(Cubic<V>),
    Composed { first: Cubic<V>, second: Cubic<V> },
}
impl<V: ScrollValue> Curve<V> {
    pub(super) fn new(from: V, to: &V, viewport: &V) -> Result<Self, V::Error> {
        let distance = from.subtract(to)?.absolute()?;
        if distance.greater(&V::number(LONG_DISTANCE_THRESHOLD).multiply(viewport)?)? {
            // Addition and subtraction are distinct: raw strings concatenate.
            // Both viewport multiplications occur, preserving coercion order.
            let (first_stop, second_stop) = if from.less(to)? {
                (
                    from.add(&V::number(EDGE_DISTANCE).multiply(viewport)?)?,
                    to.subtract(&V::number(EDGE_DISTANCE).multiply(viewport)?)?,
                )
            } else {
                (
                    from.subtract(&V::number(EDGE_DISTANCE).multiply(viewport)?)?,
                    to.add(&V::number(EDGE_DISTANCE).multiply(viewport)?)?,
                )
            };
            Ok(Self::Composed {
                first: Cubic::new(from, &first_stop)?,
                second: Cubic::new(second_stop, to)?,
            })
        } else {
            Ok(Self::Cubic(Cubic::new(from, to)?))
        }
    }
    pub(super) fn value(&self, completion: &V) -> Result<V, V::Error> {
        match self {
            Self::Cubic(curve) => curve.value(completion),
            Self::Composed { first, second } => {
                let cut = V::number(COMPOSITION_CUT);
                if completion.less(&cut)? {
                    first.value(&completion.divide(&cut)?)
                } else {
                    second.value(
                        &completion
                            .subtract(&cut)?
                            .divide(&V::number(1.0).subtract(&cut)?)?,
                    )
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AnimationUpdate<V = f64> {
    pub position: Position<V>,
    pub done: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct SmoothOperation<V = f64> {
    pub from: ScrollState<V>,
    pub to: ScrollState<V>,
    pub start_time: V,
    pub duration: V,
    pub(super) horizontal: Curve<V>,
    pub(super) vertical: Curve<V>,
}
impl<V: ScrollValue> SmoothOperation<V> {
    pub fn try_new(
        from: ScrollState<V>,
        to: ScrollState<V>,
        start_time: V,
        duration: V,
    ) -> Result<Self, V::Error> {
        let horizontal = Curve::new(from.left.clone(), &to.left, &to.width)?;
        let vertical = Curve::new(from.top.clone(), &to.top, &to.height)?;
        Ok(Self {
            from,
            to,
            start_time,
            duration,
            horizontal,
            vertical,
        })
    }
    /// The host supplies the Date.now millisecond sample. Raw duration addition
    /// happens first and can concatenate or throw, just like source start().
    pub fn try_start(
        from: ScrollState<V>,
        to: ScrollState<V>,
        duration: V,
        now: impl FnOnce() -> f64,
    ) -> Result<Self, V::Error> {
        Self::try_start_with_clock(from, to, duration, || Ok(V::number(now())))
    }
    /// Raw Date.now may return an object or throw. Subtraction performs its
    /// coercion only after the duration addition and clock invocation.
    pub fn try_start_with_clock(
        from: ScrollState<V>,
        to: ScrollState<V>,
        duration: V,
        now: impl FnOnce() -> Result<V, V::Error>,
    ) -> Result<Self, V::Error> {
        let duration = duration.add(&V::number(ADVANCE_MS))?;
        let start_time = now()?.subtract(&V::number(ADVANCE_MS))?;
        Self::try_new(from, to, start_time, duration)
    }
    pub fn try_accept_dimensions(&mut self, state: ScrollState<V>) -> Result<(), V::Error> {
        super::animation_live::accept_dimensions(&std::cell::RefCell::new(self), state)
    }
    pub fn try_tick(&self, now: f64) -> Result<AnimationUpdate<V>, V::Error> {
        self.try_tick_with_clock(|| Ok(V::number(now)))
    }
    /// Samples the fallible clock before reading the operation's start time.
    pub fn try_tick_with_clock(
        &self,
        now: impl FnOnce() -> Result<V, V::Error>,
    ) -> Result<AnimationUpdate<V>, V::Error> {
        super::animation_live::tick(&std::cell::RefCell::new(self), now)
    }
}
impl SmoothOperation {
    pub fn new(from: ScrollState, to: ScrollState, start_time: f64, duration: f64) -> Self {
        infallible(Self::try_new(from, to, start_time, duration))
    }
    pub fn start(from: ScrollState, to: ScrollState, duration: f64, now: f64) -> Self {
        infallible(Self::try_start(from, to, duration, || now))
    }
    pub fn accept_dimensions(&mut self, state: ScrollState) {
        infallible(self.try_accept_dimensions(state));
    }
    pub fn tick(&self, now: f64) -> AnimationUpdate {
        infallible(self.try_tick(now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state(top: f64) -> ScrollState {
        ScrollState::new(false, 100.0, 1000.0, 0.0, 100.0, 1000.0, top)
    }
    #[test]
    fn early_advance_and_exact_final_endpoint() {
        let operation = SmoothOperation::start(state(0.0), state(100.0), 90.0, 1000.0);
        assert_eq!(operation.start_time, 990.0);
        assert_eq!(operation.duration, 100.0);
        assert_eq!(operation.tick(1040.0).position.top, 87.5);
        assert!(!operation.tick(1089.0).done);
        assert_eq!(operation.tick(1090.0).position.top, 100.0);
        assert!(operation.tick(f64::NAN).done);
    }
    #[test]
    fn deltas_are_cached_but_final_endpoint_is_live() {
        let mut operation = SmoothOperation::new(state(0.0), state(100.0), 0.0, 100.0);
        operation.from.top = 30.0;
        operation.to.top = 200.0;
        assert_eq!(operation.tick(50.0).position.top, 87.5);
        assert_eq!(operation.tick(100.0).position.top, 200.0);
    }
    #[test]
    fn long_animation_uses_two_edge_segments() {
        let operation = SmoothOperation::new(state(0.0), state(900.0), 0.0, 100.0);
        assert_eq!(operation.tick(0.0).position.top, 0.0);
        assert_eq!(operation.tick(33.0).position.top, 825.0);
        assert_eq!(operation.tick(100.0).position.top, 900.0);
    }
}
