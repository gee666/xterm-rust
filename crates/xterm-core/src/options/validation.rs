// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Adapted from common/services/OptionsService.ts.
use super::{OptionError, OptionValue as V};

pub(super) fn sanitize(key: &'static str, mut value: V) -> Result<V, OptionError> {
    match key {
        "cursorStyle" => {
            if !value.is_truthy() {
                value = V::from("block");
            }
            if !["block", "underline", "bar"]
                .iter()
                .any(|s| value == V::from(*s))
            {
                return Err(OptionError::invalid(
                    "\"",
                    &value,
                    &format!("\" is not a valid value for {key}"),
                )?);
            }
        }
        "wordSeparator" if !value.is_truthy() => value = V::from(" ()[]{}',\"`"),
        "fontWeight" | "fontWeightBold" => {
            let numeric = matches!(value, V::Number(n) if (1.0..=1000.0).contains(&n));
            let keyword = [
                "normal", "bold", "100", "200", "300", "400", "500", "600", "700", "800", "900",
            ]
            .iter()
            .any(|s| value == V::from(*s));
            if !numeric && !keyword {
                value = V::from(if key == "fontWeight" {
                    "normal"
                } else {
                    "bold"
                });
            }
        }
        "blinkIntervalDuration" => {
            value = V::Number(value.number()?.floor());
            lower_bound(key, &value, 0.0, false)?;
        }
        "cursorWidth" => {
            value = V::Number(value.number()?.floor());
            lower_bound(key, &value, 1.0, false)?;
        }
        "lineHeight" | "tabStopWidth" => lower_bound(key, &value, 1.0, false)?,
        "minimumContrastRatio" => {
            let n = value.multiply(10.0)?;
            // JS Math.round rounds ties toward +Infinity, unlike f64::round.
            let rounded = if n.is_finite() {
                n.floor() + f64::from(n - n.floor() >= 0.5)
            } else {
                n
            };
            let rounded = rounded / 10.0;
            value = V::Number(if rounded.is_nan() {
                f64::NAN
            } else {
                rounded.clamp(1.0, 21.0)
            });
        }
        "scrollback" => {
            let n = value.number()?;
            // Rust's min ignores NaN, whereas JS Math.min propagates it.
            value = V::Number(if n.is_nan() {
                n
            } else {
                n.min(4_294_967_295.0)
            });
            lower_bound(key, &value, 0.0, false)?;
        }
        "fastScrollSensitivity" | "scrollSensitivity" => lower_bound(key, &value, 0.0, true)?,
        "rows" | "cols" if !value.is_truthy() && value != V::Number(0.0) => {
            return Err(OptionError::invalid(
                &format!("{key} must be numeric, value: "),
                &value,
                "",
            )?);
        }
        "windowsPty" if matches!(value, V::Null | V::Undefined) => {
            value = V::from(super::WindowsPty::default())
        }
        _ => {}
    }
    Ok(value)
}

fn lower_bound(key: &str, value: &V, bound: f64, inclusive: bool) -> Result<(), OptionError> {
    let rejected = if inclusive {
        value.less_equal(bound)?
    } else {
        value.less_than(bound)?
    };
    if rejected {
        let relation = if inclusive {
            "less than or equal to"
        } else {
            "less than"
        };
        return Err(OptionError::invalid(
            &format!("{key} cannot be {relation} {bound}, value: "),
            value,
            "",
        )?);
    }
    Ok(())
}
