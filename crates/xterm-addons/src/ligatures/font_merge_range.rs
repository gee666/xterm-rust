// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::font_bytes::FontError;

pub fn merge(ranges: &mut Vec<[i64; 2]>, begin: i64, end: i64) -> Result<(), FontError> {
    let mut in_range = false;
    let mut index = 0;
    while index < ranges.len() {
        let range = ranges[index];
        if !in_range {
            if end <= range[0] {
                ranges.try_reserve(1)?;
                ranges.insert(index, [begin, end]);
                return Ok(());
            }
            if end <= range[1] {
                ranges[index][0] = begin.min(range[0]);
                return Ok(());
            }
            if begin < range[1] {
                ranges[index][0] = begin.min(range[0]);
                in_range = true;
            }
        } else {
            // in_range was established in an earlier iteration, and removals
            // retain that preceding entry, so index is always at least one.
            if end <= range[0] {
                ranges[index - 1][1] = end;
                return Ok(());
            }
            if end <= range[1] {
                ranges[index - 1][1] = end.max(range[1]);
                ranges.remove(index);
                return Ok(());
            }
            ranges.remove(index);
            continue;
        }
        index += 1;
    }
    if in_range {
        ranges
            .last_mut()
            .ok_or(FontError::Invalid("Missing preceding ligature range"))?[1] = end;
    } else {
        ranges.try_reserve(1)?;
        ranges.push([begin, end]);
    }
    Ok(())
}
