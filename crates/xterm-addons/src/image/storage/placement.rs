// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{
    AddImageOptions, CellSize, CursorPosition, ImageBufferType, ImageResource, ImageSpec,
    ImageStorage, ImageTileInfo, StorageError, CELL_SIZE_DEFAULT,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    buffer::{BufferLine, CellData},
    event::EmitError,
};

impl ImageStorage {
    pub fn add_image(
        &self,
        image: ImageResource,
        options: AddImageOptions,
    ) -> Result<u64, StorageError> {
        self.evict_oldest(f64::from(image.width()?) * f64::from(image.height()?))?;
        let mut size = self.inner.renderer.cell_size()?;
        if size.width == -1.0 || size.height == -1.0 {
            size = CELL_SIZE_DEFAULT;
        }
        let (cols, rows) = image_cells(&image, size)?;
        let image_id = self.following_id();
        let service = self.inner.terminal.buffer_service();
        let buffer = service.buffer();
        let (term_cols, term_rows) = (service.cols(), service.rows());
        let (origin_x, origin_y) = {
            let buffer = buffer
                .try_borrow()
                .map_err(|_| StorageError::BorrowedBuffer)?;
            (buffer.x, buffer.y)
        };
        let offset = if options.scrolling {
            origin_x
        } else {
            let mut buffer = buffer
                .try_borrow_mut()
                .map_err(|_| StorageError::BorrowedBuffer)?;
            buffer.x = 0;
            buffer.y = 0;
            0
        };
        let y = buffer
            .try_borrow()
            .map_err(|_| StorageError::BorrowedBuffer)?
            .y;
        self.inner.terminal.mark_dirty(y)?;
        let mut tile_count = 0.0;
        for row in 0..rows {
            let line = {
                let buffer = buffer
                    .try_borrow()
                    .map_err(|_| StorageError::BorrowedBuffer)?;
                let index = buffer
                    .y
                    .checked_add(buffer.ybase)
                    .ok_or(StorageError::InvalidGeometry)?;
                buffer
                    .lines
                    .get(index)
                    .ok_or(StorageError::MissingLine(index))?
            };
            for col in 0..cols.min(term_cols.saturating_sub(offset)) {
                let mut line = line
                    .try_borrow_mut()
                    .map_err(|_| StorageError::BorrowedBuffer)?;
                self.write_to_cell(
                    &mut line,
                    offset + col,
                    image_id,
                    (row as f64) * (cols as f64) + col as f64,
                )?;
                tile_count += 1.0;
            }
            if options.scrolling {
                if row + 1 < rows {
                    self.inner.terminal.line_feed()?;
                }
            } else {
                let mut buffer = buffer
                    .try_borrow_mut()
                    .map_err(|_| StorageError::BorrowedBuffer)?;
                buffer.y = buffer
                    .y
                    .checked_add(1)
                    .ok_or(StorageError::InvalidGeometry)?;
                if buffer.y >= term_rows {
                    break;
                }
            }
            buffer
                .try_borrow_mut()
                .map_err(|_| StorageError::BorrowedBuffer)?
                .x = offset;
        }
        let y = buffer
            .try_borrow()
            .map_err(|_| StorageError::BorrowedBuffer)?
            .y;
        self.inner.terminal.mark_dirty(y)?;
        {
            let mut buffer = buffer
                .try_borrow_mut()
                .map_err(|_| StorageError::BorrowedBuffer)?;
            if options.scrolling {
                buffer.x = if options.cursor_position == CursorPosition::Iip {
                    offset.saturating_add(cols).min(term_cols)
                } else {
                    offset
                };
            } else {
                buffer.x = origin_x;
                buffer.y = origin_y;
            }
        }
        self.delete_zero_tiles(false)?;
        let marker = self.inner.terminal.register_marker(0)?;
        if let Some(marker) = &marker {
            let weak = self.downgrade();
            marker.on_dispose().try_subscribe(move |_| {
                if let Some(storage) = weak.upgrade() {
                    storage
                        .delete_entry(image_id)
                        .map_err(EmitError::callback)?;
                }
                Ok(())
            });
        }
        if self.inner.terminal.active_buffer_type() == ImageBufferType::Alternate {
            self.evict_on_alternate()?;
        }
        let spec = Rc::new(ImageSpec {
            id: image_id,
            original: image.clone(),
            actual: RefCell::new(Some(image)),
            original_cell_size: size,
            actual_cell_size: Cell::new(size),
            marker,
            tile_count: Cell::new(tile_count),
            buffer_type: self.inner.terminal.active_buffer_type(),
            layer: options.layer,
            z_index: options.z_index,
        });
        self.insert_image(image_id, spec)?;
        if let Some(callback) = self.on_image_added() {
            callback()?;
        }
        Ok(image_id)
    }
    pub(super) fn write_to_cell(
        &self,
        line: &mut BufferLine,
        x: usize,
        image_id: u64,
        tile_id: f64,
    ) -> Result<(), StorageError> {
        let mut work = CellData::default();
        if line.load_cell(x, &mut work).is_none() {
            // A linefeed callback may shrink this row after placement captured
            // the old terminal width. Source typed-array stores then ignore the
            // out-of-view words, while the caller still increments tileCount.
            line.set_cell(x, &work);
            return Ok(());
        }
        if work.has_extended_attrs() != 0 {
            if let Some(tile) = work
                .extended
                .payload()
                .and_then(|payload| payload.downcast::<ImageTileInfo>().ok())
            {
                if let Ok(old_id) = u64::try_from(tile.image_id.get()) {
                    if let Some(spec) = self.image_spec(old_id) {
                        spec.tile_count.set(spec.tile_count.get() - 1.0);
                    }
                }
                tile.image_id
                    .set(i64::try_from(image_id).map_err(|_| StorageError::IdentifierOrder)?);
                tile.tile_id.set(tile_id);
                return Ok(());
            }
        }
        let mut extended = work.extended.clone();
        extended.set_payload(Some(Rc::new(ImageTileInfo::new(
            i64::try_from(image_id).map_err(|_| StorageError::IdentifierOrder)?,
            tile_id,
        ))));
        work.extended = extended;
        work.update_extended();
        line.set_cell(x, &work);
        Ok(())
    }
    fn delete_zero_tiles(&self, alternate_only: bool) -> Result<(), StorageError> {
        let mut cursor = None;
        let mut zero = Vec::new();
        while let Some((id, spec)) = self.next_image(&mut cursor) {
            let should_delete = if alternate_only {
                spec.buffer_type == ImageBufferType::Alternate && spec.tile_count.get() == 0.0
            } else {
                spec.tile_count.get() < 1.0
            };
            if should_delete {
                if let Some(marker) = &spec.marker {
                    marker.dispose()?;
                }
                zero.push(id);
            }
        }
        for id in zero {
            self.delete_entry(id)?;
        }
        Ok(())
    }
    fn evict_on_alternate(&self) -> Result<(), StorageError> {
        let mut cursor = None;
        while let Some((_, spec)) = self.next_image(&mut cursor) {
            if spec.buffer_type == ImageBufferType::Alternate {
                spec.tile_count.set(0.0);
            }
        }
        let service = self.inner.terminal.buffer_service();
        let buffer = service.buffer();
        for row in 0..service.rows() {
            let line = buffer
                .try_borrow()
                .map_err(|_| StorageError::BorrowedBuffer)?
                .lines
                .get(row);
            let Some(line) = line else {
                continue;
            };
            for col in 0..service.cols() {
                let tile = {
                    let line = line
                        .try_borrow()
                        .map_err(|_| StorageError::BorrowedBuffer)?;
                    tile_at(&line, col)
                };
                if let Some(tile) = tile {
                    if let Ok(id) = u64::try_from(tile.image_id.get()) {
                        if let Some(spec) = self.image_spec(id) {
                            spec.tile_count.set(spec.tile_count.get() + 1.0);
                        }
                    }
                }
            }
        }
        self.delete_zero_tiles(true)
    }
}
pub(super) fn tile_at(line: &BufferLine, x: usize) -> Option<Rc<ImageTileInfo>> {
    line.get_extended(x)?
        .payload()?
        .downcast::<ImageTileInfo>()
        .ok()
}
fn image_cells(image: &ImageResource, size: CellSize) -> Result<(usize, usize), StorageError> {
    if !size.width.is_finite()
        || !size.height.is_finite()
        || size.width <= 0.0
        || size.height <= 0.0
    {
        return Err(StorageError::InvalidGeometry);
    }
    let cols = (f64::from(image.width()?) / size.width).ceil();
    let rows = (f64::from(image.height()?) / size.height).ceil();
    if !cols.is_finite()
        || !rows.is_finite()
        || cols >= usize::MAX as f64
        || rows >= usize::MAX as f64
    {
        return Err(StorageError::InvalidGeometry);
    }
    // Bounds above make these finite, nonnegative, addressable integer counts.
    Ok((cols as usize, rows as usize))
}
