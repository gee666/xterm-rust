// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! ScrollState from browser/scrollable/scrollable.ts. Number fields retain JS
//! NaN, infinity, signed zero and bitwise ToInt32 behavior at this boundary.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Position {
    pub left: f64,
    pub top: f64,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct PositionUpdate {
    pub left: Option<f64>,
    pub top: Option<f64>,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct DimensionUpdate {
    pub width: Option<f64>,
    pub scroll_width: Option<f64>,
    pub height: Option<f64>,
    pub scroll_height: Option<f64>,
}
#[derive(Clone, Copy, Debug)]
pub struct ScrollState {
    pub raw_left: f64,
    pub raw_top: f64,
    pub width: f64,
    pub scroll_width: f64,
    pub left: f64,
    pub height: f64,
    pub scroll_height: f64,
    pub top: f64,
    integer_values: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Changes {
    pub width: bool,
    pub scroll_width: bool,
    pub left: bool,
    pub height: bool,
    pub scroll_height: bool,
    pub top: bool,
}
/// Previous/current records group the source's oldWidth/width and equivalent
/// fields. Browser adapters flatten these into the original event names.
#[derive(Clone, Copy, Debug)]
pub struct ScrollEvent {
    pub in_smooth_scrolling: bool,
    pub previous: ScrollState,
    pub current: ScrollState,
    pub changed: Changes,
}
impl ScrollState {
    pub fn new(
        integer_values: bool,
        width: f64,
        scroll_width: f64,
        left: f64,
        height: f64,
        scroll_height: f64,
        top: f64,
    ) -> Self {
        let [mut width, scroll_width, mut left, mut height, scroll_height, mut top] =
            [width, scroll_width, left, height, scroll_height, top].map(|value| {
                if integer_values {
                    to_int32(value)
                } else {
                    value
                }
            });
        let raw_left = left;
        let raw_top = top;
        if width < 0.0 {
            width = 0.0;
        }
        if left + width > scroll_width {
            left = scroll_width - width;
        }
        if left < 0.0 {
            left = 0.0;
        }
        if height < 0.0 {
            height = 0.0;
        }
        if top + height > scroll_height {
            top = scroll_height - height;
        }
        if top < 0.0 {
            top = 0.0;
        }
        Self {
            integer_values,
            raw_left,
            raw_top,
            width,
            scroll_width,
            left,
            height,
            scroll_height,
            top,
        }
    }
    pub fn equals(&self, other: &Self) -> bool {
        self.raw_left == other.raw_left
            && self.raw_top == other.raw_top
            && self.width == other.width
            && self.scroll_width == other.scroll_width
            && self.left == other.left
            && self.height == other.height
            && self.scroll_height == other.scroll_height
            && self.top == other.top
    }
    pub fn position(&self) -> Position {
        Position {
            left: self.left,
            top: self.top,
        }
    }
    pub fn with_dimensions(&self, update: DimensionUpdate, raw: bool) -> Self {
        Self::new(
            self.integer_values,
            update.width.unwrap_or(self.width),
            update.scroll_width.unwrap_or(self.scroll_width),
            if raw { self.raw_left } else { self.left },
            update.height.unwrap_or(self.height),
            update.scroll_height.unwrap_or(self.scroll_height),
            if raw { self.raw_top } else { self.top },
        )
    }
    pub fn with_position(&self, update: PositionUpdate) -> Self {
        Self::new(
            self.integer_values,
            self.width,
            self.scroll_width,
            update.left.unwrap_or(self.raw_left),
            self.height,
            self.scroll_height,
            update.top.unwrap_or(self.raw_top),
        )
    }
    pub fn event(&self, previous: Self, in_smooth_scrolling: bool) -> ScrollEvent {
        ScrollEvent {
            previous,
            current: *self,
            in_smooth_scrolling,
            changed: Changes {
                width: self.width != previous.width,
                scroll_width: self.scroll_width != previous.scroll_width,
                left: self.left != previous.left,
                height: self.height != previous.height,
                scroll_height: self.scroll_height != previous.scroll_height,
                top: self.top != previous.top,
            },
        }
    }
}
fn to_int32(value: f64) -> f64 {
    const MODULUS: f64 = 4_294_967_296.0;
    const SIGN_BIT: f64 = 2_147_483_648.0;
    if !value.is_finite() || value == 0.0 {
        return 0.0;
    }
    let unsigned = value.trunc().rem_euclid(MODULUS);
    if unsigned == 0.0 {
        0.0
    } else if unsigned >= SIGN_BIT {
        unsigned - MODULUS
    } else {
        unsigned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_position_survives_clamping_and_dimension_growth() {
        let state = ScrollState::new(false, 10.0, 20.0, 19.5, 10.0, 20.0, 30.5);
        assert_eq!(
            state.position(),
            Position {
                left: 10.0,
                top: 10.0
            }
        );
        assert_eq!(state.raw_top, 30.5);
        let enlarged = state.with_dimensions(
            DimensionUpdate {
                scroll_height: Some(100.0),
                ..Default::default()
            },
            true,
        );
        assert_eq!(enlarged.top, 30.5);
        assert_eq!(
            state
                .with_dimensions(DimensionUpdate::default(), false)
                .raw_top,
            10.0
        );
    }
    #[test]
    fn bitwise_numbers_and_nan_comparisons_match_js() {
        let state = ScrollState::new(
            true,
            4_294_967_301.8,
            f64::INFINITY,
            f64::NAN,
            -3.0,
            50.0,
            -0.0,
        );
        assert_eq!(state.width, 5.0);
        assert_eq!(state.scroll_width, 0.0);
        assert_eq!(state.height, 0.0);
        assert!(!state.top.is_sign_negative());
        let nan = ScrollState::new(false, f64::NAN, 1.0, 0.0, 1.0, 2.0, f64::NAN);
        assert!(!nan.equals(&nan));
        assert!(nan.event(nan, false).changed.top);
    }
}
