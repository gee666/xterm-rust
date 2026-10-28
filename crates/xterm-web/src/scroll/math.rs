//! Numeric operations shared by scroll state, wheel classification and widgets.
pub(crate) use crate::links::debounce::{js_max as max, js_min as min};

/// JavaScript ties toward positive infinity and preserves negative zero. Adding
/// 0.5 before flooring would misround the representable value just below 0.5.
pub(crate) fn round(value: f64) -> f64 {
    if !value.is_finite() || value == 0.0 {
        return value;
    }
    let floor = value.floor();
    let result = if value - floor < 0.5 {
        floor
    } else {
        floor + 1.0
    };
    if result == 0.0 {
        0.0_f64.copysign(value)
    } else {
        result
    }
}
