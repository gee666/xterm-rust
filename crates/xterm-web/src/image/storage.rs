// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{js::error, BrowserImageRenderer};
use xterm_addons::image::storage::{
    CellSize, ImageLayer, ImageResource, ImageSpec, StorageError, StorageRenderer,
};

impl StorageRenderer for BrowserImageRenderer {
    fn cell_size(&self) -> Result<CellSize, StorageError> {
        self.cell_size().map_err(error)
    }
    fn has_layer(&self, layer: ImageLayer) -> bool {
        self.has_layer(layer)
    }
    fn insert_layer(&self, layer: ImageLayer) -> Result<(), StorageError> {
        self.insert_layer(layer).map_err(error)
    }
    fn remove_layer(&self, layer: ImageLayer) -> Result<(), StorageError> {
        self.remove_layer(layer).map_err(error)
    }
    fn rescale_canvas(&self) -> Result<(), StorageError> {
        self.rescale_canvas().map_err(error)
    }
    fn clear_all(&self, layer: Option<ImageLayer>) -> Result<(), StorageError> {
        self.clear_all(layer).map_err(error)
    }
    fn clear_lines(&self, start: usize, end: usize) -> Result<(), StorageError> {
        self.clear_lines(start as f64, end as f64, None)
            .map_err(error)
    }
    fn draw_placeholder(&self, col: usize, row: usize, count: usize) -> Result<(), StorageError> {
        self.draw_placeholder(col as f64, row as f64, count as f64)
            .map_err(error)
    }
    fn draw(
        &self,
        spec: &ImageSpec,
        tile_id: f64,
        col: usize,
        row: usize,
        count: usize,
    ) -> Result<(), StorageError> {
        self.draw(spec, tile_id, col as f64, row as f64, count as f64)
            .map_err(error)
    }
    fn extract_tile(
        &self,
        spec: &ImageSpec,
        tile_id: f64,
    ) -> Result<Option<ImageResource>, StorageError> {
        self.extract_tile(spec, tile_id).map_err(error)
    }
    fn bitmap_to_canvas(&self, image: &ImageResource) -> Result<ImageResource, StorageError> {
        self.bitmap_to_canvas(image).map_err(error)
    }
}
