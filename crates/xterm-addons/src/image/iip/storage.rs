// Copyright (c) 2023 The xterm.js authors. MIT license.
//! IIP's fixed placement policy over the shared image store.
use crate::image::storage::{
    AddImageOptions, CursorPosition, ImageLayer, ImageResource, ImageStorage, StorageError,
};

#[derive(Clone)]
pub struct IipImageStorage {
    storage: ImageStorage,
}
impl IipImageStorage {
    pub fn new(storage: ImageStorage) -> Self {
        Self { storage }
    }
    pub fn add_image(&self, image: ImageResource) -> Result<(), StorageError> {
        self.storage.add_image(
            image,
            AddImageOptions {
                scrolling: true,
                layer: ImageLayer::Top,
                z_index: 0.0,
                cursor_position: CursorPosition::Iip,
            },
        )?;
        Ok(())
    }
}
