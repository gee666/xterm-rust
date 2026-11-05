// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! One ScrollState algorithm for native numbers and raw browser values.
use super::value::{infallible, ScrollValue};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Position<V = f64> {
    pub left: V,
    pub top: V,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct PositionUpdate<V = f64> {
    pub left: Option<V>,
    pub top: Option<V>,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct DimensionUpdate<V = f64> {
    pub width: Option<V>,
    pub scroll_width: Option<V>,
    pub height: Option<V>,
    pub scroll_height: Option<V>,
}
#[derive(Clone, Copy, Debug)]
pub struct ScrollState<V = f64> {
    pub raw_left: V,
    pub raw_top: V,
    pub width: V,
    pub scroll_width: V,
    pub left: V,
    pub height: V,
    pub scroll_height: V,
    pub top: V,
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
/// Browser adapters flatten previous/current into the original event names.
#[derive(Clone, Copy, Debug)]
pub struct ScrollEvent<V = f64> {
    pub in_smooth_scrolling: bool,
    pub previous: ScrollState<V>,
    pub current: ScrollState<V>,
    pub changed: Changes,
}
impl<V: ScrollValue> ScrollState<V> {
    /// Preserve operator order, raw identity and failures of the source
    /// constructor. No state is published if a coercion throws.
    pub fn try_new(
        integer_values: bool,
        mut width: V,
        mut scroll_width: V,
        mut left: V,
        mut height: V,
        mut scroll_height: V,
        mut top: V,
    ) -> Result<Self, V::Error> {
        if integer_values {
            width = width.int32()?;
            scroll_width = scroll_width.int32()?;
            left = left.int32()?;
            height = height.int32()?;
            scroll_height = scroll_height.int32()?;
            top = top.int32()?;
        }
        let raw_left = left.clone();
        let raw_top = top.clone();
        let zero = V::number(0.0);
        if width.less(&zero)? {
            width = zero.clone();
        }
        if left.add(&width)?.greater(&scroll_width)? {
            left = scroll_width.subtract(&width)?;
        }
        if left.less(&zero)? {
            left = zero.clone();
        }
        if height.less(&zero)? {
            height = zero.clone();
        }
        if top.add(&height)?.greater(&scroll_height)? {
            top = scroll_height.subtract(&height)?;
        }
        if top.less(&zero)? {
            top = zero;
        }
        Ok(Self {
            integer_values,
            raw_left,
            raw_top,
            width,
            scroll_width,
            left,
            height,
            scroll_height,
            top,
        })
    }
    pub fn equals(&self, other: &Self) -> bool {
        self.raw_left.strict_eq(&other.raw_left)
            && self.raw_top.strict_eq(&other.raw_top)
            && self.width.strict_eq(&other.width)
            && self.scroll_width.strict_eq(&other.scroll_width)
            && self.left.strict_eq(&other.left)
            && self.height.strict_eq(&other.height)
            && self.scroll_height.strict_eq(&other.scroll_height)
            && self.top.strict_eq(&other.top)
    }
    pub fn position(&self) -> Position<V> {
        Position {
            left: self.left.clone(),
            top: self.top.clone(),
        }
    }
    pub fn try_with_dimensions(
        &self,
        update: DimensionUpdate<V>,
        raw: bool,
    ) -> Result<Self, V::Error> {
        Self::try_new(
            self.integer_values,
            update.width.unwrap_or_else(|| self.width.clone()),
            update
                .scroll_width
                .unwrap_or_else(|| self.scroll_width.clone()),
            if raw { &self.raw_left } else { &self.left }.clone(),
            update.height.unwrap_or_else(|| self.height.clone()),
            update
                .scroll_height
                .unwrap_or_else(|| self.scroll_height.clone()),
            if raw { &self.raw_top } else { &self.top }.clone(),
        )
    }
    pub fn try_with_position(&self, update: PositionUpdate<V>) -> Result<Self, V::Error> {
        Self::try_new(
            self.integer_values,
            self.width.clone(),
            self.scroll_width.clone(),
            update.left.unwrap_or_else(|| self.raw_left.clone()),
            self.height.clone(),
            self.scroll_height.clone(),
            update.top.unwrap_or_else(|| self.raw_top.clone()),
        )
    }
    pub fn event(&self, previous: Self, in_smooth_scrolling: bool) -> ScrollEvent<V> {
        let changed = Changes {
            width: !self.width.strict_eq(&previous.width),
            scroll_width: !self.scroll_width.strict_eq(&previous.scroll_width),
            left: !self.left.strict_eq(&previous.left),
            height: !self.height.strict_eq(&previous.height),
            scroll_height: !self.scroll_height.strict_eq(&previous.scroll_height),
            top: !self.top.strict_eq(&previous.top),
        };
        ScrollEvent {
            previous,
            current: self.clone(),
            in_smooth_scrolling,
            changed,
        }
    }
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
        infallible(Self::try_new(
            integer_values,
            width,
            scroll_width,
            left,
            height,
            scroll_height,
            top,
        ))
    }
    pub fn with_dimensions(&self, update: DimensionUpdate, raw: bool) -> Self {
        infallible(self.try_with_dimensions(update, raw))
    }
    pub fn with_position(&self, update: PositionUpdate) -> Self {
        infallible(self.try_with_position(update))
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
