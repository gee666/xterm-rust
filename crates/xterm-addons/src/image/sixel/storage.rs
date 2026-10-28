// Copyright (c) 2020 The xterm.js authors. MIT license.
//! Source SixelImageStorage policy over real shared image storage/effects.
use crate::image::storage::{
    AddImageOptions, CellSize, CursorPosition, ImageLayer, ImageResource, ImageStorage,
    StorageError, CELL_SIZE_DEFAULT,
};

/// Source addImage returns void, retaining no separate image identity table.
pub fn add_sixel_image(
    storage: &ImageStorage,
    image: ImageResource,
    scrolling: bool,
) -> Result<(), StorageError> {
    storage
        .add_image(
            image,
            AddImageOptions {
                scrolling,
                layer: ImageLayer::Top,
                z_index: 0.0,
                cursor_position: CursorPosition::Vt340,
            },
        )
        .map(|_| ())
}

/// Empty-width rasters advance rows without creating a canvas. Effects remain
/// fallible and lazy so no terminal/renderer borrow crosses a linefeed callback.
pub fn advance_sixel_cursor<E>(
    height: f64,
    scrolling: bool,
    cell_size: impl FnOnce() -> CellSize,
    mut line_feed: impl FnMut() -> Result<(), E>,
) -> Result<(), E> {
    if scrolling {
        let mut size = cell_size();
        if size.width == -1.0 || size.height == -1.0 {
            size = CELL_SIZE_DEFAULT;
        }
        let rows = (height / size.height).ceil();
        let mut i = 1.0;
        while i < rows {
            line_feed()?;
            i += 1.0;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn fallback_rounding_and_non_scrolling_match_source() {
        for (height, expected) in [
            (0.0, 0),
            (1.0, 0),
            (14.0, 0),
            (15.0, 1),
            (28.0, 1),
            (32.0, 2),
        ] {
            let count = Cell::new(0);
            advance_sixel_cursor(
                height,
                true,
                || CellSize {
                    width: -1.0,
                    height: 3.0,
                },
                || {
                    count.set(count.get() + 1);
                    Ok::<_, ()>(())
                },
            )
            .unwrap();
            assert_eq!(count.get(), expected);
        }
        advance_sixel_cursor(
            32.0,
            false,
            || panic!("source does not read cellSize when scrolling=false"),
            || -> Result<(), ()> { panic!("source does not linefeed") },
        )
        .unwrap();
    }

    #[test]
    fn fractional_metrics_and_callback_errors_preserve_order() {
        let reads = Cell::new(0);
        let count = Cell::new(0);
        let result = advance_sixel_cursor(
            8.0,
            true,
            || {
                reads.set(reads.get() + 1);
                CellSize {
                    width: 7.5,
                    height: 2.5,
                }
            },
            || {
                count.set(count.get() + 1);
                if count.get() == 2 {
                    Err("linefeed failed")
                } else {
                    Ok(())
                }
            },
        );
        assert_eq!(result, Err("linefeed failed"));
        assert_eq!(reads.get(), 1);
        assert_eq!(count.get(), 2);
    }
}
