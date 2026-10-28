// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
//! Stateful wheel and touch accumulation from browser/services/MouseService.ts.

const DOM_DELTA_PIXEL: f64 = 0.0;
const DOM_DELTA_PAGE: f64 = 2.0;
pub const TRACKPAD_THRESHOLD: f64 = 50.0;
pub const TRACKPAD_MULTIPLIER: f64 = 0.3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WheelInput {
    pub delta_y: f64,
    pub delta_mode: f64,
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelOptions {
    pub scroll_sensitivity: f64,
    pub fast_scroll_sensitivity: f64,
    pub rows: f64,
}

impl Default for WheelOptions {
    fn default() -> Self {
        Self {
            scroll_sensitivity: 1.0,
            fast_scroll_sensitivity: 5.0,
            rows: 24.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WheelState {
    partial_scroll: f64,
    touch_accumulator: f64,
}

impl WheelState {
    pub fn reset(&mut self) {
        self.partial_scroll = 0.0;
        self.touch_accumulator = 0.0;
    }

    pub fn partial_scroll(&self) -> f64 {
        self.partial_scroll
    }
    pub fn touch_accumulator(&self) -> f64 {
        self.touch_accumulator
    }

    /// Cell height is in device pixels. Both metrics must be present, but the
    /// source does not substitute defaults or reject zero/nonfinite metrics.
    pub fn consume(
        &mut self,
        input: WheelInput,
        cell_height: Option<f64>,
        dpr: Option<f64>,
        options: WheelOptions,
    ) -> f64 {
        if input.delta_y == 0.0 || input.shift {
            return 0.0;
        }
        let (Some(cell_height), Some(dpr)) = (cell_height, dpr) else {
            return 0.0;
        };
        let mut amount = input.delta_y;
        if input.alt || input.ctrl || input.shift {
            amount = amount * options.fast_scroll_sensitivity * options.scroll_sensitivity;
        } else {
            amount *= options.scroll_sensitivity;
        }
        if input.delta_mode == DOM_DELTA_PIXEL {
            amount /= cell_height / dpr;
            if input.delta_y.abs() < TRACKPAD_THRESHOLD {
                amount *= TRACKPAD_MULTIPLIER;
            }
            amount = self.accumulate_pixels(amount);
        } else if input.delta_mode == DOM_DELTA_PAGE {
            amount *= options.rows;
        }
        amount
    }

    /// Commit already-scaled pixels only after the DOM adapter finishes its
    /// fallible, source-ordered property reads. No host callback runs here.
    pub fn accumulate_pixels(&mut self, amount: f64) -> f64 {
        self.partial_scroll += amount;
        let whole =
            self.partial_scroll.abs().floor() * if self.partial_scroll > 0.0 { 1.0 } else { -1.0 };
        self.partial_scroll %= 1.0;
        whole
    }

    pub fn touch_start(&mut self) {
        self.touch_accumulator = 0.0;
    }

    /// Return signed whole-line count. The adapter emits one cursor key or
    /// mouse-wheel report per absolute line, or delegates the gesture to the
    /// viewport when it has scrollback and no mouse wheel reporting is active.
    pub fn consume_touch(&mut self, translation_y: f64, css_cell_height: Option<f64>) -> f64 {
        let Some(height) = css_cell_height else {
            return 0.0;
        };
        if height == 0.0 || height.is_nan() {
            return 0.0;
        }
        self.touch_accumulator -= translation_y;
        let lines = (self.touch_accumulator / height).trunc();
        if lines != 0.0 {
            self.touch_accumulator -= lines * height;
        }
        lines
    }
}

/// Passive wheel sends one cursor sequence regardless of the consumed magnitude.
/// Touch sends this same sequence once per whole line returned by consume_touch.
pub fn cursor_sequence(up: bool, application_cursor: bool) -> [u16; 3] {
    [
        0x1b,
        u16::from(if application_cursor { b'O' } else { b'[' }),
        u16::from(if up { b'A' } else { b'B' }),
    ]
}
