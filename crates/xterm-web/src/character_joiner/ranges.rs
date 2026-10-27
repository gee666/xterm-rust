// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from CharacterJoinerService.ts.
use super::JoinRange;
use xterm_core::buffer::BufferLine;

/// Keep adjacent ranges separate. Overlapping ranges merge in source order.
pub fn merge_ranges(ranges: &mut Vec<JoinRange>, new: JoinRange) {
    let mut in_range = false;
    let mut index = 0;
    while index < ranges.len() {
        let range = ranges[index];
        if !in_range {
            if new[1] <= range[0] {
                ranges.insert(index, new);
                return;
            }
            if new[1] <= range[1] {
                ranges[index][0] = new[0].min(range[0]);
                return;
            }
            if new[0] < range[1] {
                ranges[index][0] = new[0].min(range[0]);
                in_range = true;
            }
            index += 1;
        } else {
            if new[1] <= range[0] {
                ranges[index - 1][1] = new[1];
                return;
            }
            if new[1] <= range[1] {
                ranges[index - 1][1] = new[1].max(range[1]);
                ranges.remove(index);
                return;
            }
            ranges.remove(index);
        }
    }
    if in_range {
        // Entering in_range requires a retained predecessor in ranges.
        if let Some(last) = ranges.last_mut() {
            last[1] = new[1];
        }
    } else {
        ranges.push(new);
    }
}

pub fn string_ranges_to_cell_ranges(ranges: &mut [JoinRange], line: &BufferLine, start_col: usize) {
    if ranges.is_empty() {
        return;
    }
    let mut range_index = 0;
    let mut started = false;
    let mut string_index = 0;
    let trimmed_length = line.get_trimmed_length();
    for x in start_col..trimmed_length {
        if line.get_width(x) == Some(0) {
            continue;
        }
        if !started && ranges[range_index][0] <= string_index {
            ranges[range_index][0] = x;
            started = true;
        }
        if ranges[range_index][1] <= string_index {
            ranges[range_index][1] = x;
            range_index += 1;
            if range_index == ranges.len() {
                break;
            }
            if ranges[range_index][0] <= string_index {
                ranges[range_index][0] = x;
                started = true;
            } else {
                started = false;
            }
        }
        string_index += line.get_string(x).map_or(1, |s| s.len().max(1));
    }
    if let Some(range) = ranges.get_mut(range_index) {
        range[1] = trimmed_length;
    }
}
