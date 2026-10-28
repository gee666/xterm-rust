// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{placement::tile_at, ImageStorage, StorageError};

impl ImageStorage {
    pub fn viewport_resize(&self, cols: usize, rows: usize) -> Result<(), StorageError> {
        let (old_cols, _) = self.inner.viewport_metrics.get();
        if self.image_count() == 0 || old_cols >= cols {
            self.inner.viewport_metrics.set((cols, rows));
            return Ok(());
        }
        let Some(old_col) = old_cols.checked_sub(1) else {
            // Source getExtended(-1) has no extended entry.
            self.inner.viewport_metrics.set((cols, rows));
            return Ok(());
        };
        let buffer = self.inner.terminal.buffer_service().buffer();
        let count = buffer
            .try_borrow()
            .map_err(|_| StorageError::BorrowedBuffer)?
            .lines
            .len();
        for row in 0..count {
            let line = buffer
                .try_borrow()
                .map_err(|_| StorageError::BorrowedBuffer)?
                .lines
                .get(row)
                .ok_or(StorageError::MissingLine(row))?;
            let tile = {
                let line = line
                    .try_borrow()
                    .map_err(|_| StorageError::BorrowedBuffer)?;
                tile_at(&line, old_col)
            };
            let Some(tile) = tile.filter(|tile| tile.image_id.get() != -1) else {
                continue;
            };
            let Some(spec) = u64::try_from(tile.image_id.get())
                .ok()
                .and_then(|id| self.image_spec(id))
            else {
                continue;
            };
            let actual = spec.actual_resource();
            let width = match actual {
                Some(image) => f64::from(image.width()?),
                None => 0.0,
            };
            let per_row = (width / spec.actual_cell_size.get().width).ceil();
            let remainder = tile.tile_id.get() % per_row;
            if remainder + 1.0 >= per_row {
                continue;
            }
            // Upstream's content-check loop uses rightCol > newCols. For a
            // width enlargement it executes zero times, so even nonempty right
            // cells are overwritten. Do not silently fix that source quirk.
            let proposed = per_row - remainder + old_col as f64;
            if proposed.is_nan() {
                continue;
            }
            let end = proposed.min(cols as f64);
            let mut tile_id = tile.tile_id.get();
            let mut col = old_col + 1;
            while (col as f64) < end {
                tile_id += 1.0;
                let mut line = line
                    .try_borrow_mut()
                    .map_err(|_| StorageError::BorrowedBuffer)?;
                self.write_to_cell(&mut line, col, spec.id, tile_id)?;
                spec.tile_count.set(spec.tile_count.get() + 1.0);
                col += 1;
            }
        }
        self.inner.viewport_metrics.set((cols, rows));
        Ok(())
    }
}
