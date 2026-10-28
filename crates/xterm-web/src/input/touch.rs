// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License. Port of browser/scrollable/touch.ts policy.

//! Gesture arithmetic uses Date.now-compatible numbers. Host getters can reenter
//! between history pushes, so X, Y and timestamps must retain independent arrays.

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

/// One captured touch record, independent of replacement in the active-ID map.
/// Normal commits retain four samples. Reentrant coordinate getters can expose
/// unequal/empty arrays or exceed four entries because source shifts only once.
#[derive(Clone, Debug)]
pub struct TouchData<T> {
    pub initial_target: T,
    initial: Sample,
    times: Vec<f64>,
    page_x: Vec<f64>,
    page_y: Vec<f64>,
}

impl<T> TouchData<T> {
    pub fn new(target: T, now: f64, page_x: f64, page_y: f64) -> Self {
        Self::from_snapshots(target, now, (page_x, page_y), (page_x, page_y))
    }

    /// Initial and first rolling positions are separate source getter reads.
    pub fn from_snapshots(target: T, now: f64, initial: (f64, f64), rolling: (f64, f64)) -> Self {
        Self {
            initial_target: target,
            initial: Sample {
                time: now,
                x: initial.0,
                y: initial.1,
            },
            times: history(now),
            page_x: history(rolling.0),
            page_y: history(rolling.1),
        }
    }

    pub fn last_x(&self) -> Option<f64> {
        self.page_x.last().copied()
    }
    pub fn last_y(&self) -> Option<f64> {
        self.page_y.last().copied()
    }

    /// Numeric arithmetic converts an absent JS array tail to NaN. DOM field
    /// assignment uses last_x/last_y instead to preserve actual undefined values.
    pub fn last(&self) -> (f64, f64) {
        (numeric(self.last_x()), numeric(self.last_y()))
    }
    pub fn translation(&self, page_x: f64, page_y: f64) -> (f64, f64) {
        let (x, y) = self.last();
        (page_x - x, page_y - y)
    }

    /// Shift before the first coordinate getter, not after resolving all values.
    pub fn begin_commit(&mut self) {
        if self.page_x.len() >= ROLLING_SAMPLES {
            self.page_x.remove(0);
            if !self.page_y.is_empty() {
                self.page_y.remove(0);
            }
            if !self.times.is_empty() {
                self.times.remove(0);
            }
        }
    }
    pub fn push_x(&mut self, value: f64) {
        self.page_x.push(value);
    }
    pub fn push_y(&mut self, value: f64) {
        self.page_y.push(value);
    }
    pub fn push_time(&mut self, value: f64) {
        self.times.push(value);
    }

    pub fn commit_move(&mut self, now: f64, page_x: f64, page_y: f64) {
        self.begin_commit();
        self.push_x(page_x);
        self.push_y(page_y);
        self.push_time(now);
    }

    /// The clocks are separate source reads. The DOM adapter applies its initial
    /// active-touch-count snapshot before scheduling a returned inertia state.
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
        let delta_time = numeric(self.times.last().copied()) - numeric(self.times.first().copied());
        let delta_x = x - numeric(self.page_x.first().copied());
        let delta_y = y - numeric(self.page_y.first().copied());
        EndGesture::Inertia(Inertia {
            time: inertia_start,
            velocity_x: delta_x.abs() / delta_time,
            direction_x: if delta_x > 0.0 { 1.0 } else { -1.0 },
            velocity_y: delta_y.abs() / delta_time,
            direction_y: if delta_y > 0.0 { 1.0 } else { -1.0 },
        })
    }
}

fn numeric(value: Option<f64>) -> f64 {
    value.unwrap_or(f64::NAN)
}
fn history(value: f64) -> Vec<f64> {
    let mut values = Vec::with_capacity(ROLLING_SAMPLES);
    values.push(value);
    values
}

#[derive(Clone, Debug)]
pub enum EndGesture {
    Tap,
    ContextMenu,
    Inertia(Inertia),
}

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
    /// Emit even the final zero frame, retaining JS nonfinite arithmetic.
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

/// Reset precedes ordinary CHANGE/CONTEXT_MENU filtering, never inertial CHANGE.
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
