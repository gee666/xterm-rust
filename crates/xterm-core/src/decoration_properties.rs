// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use crate::options::OptionValue as Value;

impl Decoration {
    /// Read one property at its point of use. Host getters run without a native
    /// options borrow, so they may register, dispose or mutate decorations.
    pub fn property(&self, key: &str) -> Result<Value, DecorationError> {
        if let Some(source) = &self.state.source {
            return Ok(source.get_property(key)?);
        }
        let options = self.state.options.borrow();
        let number = |value: Option<f64>| value.map_or(Value::Undefined, Value::Number);
        Ok(match key {
            "x" => number(options.x),
            "width" => number(options.width),
            "height" => number(options.height.map(f64::from)),
            "layer" => options.layer.map_or(Value::Undefined, |layer| match layer {
                DecorationLayer::Top => "top".into(),
                DecorationLayer::Bottom => "bottom".into(),
            }),
            "anchor" => options
                .anchor
                .map_or(Value::Undefined, |anchor| match anchor {
                    DecorationAnchor::Left => "left".into(),
                    DecorationAnchor::Right => "right".into(),
                }),
            "backgroundColor" => options
                .background_color
                .as_deref()
                .map_or(Value::Undefined, Into::into),
            "foregroundColor" => options
                .foreground_color
                .as_deref()
                .map_or(Value::Undefined, Into::into),
            "overviewRulerOptions" => {
                options
                    .overview_ruler_options
                    .as_ref()
                    .map_or(Value::Undefined, |overview| {
                        Value::property_map([
                            ("color", overview.color.as_str().into()),
                            (
                                "position",
                                overview.position.map_or(
                                    Value::Undefined,
                                    |position| match position {
                                        OverviewPosition::Full => "full".into(),
                                        OverviewPosition::Left => "left".into(),
                                        OverviewPosition::Center => "center".into(),
                                        OverviewPosition::Right => "right".into(),
                                    },
                                ),
                            ),
                        ])
                    })
            }
            _ => Value::Undefined,
        })
    }
    pub(super) fn height(&self) -> Result<Value, DecorationError> {
        Ok(nullish(self.property("height")?, 1.0.into()))
    }
    pub(super) fn matches(
        &self,
        x: f64,
        layer: Option<DecorationLayer>,
    ) -> Result<bool, DecorationError> {
        let xmin = nullish(self.property("x")?, 0.0.into());
        let xmax = xmin.js_add(&nullish(self.property("width")?, 1.0.into()))?;
        if !(x >= xmin.to_number()? && x < xmax.to_number()?) {
            return Ok(false);
        }
        Ok(match layer {
            None => true,
            Some(layer) => {
                nullish(self.property("layer")?, "bottom".into())
                    == match layer {
                        DecorationLayer::Top => "top".into(),
                        DecorationLayer::Bottom => "bottom".into(),
                    }
            }
        })
    }
}
pub fn nullish(value: Value, fallback: Value) -> Value {
    if matches!(value, Value::Undefined | Value::Null) {
        fallback
    } else {
        value
    }
}
