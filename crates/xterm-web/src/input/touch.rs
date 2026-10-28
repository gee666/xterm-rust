// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License. Port of browser/scrollable/touch.ts policy.

//! Gesture arithmetic uses Date.now-compatible numbers. In particular, equal
//! timestamps and nonfinite values retain JavaScript division/comparison rules.
//! The DOM adapter dispatches movement before committing this rolling history.

#[cfg(target_arch = "wasm32")]
pub use super::touch_dom::GestureContext;

const HOLD_DELAY_MS: f64 = 700.0;
const STATIONARY_DISTANCE: f64 = 30.0;
const CLEAR_TAP_COUNT_MS: f64 = 400.0;
const SCROLL_FRICTION: f64 = -0.005;
const ROLLING_SAMPLES: usize = 4;

#[derive(Clone, Copy, Debug)]
struct Sample {
    time: f64,
    x: f64,
    y: f64,
}

/// One touch record. Keep it alive across DOM callbacks even when a reentrant
/// start replaces the active map entry with a different record for the same ID.
#[derive(Clone, Debug)]
pub struct TouchData<T> {
    pub initial_target: T,
    initial: Sample,
    rolling: [Sample; ROLLING_SAMPLES],
    length: usize,
}

impl<T> TouchData<T> {
    pub fn new(target: T, now: f64, page_x: f64, page_y: f64) -> Self {
        let initial = Sample {
            time: now,
            x: page_x,
            y: page_y,
        };
        Self {
            initial_target: target,
            initial,
            rolling: [initial; ROLLING_SAMPLES],
            length: 1,
        }
    }

    pub fn last(&self) -> (f64, f64) {
        let sample = self.rolling[self.length - 1];
        (sample.x, sample.y)
    }

    /// Preparing a movement must not change the last sample before dispatch.
    pub fn translation(&self, page_x: f64, page_y: f64) -> (f64, f64) {
        let (x, y) = self.last();
        (page_x - x, page_y - y)
    }

    pub fn commit_move(&mut self, now: f64, page_x: f64, page_y: f64) {
        if self.length == ROLLING_SAMPLES {
            self.rolling.rotate_left(1);
        } else {
            self.length += 1;
        }
        self.rolling[self.length - 1] = Sample {
            time: now,
            x: page_x,
            y: page_y,
        };
    }

    /// The two clocks are separate source reads. The DOM layer schedules inertia
    /// only if the active touch count, captured before its end loop, equals one.
    pub fn end(&self, hold_now: f64, inertia_start: f64) -> EndGesture {
        let (x, y) = self.last();
        let hold = hold_now - self.initial.time;
        let stationary = (self.initial.x - x).abs() < STATIONARY_DISTANCE
            && (self.initial.y - y).abs() < STATIONARY_DISTANCE;
        if stationary && hold < HOLD_DELAY_MS {
            return EndGesture::Tap;
        }
        if stationary && hold >= HOLD_DELAY_MS {
            return EndGesture::ContextMenu;
        }
        // Every record starts with one sample and only replaces/appends samples.
        // Thus these indices remain valid regardless of untrusted coordinates.
        let first = self.rolling[0];
        let last = self.rolling[self.length - 1];
        let delta_time = last.time - first.time;
        let delta_x = last.x - first.x;
        let delta_y = last.y - first.y;
        EndGesture::Inertia(Inertia {
            time: inertia_start,
            velocity_x: delta_x.abs() / delta_time,
            direction_x: if delta_x > 0.0 { 1.0 } else { -1.0 },
            velocity_y: delta_y.abs() / delta_time,
            direction_y: if delta_y > 0.0 { 1.0 } else { -1.0 },
        })
    }
}

#[derive(Clone, Debug)]
pub enum EndGesture {
    Tap,
    ContextMenu,
    Inertia(Inertia),
}

/// State for source-compatible requestAnimationFrame displacement calculations.
/// The host owns scheduling/cancellation and the insertion-order target snapshot.
#[derive(Clone, Debug)]
pub struct Inertia {
    time: f64,
    velocity_x: f64,
    direction_x: f64,
    velocity_y: f64,
    direction_y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InertiaFrame {
    pub translation_x: f64,
    pub translation_y: f64,
    pub continues: bool,
}

impl Inertia {
    /// Emit even the final zero frame. Continuing a stopped animation is a host
    /// error, but the arithmetic stays defined and matches repeated source steps.
    pub fn step(&mut self, now: f64) -> InertiaFrame {
        let elapsed = now - self.time;
        self.time = now;
        self.velocity_x += SCROLL_FRICTION * elapsed;
        self.velocity_y += SCROLL_FRICTION * elapsed;
        let moving_x = self.velocity_x > 0.0;
        let moving_y = self.velocity_y > 0.0;
        InertiaFrame {
            translation_x: if moving_x {
                self.direction_x * self.velocity_x * elapsed
            } else {
                0.0
            },
            translation_y: if moving_y {
                self.direction_y * self.velocity_y * elapsed
            } else {
                0.0
            },
            continues: moving_x || moving_y,
        }
    }
}

/// The source resets this timestamp for ordinary CHANGE/CONTEXT_MENU dispatch
/// before ignored-target filtering, but not for inertial CHANGE or START/END.
#[derive(Clone, Copy, Debug, Default)]
pub struct TapCount {
    last: f64,
}

impl TapCount {
    pub fn tap(&mut self, now: f64) -> u8 {
        let count = if now - self.last > CLEAR_TAP_COUNT_MS {
            1
        } else {
            2
        };
        self.last = now;
        count
    }

    pub fn reset(&mut self) {
        self.last = 0.0;
    }
}
