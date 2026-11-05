// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::ops::*;
use wasm_bindgen::JsValue;

pub(super) fn merge(ranges: &JsValue, new: &JsValue) -> Result<(), JsValue> {
    let mut in_range = false;
    let mut i = 0.0;
    while less(&i.into(), &length(ranges)?)? {
        let range = at(ranges, i)?;
        if !in_range {
            if le(&at(new, 1.0)?, &at(&range, 0.0)?)? {
                return insert(ranges, i, new);
            }
            if le(&at(new, 1.0)?, &at(&range, 1.0)?)? {
                return put(&range, 0.0, &min_start(new, &range)?);
            }
            if less(&at(new, 0.0)?, &at(&range, 1.0)?)? {
                put(&range, 0.0, &min_start(new, &range)?)?;
                in_range = true;
            }
            i += 1.0;
        } else {
            if le(&at(new, 1.0)?, &at(&range, 0.0)?)? {
                return put(&at(ranges, i - 1.0)?, 1.0, &at(new, 1.0)?);
            }
            if le(&at(new, 1.0)?, &at(&range, 1.0)?)? {
                put(&at(ranges, i - 1.0)?, 1.0, &max_end(new, &range)?)?;
                return remove(ranges, i);
            }
            remove(ranges, i)?;
        }
    }
    if in_range {
        // Preserve the source's subtraction conversion on a raw length.
        // Array length is normally numeric, but custom array-likes may coerce.
        let index = subtract_one(&length(ranges)?)?;
        put(&at(ranges, index)?, 1.0, &at(new, 1.0)?)
    } else {
        push(ranges, new)
    }
}
