// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
//! Shared image ownership and buffer tiles. Platform drawing is an explicit host.
mod budget;
mod error;
mod host;
mod native;
mod placement;
mod render;
mod resize;
mod resource;
mod spec;
mod types;
pub use error::StorageError;
pub use host::{StorageDiagnostic, StorageRenderer, StorageTerminal};
pub use native::{MarkerProvider, StorageLogger, TerminalStorageHost};
pub use resource::{BitmapClose, ImageResource, ImageSurface};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
pub use types::{
    AddImageOptions, CellSize, CursorPosition, ImageBufferType, ImageLayer, ImageSpec,
    ImageStorageOptions, ImageTileInfo, CELL_SIZE_DEFAULT,
};

pub type ImageAddedCallback = Rc<dyn Fn() -> Result<(), StorageError>>;
pub type ImageDeletedCallback = Rc<dyn Fn(u64) -> Result<(), StorageError>>;
const JS_ID_LIMIT: u64 = 1 << 53;
const FALLBACK_PIXEL_LIMIT: u32 = 2_500_000;

struct Inner {
    terminal: Rc<dyn StorageTerminal>,
    renderer: Rc<dyn StorageRenderer>,
    images: RefCell<crate::image::number_map::NumberMap<Rc<ImageSpec>>>,
    last_id: Cell<u64>,
    lowest_id: Cell<u64>,
    fully_cleared: Cell<bool>,
    needs_full_clear: Cell<bool>,
    pixel_limit: Cell<u32>,
    viewport_metrics: Cell<(usize, usize)>,
    show_placeholder: Rc<Cell<bool>>,
    added: RefCell<Option<ImageAddedCallback>>,
    deleted: RefCell<Option<ImageDeletedCallback>>,
}

#[derive(Clone)]
pub struct ImageStorage {
    inner: Rc<Inner>,
}
#[derive(Clone)]
pub struct WeakImageStorage {
    inner: Weak<Inner>,
}
impl WeakImageStorage {
    pub fn upgrade(&self) -> Option<ImageStorage> {
        self.inner.upgrade().map(|inner| ImageStorage { inner })
    }
}
impl ImageStorage {
    pub fn new(
        terminal: Rc<dyn StorageTerminal>,
        renderer: Rc<dyn StorageRenderer>,
        options: ImageStorageOptions,
    ) -> Result<Self, StorageError> {
        let service = terminal.buffer_service();
        let storage = Self {
            inner: Rc::new(Inner {
                terminal,
                renderer,
                images: RefCell::new(Default::default()),
                last_id: Cell::new(0),
                lowest_id: Cell::new(0),
                fully_cleared: Cell::new(false),
                needs_full_clear: Cell::new(false),
                pixel_limit: Cell::new(FALLBACK_PIXEL_LIMIT),
                viewport_metrics: Cell::new((service.cols(), service.rows())),
                show_placeholder: options.show_placeholder,
                added: RefCell::new(None),
                deleted: RefCell::new(None),
            }),
        };
        match storage.set_limit(options.storage_limit) {
            Ok(()) => {}
            Err(StorageError::InvalidLimit) => {
                storage.inner.terminal.diagnostic(
                    StorageDiagnostic::Error,
                    &StorageError::InvalidLimit.to_string(),
                )?;
                storage.inner.terminal.diagnostic(
                    StorageDiagnostic::Warning,
                    &format!("storageLimit is set to {} MB", storage.get_limit()),
                )?;
            }
            Err(error) => return Err(error),
        }
        Ok(storage)
    }
    pub fn downgrade(&self) -> WeakImageStorage {
        WeakImageStorage {
            inner: Rc::downgrade(&self.inner),
        }
    }
    pub fn on_image_added(&self) -> Option<ImageAddedCallback> {
        self.inner.added.borrow().clone()
    }
    pub fn set_on_image_added(&self, callback: Option<ImageAddedCallback>) {
        let previous = self.inner.added.replace(callback);
        // Dropping a captured user value may reenter this callback slot.
        drop(previous);
    }
    pub fn on_image_deleted(&self) -> Option<ImageDeletedCallback> {
        self.inner.deleted.borrow().clone()
    }
    pub fn set_on_image_deleted(&self, callback: Option<ImageDeletedCallback>) {
        let previous = self.inner.deleted.replace(callback);
        drop(previous);
    }
    pub fn show_placeholder(&self) -> bool {
        self.inner.show_placeholder.get()
    }
    pub fn set_show_placeholder(&self, value: bool) {
        self.inner.show_placeholder.set(value);
    }
    pub fn image_count(&self) -> usize {
        self.inner.images.borrow().len()
    }
    /// Read-only insertion-order identifiers, including nested placement order.
    pub fn image_ids(&self) -> Vec<u64> {
        let mut cursor = None;
        let mut ids = Vec::new();
        while let Some((id, _)) = self.next_image(&mut cursor) {
            ids.push(id);
        }
        ids
    }
    pub fn image_spec(&self, id: u64) -> Option<Rc<ImageSpec>> {
        self.inner.images.borrow().get(id as f64).cloned()
    }
    pub fn dispose(&self) -> Result<(), StorageError> {
        self.reset()
    }

    pub fn reset(&self) -> Result<(), StorageError> {
        let mut cursor = None;
        while let Some((_, spec)) = self.next_image(&mut cursor) {
            if let Some(marker) = &spec.marker {
                marker.dispose()?;
            }
        }
        // Marker-less alternate images are cleared without explicit bitmap close,
        // matching the source rather than inventing extra close callbacks.
        let removed = self.inner.images.borrow_mut().drain_values();
        drop(removed);
        self.inner.renderer.clear_all(None)
    }
    pub fn delete_image(&self, id: u64) -> Result<(), StorageError> {
        if let Some(spec) = self.image_spec(id) {
            if let Some(marker) = &spec.marker {
                marker.dispose()?;
            }
            self.delete_entry(id)?;
        }
        Ok(())
    }
    pub fn wipe_alternate(&self) -> Result<(), StorageError> {
        let mut cursor = None;
        let mut zero = Vec::new();
        while let Some((id, spec)) = self.next_image(&mut cursor) {
            if spec.buffer_type == ImageBufferType::Alternate {
                if let Some(marker) = &spec.marker {
                    marker.dispose()?;
                }
                zero.push(id);
            }
        }
        for id in zero {
            self.delete_entry(id)?;
        }
        self.inner.needs_full_clear.set(true);
        self.inner.fully_cleared.set(false);
        Ok(())
    }
    // Iteration follows insertion ordinals, not numeric IDs: a nested placement
    // can insert ID 2 before its outer placement inserts ID 1.
    fn next_image(&self, cursor: &mut Option<u64>) -> Option<(u64, Rc<ImageSpec>)> {
        let next = self
            .inner
            .images
            .borrow()
            .after(*cursor)
            .map(|(ordinal, key, spec)| (ordinal, key, Rc::clone(spec)));
        let (ordinal, key, spec) = next?;
        *cursor = Some(ordinal);
        // Only exact generated IDs through 2^53 enter this private map.
        Some((key as u64, spec))
    }
    fn next_eviction(&self) -> Option<(u64, Rc<ImageSpec>)> {
        let map = self.inner.images.borrow();
        let expected = (self.inner.lowest_id.get() + 1).min(JS_ID_LIMIT);
        if let Some(spec) = map.get(expected as f64) {
            return Some((expected, Rc::clone(spec)));
        }
        let mut cursor = None;
        let mut next: Option<(u64, Rc<ImageSpec>)> = None;
        while let Some((ordinal, key, spec)) = map.after(cursor) {
            cursor = Some(ordinal);
            let id = key as u64;
            if id >= expected && next.as_ref().is_none_or(|(previous, _)| id < *previous) {
                next = Some((id, Rc::clone(spec)));
            }
        }
        next
    }
    fn insert_image(&self, id: u64, spec: Rc<ImageSpec>) -> Result<(), StorageError> {
        let previous = self
            .inner
            .images
            .borrow_mut()
            .insert(id as f64, spec)
            .map_err(StorageError::host)?;
        drop(previous);
        Ok(())
    }
    fn delete_entry(&self, id: u64) -> Result<(), StorageError> {
        let spec = self.inner.images.borrow_mut().remove(id as f64);
        if let Some(spec) = spec {
            if let Some(close) = spec.original.bitmap_close()? {
                close()?;
            }
            let callback = self.on_image_deleted();
            if let Some(callback) = callback {
                callback(id)?;
            }
        }
        Ok(())
    }
    fn following_id(&self) -> u64 {
        let id = (self.inner.last_id.get() + 1).min(JS_ID_LIMIT);
        self.inner.last_id.set(id);
        id
    }
}
