// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Adapted from browser/input/CompositionHelper.ts.

#[derive(Clone, Copy, Debug)]
pub struct CursorGeometry {
    pub in_viewport: bool,
    pub x: usize,
    pub y: usize,
    pub cols: usize,
    pub cell_width: f64,
    pub cell_height: f64,
    pub overlay_width: f64,
    pub overlay_height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompositionGeometry {
    pub left: f64,
    pub top: f64,
    pub height: f64,
    pub max_width: f64,
    pub textarea_width: f64,
    pub textarea_height: f64,
    pub textarea_line_height: f64,
}

pub fn composition_geometry(cursor: CursorGeometry) -> Option<CompositionGeometry> {
    if !cursor.in_viewport || cursor.cols == 0 {
        return None;
    }
    let left = cursor.x.min(cursor.cols - 1) as f64 * cursor.cell_width;
    Some(CompositionGeometry {
        left,
        top: cursor.y as f64 * cursor.cell_height,
        height: cursor.cell_height,
        max_width: cursor.cols as f64 * cursor.cell_width - left,
        textarea_width: cursor.overlay_width.max(1.0),
        textarea_height: cursor.overlay_height.max(1.0),
        textarea_line_height: cursor.overlay_height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clamps_wrap_pending_cursor_and_minimum_textarea() {
        let mut cursor = CursorGeometry {
            in_viewport: true,
            x: 80,
            y: 3,
            cols: 80,
            cell_width: 8.5,
            cell_height: 17.0,
            overlay_width: 0.0,
            overlay_height: 0.0,
        };
        assert_eq!(
            composition_geometry(cursor),
            Some(CompositionGeometry {
                left: 671.5,
                top: 51.0,
                height: 17.0,
                max_width: 8.5,
                textarea_width: 1.0,
                textarea_height: 1.0,
                textarea_line_height: 0.0,
            })
        );
        cursor.in_viewport = false;
        assert_eq!(composition_geometry(cursor), None);
    }
}
