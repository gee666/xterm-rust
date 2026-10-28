// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Winpty linefeed wrap heuristic, independent of the current operating system.
use crate::{
    buffer::{
        constants::{NULL_CELL_CODE, WHITESPACE_CELL_CODE},
        BufferLine,
    },
    circular_list::CircularList,
};
use std::{cell::RefCell, rc::Rc};

/// Marks the current row using the preceding row's final codepoint. Only a null
/// cell and U+0020 count as whitespace here. An absent prior row leaves the
/// current row untouched; an out-of-range cell acts as null, matching legacy
/// BufferLine.get's JavaScript bitwise coercion. RefCell borrows end before mutation,
/// including when circular storage aliases both rows.
pub fn update_windows_mode_wrapped_state(
    lines: &CircularList<Rc<RefCell<BufferLine>>>,
    ybase: usize,
    y: usize,
    cols: usize,
) {
    let Some(current) = ybase.checked_add(y) else {
        return;
    };
    let previous = current
        .checked_sub(1)
        .map_or_else(|| lines.get_signed(-1), |index| lines.get(index));
    let wrapped = previous.map(|line| {
        let line = line.borrow();
        cols.checked_sub(1).is_some_and(|column| {
            line.get_code_point(column).map_or_else(
                // An empty combined string produces NaN from charCodeAt(-1)
                // upstream, which compares unequal to both excluded codes.
                || line.is_combined(column).is_some_and(|flags| flags != 0),
                |code| code != NULL_CELL_CODE && code != WHITESPACE_CELL_CODE,
            )
        })
    });
    if let (Some(wrapped), Some(line)) = (wrapped, lines.get(current)) {
        line.borrow_mut().is_wrapped = wrapped;
    }
}
