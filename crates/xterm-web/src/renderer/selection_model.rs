// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
use super::row_types::Selection;
#[derive(Clone, Copy, Debug, Default)]
pub struct SelectionRenderModel {
    pub has_selection: bool,
    pub selection: Selection,
    pub viewport_start: i128,
    pub viewport_end: i128,
    pub capped_start: i128,
    pub capped_end: i128,
}
impl SelectionRenderModel {
    pub fn update(rows: usize, viewport_y: usize, selection: Selection) -> Self {
        let (Some(start), Some(end)) = (selection.start, selection.end) else {
            return Self::default();
        };
        if start == end {
            return Self::default();
        }
        let viewport_start = start[1] as i128 - viewport_y as i128;
        let viewport_end = end[1] as i128 - viewport_y as i128;
        let capped_start = viewport_start.max(0);
        let capped_end = viewport_end.min(rows as i128 - 1);
        if capped_start >= rows as i128 || capped_end < 0 {
            return Self::default();
        }
        Self {
            has_selection: true,
            selection,
            viewport_start,
            viewport_end,
            capped_start,
            capped_end,
        }
    }
}
