// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::StorageError;
use crate::image::types::RgbaImage;
use std::{any::Any, fmt, rc::Rc};

pub type BitmapClose = Rc<dyn Fn() -> Result<(), StorageError>>;

/// Platform pixels or a live canvas/bitmap. Pixel dimensions use the host
/// resource's unsigned contract rather than user-supplied placement numbers.
pub trait ImageSurface: Any {
    fn width(&self) -> Result<u32, StorageError>;
    fn height(&self) -> Result<u32, StorageError>;
    fn bitmap_close(&self) -> Result<Option<BitmapClose>, StorageError>;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Clone)]
pub struct ImageResource(Rc<dyn ImageSurface>);
impl ImageResource {
    pub fn new(surface: impl ImageSurface) -> Self {
        Self(Rc::new(surface))
    }
    pub fn from_pixels(image: RgbaImage) -> Self {
        Self::new(PixelSurface(image))
    }
    pub fn width(&self) -> Result<u32, StorageError> {
        self.0.width()
    }
    pub fn height(&self) -> Result<u32, StorageError> {
        self.0.height()
    }
    pub fn bitmap_close(&self) -> Result<Option<BitmapClose>, StorageError> {
        self.0.bitmap_close()
    }
    pub fn same_identity(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
    pub fn surface<T: Any>(&self) -> Option<&T> {
        self.0.as_any().downcast_ref()
    }
    pub fn pixels(&self) -> Option<&RgbaImage> {
        self.surface::<PixelSurface>().map(|surface| &surface.0)
    }
}
impl fmt::Debug for ImageResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Do not invoke platform getters while formatting an unrelated failure.
        f.debug_struct("ImageResource").finish_non_exhaustive()
    }
}
struct PixelSurface(RgbaImage);
impl ImageSurface for PixelSurface {
    fn width(&self) -> Result<u32, StorageError> {
        Ok(self.0.width())
    }
    fn height(&self) -> Result<u32, StorageError> {
        Ok(self.0.height())
    }
    fn bitmap_close(&self) -> Result<Option<BitmapClose>, StorageError> {
        Ok(None)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
