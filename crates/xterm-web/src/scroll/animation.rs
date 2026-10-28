// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! SmoothScrollingOperation from browser/scrollable/scrollable.ts.
use super::state::{Position, PositionUpdate, ScrollState};

const ADVANCE_MS: f64 = 10.0;
const LONG_DISTANCE_THRESHOLD: f64 = 2.5;
const EDGE_DISTANCE: f64 = 0.75;
const COMPOSITION_CUT: f64 = 0.33;

#[derive(Clone, Copy, Debug)]
enum Curve {
    Cubic {
        from: f64,
        to: f64,
    },
    Composed {
        from: f64,
        first_stop: f64,
        second_stop: f64,
        to: f64,
    },
}
impl Curve {
    fn new(from: f64, to: f64, viewport: f64) -> Self {
        if (from - to).abs() > LONG_DISTANCE_THRESHOLD * viewport {
            let direction = if from < to { 1.0 } else { -1.0 };
            Self::Composed {
                from,
                first_stop: from + direction * EDGE_DISTANCE * viewport,
                second_stop: to - direction * EDGE_DISTANCE * viewport,
                to,
            }
        } else {
            Self::Cubic { from, to }
        }
    }
    fn value(self, completion: f64) -> f64 {
        match self {
            Self::Cubic { from, to } => cubic(from, to, completion),
            Self::Composed {
                from,
                first_stop,
                second_stop,
                to,
            } => {
                if completion < COMPOSITION_CUT {
                    cubic(from, first_stop, completion / COMPOSITION_CUT)
                } else {
                    cubic(
                        second_stop,
                        to,
                        (completion - COMPOSITION_CUT) / (1.0 - COMPOSITION_CUT),
                    )
                }
            }
        }
    }
}
fn cubic(from: f64, to: f64, completion: f64) -> f64 {
    from + (to - from) * (1.0 - (1.0 - completion).powi(3))
}

#[derive(Clone, Copy, Debug)]
pub struct AnimationUpdate {
    pub position: Position,
    pub done: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct SmoothOperation {
    pub from: ScrollState,
    pub to: ScrollState,
    pub start_time: f64,
    pub duration: f64,
    horizontal: Curve,
    vertical: Curve,
}
impl SmoothOperation {
    pub fn new(from: ScrollState, to: ScrollState, start_time: f64, duration: f64) -> Self {
        Self {
            from,
            to,
            start_time,
            duration,
            horizontal: Curve::new(from.left, to.left, to.width),
            vertical: Curve::new(from.top, to.top, to.height),
        }
    }
    pub fn start(from: ScrollState, to: ScrollState, duration: f64, now: f64) -> Self {
        Self::new(from, to, now - ADVANCE_MS, duration + ADVANCE_MS)
    }
    pub fn accept_dimensions(&mut self, state: ScrollState) {
        self.to = state.with_position(PositionUpdate {
            left: Some(self.to.left),
            top: Some(self.to.top),
        });
        self.horizontal = Curve::new(self.from.left, self.to.left, self.to.width);
        self.vertical = Curve::new(self.from.top, self.to.top, self.to.height);
    }
    pub fn tick(&self, now: f64) -> AnimationUpdate {
        let completion = (now - self.start_time) / self.duration;
        if completion < 1.0 {
            AnimationUpdate {
                position: Position {
                    left: self.horizontal.value(completion),
                    top: self.vertical.value(completion),
                },
                done: false,
            }
        } else {
            AnimationUpdate {
                position: self.to.position(),
                done: true,
            }
        }
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
    fn long_animation_uses_two_edge_segments() {
        let operation = SmoothOperation::new(state(0.0), state(900.0), 0.0, 100.0);
        assert_eq!(operation.tick(0.0).position.top, 0.0);
        assert_eq!(operation.tick(33.0).position.top, 825.0);
        assert_eq!(operation.tick(100.0).position.top, 900.0);
    }
}
