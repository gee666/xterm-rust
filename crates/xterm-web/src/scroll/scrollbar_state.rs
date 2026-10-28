// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
// Port of browser/scrollable/scrollbarState.ts.

use super::math::{max as js_max, round as js_round};

const MINIMUM_SLIDER_SIZE: f64 = 20.0;

/// Geometry retains JavaScript numbers, including negative zero and nonfinite
/// values. Clamping the desired scroll position belongs to Scrollable, not here.
#[derive(Debug)]
pub struct ScrollbarState {
    scrollbar_size: f64,
    opposite_scrollbar_size: f64,
    arrow_size: f64,
    visible_size: f64,
    scroll_size: f64,
    scroll_position: f64,
    available_size: f64,
    needed: bool,
    slider_size: f64,
    slider_ratio: f64,
    slider_position: f64,
}

impl Clone for ScrollbarState {
    fn clone(&self) -> Self {
        // The source reconstructs and recomputes, including an opposite size
        // whose setter deliberately did not refresh the original instance.
        Self::new(
            self.arrow_size,
            self.scrollbar_size,
            self.opposite_scrollbar_size,
            self.visible_size,
            self.scroll_size,
            self.scroll_position,
        )
    }
}

impl ScrollbarState {
    pub fn new(
        arrow_size: f64,
        scrollbar_size: f64,
        opposite_scrollbar_size: f64,
        visible_size: f64,
        scroll_size: f64,
        scroll_position: f64,
    ) -> Self {
        let mut state = Self {
            scrollbar_size: js_round(scrollbar_size),
            opposite_scrollbar_size: js_round(opposite_scrollbar_size),
            arrow_size: js_round(arrow_size),
            visible_size,
            scroll_size,
            scroll_position,
            available_size: 0.0,
            needed: false,
            slider_size: 0.0,
            slider_ratio: 0.0,
            slider_position: 0.0,
        };
        state.refresh();
        state
    }

    pub fn set_visible_size(&mut self, value: f64) -> bool {
        let value = js_round(value);
        if self.visible_size == value {
            return false;
        }
        self.visible_size = value;
        self.refresh();
        true
    }

    pub fn set_scroll_size(&mut self, value: f64) -> bool {
        let value = js_round(value);
        if self.scroll_size == value {
            return false;
        }
        self.scroll_size = value;
        self.refresh();
        true
    }

    pub fn set_scroll_position(&mut self, value: f64) -> bool {
        let value = js_round(value);
        if self.scroll_position == value {
            return false;
        }
        self.scroll_position = value;
        self.refresh();
        true
    }

    pub fn set_scrollbar_size(&mut self, value: f64) {
        self.scrollbar_size = js_round(value);
    }
    pub fn set_opposite_scrollbar_size(&mut self, value: f64) {
        self.opposite_scrollbar_size = js_round(value);
    }
    pub fn set_arrow_size(&mut self, value: f64) {
        let value = js_round(value);
        if self.arrow_size != value {
            self.arrow_size = value;
            self.refresh();
        }
    }

    pub fn get_arrow_size(&self) -> f64 {
        self.arrow_size
    }
    pub fn get_scroll_position(&self) -> f64 {
        self.scroll_position
    }
    pub fn get_rectangle_large_size(&self) -> f64 {
        self.available_size
    }
    pub fn get_rectangle_small_size(&self) -> f64 {
        self.scrollbar_size
    }
    pub fn is_needed(&self) -> bool {
        self.needed
    }
    pub fn get_slider_size(&self) -> f64 {
        self.slider_size
    }
    pub fn get_slider_position(&self) -> f64 {
        self.slider_position
    }

    pub fn get_desired_scroll_position_from_offset(&self, offset: f64) -> f64 {
        if !self.needed {
            return 0.0;
        }
        let desired = offset - self.arrow_size - self.slider_size / 2.0;
        js_round(desired / self.slider_ratio)
    }

    pub fn get_desired_scroll_position_from_offset_paged(&self, offset: f64) -> f64 {
        if !self.needed {
            return 0.0;
        }
        if offset - self.arrow_size < self.slider_position {
            self.scroll_position - self.visible_size
        } else {
            self.scroll_position + self.visible_size
        }
    }

    pub fn get_desired_scroll_position_from_delta(&self, delta: f64) -> f64 {
        if !self.needed {
            return 0.0;
        }
        js_round((self.slider_position + delta) / self.slider_ratio)
    }

    fn refresh(&mut self) {
        let available = js_max(0.0, self.visible_size - self.opposite_scrollbar_size);
        let representable = js_max(0.0, available - 2.0 * self.arrow_size);
        self.needed = self.scroll_size > 0.0 && self.scroll_size > self.visible_size;
        self.available_size = js_round(available);
        if !self.needed {
            self.slider_size = js_round(representable);
            self.slider_ratio = 0.0;
            self.slider_position = 0.0;
            return;
        }
        let proportional = (self.visible_size * representable / self.scroll_size).floor();
        let size = js_max(MINIMUM_SLIDER_SIZE, proportional);
        self.slider_size = js_round(size);
        self.slider_ratio =
            (representable - self.slider_size) / (self.scroll_size - self.visible_size);
        self.slider_position = js_round(self.scroll_position * self.slider_ratio);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_rounding_and_nan_propagation_are_not_rust_round_or_max() {
        assert_eq!(js_round(0.49999999999999994), 0.0);
        assert_eq!(js_round(-1.5), -1.0);
        assert!(js_round(-0.5).is_sign_negative());
        assert!(js_round(-0.0).is_sign_negative());
        let state = ScrollbarState::new(0.0, 10.0, 0.0, f64::NAN, 100.0, 0.0);
        assert!(!state.is_needed());
        assert!(state.get_rectangle_large_size().is_nan());
        assert!(state.get_slider_size().is_nan());
    }

    #[test]
    fn opposite_size_is_deferred_but_clone_refreshes_it() {
        let mut state = ScrollbarState::new(0.0, 10.0, 0.0, 100.0, 200.0, 40.0);
        assert_eq!(state.get_rectangle_large_size(), 100.0);
        state.set_opposite_scrollbar_size(20.0);
        assert_eq!(state.get_rectangle_large_size(), 100.0);
        assert!(!state.set_visible_size(100.0));
        assert_eq!(state.clone().get_rectangle_large_size(), 80.0);
        state.set_arrow_size(5.0);
        assert_eq!(state.get_rectangle_large_size(), 80.0);
    }

    #[test]
    fn minimum_slider_can_exceed_track_and_desired_values_are_not_clamped() {
        let state = ScrollbarState::new(0.0, 1.0, 0.0, 10.0, 100.0, 0.0);
        assert!(state.is_needed());
        assert_eq!(state.get_slider_size(), 20.0);
        assert!(state.get_desired_scroll_position_from_delta(1.0) < 0.0);
        assert_eq!(
            state.get_desired_scroll_position_from_offset_paged(-1.0),
            -10.0
        );
        assert_eq!(
            state.get_desired_scroll_position_from_offset_paged(0.0),
            10.0
        );
    }
}
