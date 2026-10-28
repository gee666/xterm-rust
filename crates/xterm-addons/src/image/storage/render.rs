// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{placement::tile_at, ImageLayer, ImageResource, ImageSpec, ImageStorage, StorageError};
use std::{cmp::Ordering, rc::Rc};
use xterm_core::sorted_list::stable_sort_by_js_number;

struct Draw {
    spec: Rc<ImageSpec>,
    tile_id: f64,
    col: usize,
    row: usize,
    count: usize,
}
impl ImageStorage {
    pub fn render(&self, start: usize, end: usize) -> Result<(), StorageError> {
        let (mut top, mut bottom) = (false, false);
        let mut cursor = None;
        while let Some((_, spec)) = self.next_image(&mut cursor) {
            match spec.layer {
                ImageLayer::Top => top = true,
                ImageLayer::Bottom => bottom = true,
            }
            if top && bottom {
                break;
            }
        }
        let renderer = &self.inner.renderer;
        if top && !renderer.has_layer(ImageLayer::Top) {
            renderer.insert_layer(ImageLayer::Top)?;
            if !renderer.has_layer(ImageLayer::Top) {
                return Ok(());
            }
        }
        if bottom && !renderer.has_layer(ImageLayer::Bottom) {
            renderer.insert_layer(ImageLayer::Bottom)?;
        }
        renderer.rescale_canvas()?;
        if self.image_count() == 0 {
            if !self.inner.fully_cleared.get() {
                renderer.clear_all(None)?;
                self.inner.fully_cleared.set(true);
                self.inner.needs_full_clear.set(false);
            }
            for layer in [ImageLayer::Top, ImageLayer::Bottom] {
                if renderer.has_layer(layer) {
                    renderer.remove_layer(layer)?;
                }
            }
            return Ok(());
        }
        for (needed, layer) in [(top, ImageLayer::Top), (bottom, ImageLayer::Bottom)] {
            if !needed && renderer.has_layer(layer) {
                renderer.clear_all(Some(layer))?;
                renderer.remove_layer(layer)?;
            }
        }
        if self.inner.needs_full_clear.get() {
            renderer.clear_all(None)?;
            self.inner.fully_cleared.set(true);
            self.inner.needs_full_clear.set(false);
        }
        let service = self.inner.terminal.buffer_service();
        let buffer = service.buffer();
        let cols = service.cols();
        renderer.clear_lines(start, end)?;
        let mut draws = Vec::new();
        let mut placeholders = Vec::new();
        if start <= end {
            for row in start..=end {
                let line = {
                    let buffer = buffer
                        .try_borrow()
                        .map_err(|_| StorageError::BorrowedBuffer)?;
                    let index = buffer
                        .ydisp
                        .checked_add(row)
                        .ok_or(StorageError::InvalidGeometry)?;
                    buffer.lines.get(index)
                };
                // A missing row returns after clearLines, discarding pending
                // draw calls exactly like the source's early return.
                let Some(line) = line else {
                    return Ok(());
                };
                let mut col = 0;
                while col < cols {
                    let tile = {
                        let line = line
                            .try_borrow()
                            .map_err(|_| StorageError::BorrowedBuffer)?;
                        tile_at(&line, col)
                    };
                    if let Some(tile) =
                        tile.filter(|tile| tile.image_id.get() != -1 && tile.tile_id.get() != -1.0)
                    {
                        let id = tile.image_id.get();
                        let first_tile = tile.tile_id.get();
                        let first_col = col;
                        let mut count = 1;
                        while first_col + count < cols {
                            let next = {
                                let line = line
                                    .try_borrow()
                                    .map_err(|_| StorageError::BorrowedBuffer)?;
                                tile_at(&line, first_col + count)
                            };
                            if !next.is_some_and(|next| {
                                next.image_id.get() == id
                                    && next.tile_id.get() == first_tile + count as f64
                            }) {
                                break;
                            }
                            count += 1;
                        }
                        let spec = u64::try_from(id).ok().and_then(|id| self.image_spec(id));
                        if let Some(spec) = spec {
                            if spec.actual.borrow().is_some() {
                                draws.push(Draw {
                                    spec,
                                    tile_id: first_tile,
                                    col: first_col,
                                    row,
                                    count,
                                });
                            }
                        } else if self.show_placeholder() {
                            placeholders.push((first_col, row, count));
                        }
                        self.inner.fully_cleared.set(false);
                        col += count;
                    } else {
                        col += 1;
                    }
                }
            }
        }
        if draws.iter().any(|draw| draw.spec.z_index.is_nan()) {
            stable_sort_by_js_number(&mut draws, &|draw: &Draw| draw.spec.z_index);
        } else {
            draws.sort_by(|a, b| {
                (a.spec.z_index - b.spec.z_index)
                    .partial_cmp(&0.0)
                    .unwrap_or(Ordering::Equal)
            });
        }
        for (col, row, count) in placeholders {
            renderer.draw_placeholder(col, row, count)?;
        }
        for draw in draws {
            renderer.draw(&draw.spec, draw.tile_id, draw.col, draw.row, draw.count)?;
        }
        Ok(())
    }
    pub fn get_image_at_buffer_cell(
        &self,
        x: usize,
        y: usize,
    ) -> Result<Option<ImageResource>, StorageError> {
        let Some((spec, _)) = self.spec_at(x, y)? else {
            return Ok(None);
        };
        if spec.original.bitmap_close()?.is_some() {
            return self
                .inner
                .renderer
                .bitmap_to_canvas(&spec.original)
                .map(Some);
        }
        Ok(Some(spec.original.clone()))
    }
    pub fn extract_tile_at_buffer_cell(
        &self,
        x: usize,
        y: usize,
    ) -> Result<Option<ImageResource>, StorageError> {
        let Some((spec, tile)) = self.spec_at(x, y)? else {
            return Ok(None);
        };
        if tile == -1.0 {
            return Ok(None);
        }
        self.inner.renderer.extract_tile(&spec, tile)
    }
    /// Number boundary for bindings, retaining signed/cyclic row lookup.
    pub fn get_image_at_buffer_cell_number(
        &self,
        x: f64,
        y: f64,
    ) -> Result<Option<ImageResource>, StorageError> {
        let Some((spec, _)) = self.spec_at_number(x, y)? else {
            return Ok(None);
        };
        if spec.original.bitmap_close()?.is_some() {
            return self
                .inner
                .renderer
                .bitmap_to_canvas(&spec.original)
                .map(Some);
        }
        Ok(Some(spec.original.clone()))
    }
    pub fn extract_tile_at_buffer_cell_number(
        &self,
        x: f64,
        y: f64,
    ) -> Result<Option<ImageResource>, StorageError> {
        let Some((spec, tile)) = self.spec_at_number(x, y)? else {
            return Ok(None);
        };
        if tile == -1.0 {
            return Ok(None);
        }
        self.inner.renderer.extract_tile(&spec, tile)
    }
    fn spec_at_number(&self, x: f64, y: f64) -> Result<Option<(Rc<ImageSpec>, f64)>, StorageError> {
        let buffer = self.inner.terminal.buffer_service().buffer();
        let line = buffer
            .try_borrow()
            .map_err(|_| StorageError::BorrowedBuffer)?
            .lines
            .get_number(y);
        let Some(line) = line else {
            return Ok(None);
        };
        // Storage creates payloads at integer cell addresses only. Other
        // extended-property keys have no ImageTileInfo assigned by this addon.
        if !x.is_finite() || x < 0.0 || x.fract() != 0.0 || x >= usize::MAX as f64 {
            return Ok(None);
        }
        self.spec_on_line(&line, x as usize)
    }
    fn spec_at(&self, x: usize, y: usize) -> Result<Option<(Rc<ImageSpec>, f64)>, StorageError> {
        let buffer = self.inner.terminal.buffer_service().buffer();
        let line = buffer
            .try_borrow()
            .map_err(|_| StorageError::BorrowedBuffer)?
            .lines
            .get(y);
        let Some(line) = line else {
            return Ok(None);
        };
        self.spec_on_line(&line, x)
    }
    fn spec_on_line(
        &self,
        line: &xterm_core::buffer::LineRef,
        x: usize,
    ) -> Result<Option<(Rc<ImageSpec>, f64)>, StorageError> {
        let tile = {
            let line = line
                .try_borrow()
                .map_err(|_| StorageError::BorrowedBuffer)?;
            tile_at(&line, x)
        };
        let Some(tile) = tile.filter(|tile| tile.image_id.get() != 0 && tile.image_id.get() != -1)
        else {
            return Ok(None);
        };
        Ok(u64::try_from(tile.image_id.get())
            .ok()
            .and_then(|id| self.image_spec(id))
            .map(|spec| (spec, tile.tile_id.get())))
    }
}
