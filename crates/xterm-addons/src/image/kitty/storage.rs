// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-image/src/kitty/KittyImageStorage.ts.

use super::number_map::NumberMap;
use crate::image::encoded::EncodedBytes;
use crate::image::storage::{
    AddImageOptions, CursorPosition, ImageLayer, ImageResource, ImageStorage, StorageError,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[path = "storage_views.rs"]
mod views;
pub use views::{KittyImages, KittyMapIterator, KittyPlacements};

const MAX_STORED_IMAGES: usize = 256;
pub type DeletedCallback = Rc<dyn Fn(u64) -> Result<(), StorageError>>;

/// Actual placement storage, not a renderer success placeholder. Methods must
/// release internal borrows before callbacks; Kitty also releases its map borrows.
pub trait KittyPlacementStorage {
    fn add_image(
        &self,
        image: ImageResource,
        options: AddImageOptions,
    ) -> Result<u64, StorageError>;
    fn delete_image(&self, storage_id: u64) -> Result<(), StorageError>;
    fn on_deleted(&self) -> Option<DeletedCallback>;
    fn set_on_deleted(&self, callback: Option<DeletedCallback>);
}

impl KittyPlacementStorage for ImageStorage {
    fn add_image(
        &self,
        image: ImageResource,
        options: AddImageOptions,
    ) -> Result<u64, StorageError> {
        ImageStorage::add_image(self, image, options)
    }
    fn delete_image(&self, storage_id: u64) -> Result<(), StorageError> {
        ImageStorage::delete_image(self, storage_id)
    }
    fn on_deleted(&self) -> Option<DeletedCallback> {
        self.on_image_deleted()
    }
    fn set_on_deleted(&self, callback: Option<DeletedCallback>) {
        self.set_on_image_deleted(callback);
    }
}

/// Copied transmitted data remains distinct from any decoded/displayed surface.
#[derive(Clone, Debug)]
pub struct KittyImageBytes {
    pub data: EncodedBytes,
    pub width: f64,
    pub height: f64,
    pub format: f64,
    pub compression: Vec<u16>,
}
#[derive(Clone, Debug)]
pub struct KittyImageData {
    pub id: f64,
    pub bytes: KittyImageBytes,
}

#[derive(Debug)]
struct BorrowedKittyState;
impl fmt::Display for BorrowedKittyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Kitty image state is already borrowed")
    }
}
impl std::error::Error for BorrowedKittyState {}
fn borrowed(_: impl fmt::Debug) -> StorageError {
    StorageError::host(BorrowedKittyState)
}

struct State {
    next_image_id: f64,
    images: NumberMap<Rc<KittyImageData>>,
    forward: NumberMap<u64>,
    reverse: HashMap<u64, f64>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            next_image_id: 1.0,
            images: NumberMap::default(),
            forward: NumberMap::default(),
            reverse: HashMap::new(),
        }
    }
}
impl State {
    fn evict_undisplayed(&mut self) {
        let mut cursor = None;
        while self.images.len() > MAX_STORED_IMAGES / 2 {
            let Some((ordinal, id, _)) = self.images.after(cursor) else {
                break;
            };
            cursor = Some(ordinal);
            if !self.forward.contains(id) {
                self.images.remove(id);
            }
        }
    }
    fn deleted(&mut self, storage_id: u64) {
        if let Some(id) = self.reverse.get(&storage_id).copied() {
            self.forward.remove(id);
            self.reverse.remove(&storage_id);
            self.images.remove(id);
        }
    }
}

pub struct KittyImageStorage {
    host: Rc<dyn KittyPlacementStorage>,
    state: Rc<RefCell<State>>,
    previous: Option<DeletedCallback>,
    wrapped: DeletedCallback,
}
impl KittyImageStorage {
    pub fn new(host: Rc<dyn KittyPlacementStorage>) -> Self {
        let state = Rc::new(RefCell::new(State::default()));
        let weak = Rc::downgrade(&state);
        let previous = host.on_deleted();
        let before = previous.clone();
        let wrapped: DeletedCallback = Rc::new(move |storage_id| {
            if let Some(before) = &before {
                before(storage_id)?;
            }
            if let Some(state) = weak.upgrade() {
                state
                    .try_borrow_mut()
                    .map_err(borrowed)?
                    .deleted(storage_id);
            }
            Ok(())
        });
        host.set_on_deleted(Some(Rc::clone(&wrapped)));
        Self {
            host,
            state,
            previous,
            wrapped,
        }
    }

    pub fn reset(&self) -> Result<(), StorageError> {
        let mut state = self.state.try_borrow_mut().map_err(borrowed)?;
        state.next_image_id = 1.0;
        state.images.clear();
        state.forward.clear();
        state.reverse.clear();
        Ok(())
    }
    fn restore_callback(&self) {
        if self
            .host
            .on_deleted()
            .is_some_and(|current| Rc::ptr_eq(&current, &self.wrapped))
        {
            self.host.set_on_deleted(self.previous.clone());
        }
    }
    pub fn dispose(&self) -> Result<(), StorageError> {
        self.reset()?;
        self.restore_callback();
        Ok(())
    }

    pub fn store_image(
        &self,
        id: Option<f64>,
        bytes: KittyImageBytes,
    ) -> Result<f64, StorageError> {
        let (id, old_storage) = {
            let mut state = self.state.try_borrow_mut().map_err(borrowed)?;
            let id = id.unwrap_or_else(|| {
                let id = state.next_image_id;
                state.next_image_id += 1.0;
                id
            });
            (id, state.forward.get(id).copied())
        };
        if let Some(old_storage) = old_storage {
            self.host.delete_image(old_storage)?;
            let mut state = self.state.try_borrow_mut().map_err(borrowed)?;
            state.forward.remove(id);
            state.reverse.remove(&old_storage);
        }
        let mut state = self.state.try_borrow_mut().map_err(borrowed)?;
        if !state.images.contains(id) && state.images.len() >= MAX_STORED_IMAGES {
            state.evict_undisplayed();
        }
        state
            .images
            .set(id, Rc::new(KittyImageData { id, bytes }))
            .map_err(StorageError::host)?;
        Ok(id)
    }

    pub fn add_image(
        &self,
        id: f64,
        image: ImageResource,
        scrolling: bool,
        layer: ImageLayer,
        z_index: f64,
    ) -> Result<u64, StorageError> {
        {
            let mut state = self.state.try_borrow_mut().map_err(borrowed)?;
            if let Some(old_storage) = state.forward.get(id).copied() {
                state.reverse.remove(&old_storage);
            }
        }
        let storage_id = self.host.add_image(
            image,
            AddImageOptions {
                scrolling,
                layer,
                z_index,
                cursor_position: CursorPosition::Iip,
            },
        )?;
        let mut state = self.state.try_borrow_mut().map_err(borrowed)?;
        state
            .forward
            .set(id, storage_id)
            .map_err(StorageError::host)?;
        state.reverse.insert(storage_id, id);
        Ok(storage_id)
    }

    pub fn get_image(&self, id: f64) -> Result<Option<Rc<KittyImageData>>, StorageError> {
        Ok(self
            .state
            .try_borrow()
            .map_err(borrowed)?
            .images
            .get(id)
            .cloned())
    }
    pub fn delete_by_id(&self, id: f64) -> Result<(), StorageError> {
        let storage_id = {
            let mut state = self.state.try_borrow_mut().map_err(borrowed)?;
            state.images.remove(id);
            state.forward.get(id).copied()
        };
        if let Some(storage_id) = storage_id {
            self.host.delete_image(storage_id)?;
            let mut state = self.state.try_borrow_mut().map_err(borrowed)?;
            state.forward.remove(id);
            state.reverse.remove(&storage_id);
        }
        Ok(())
    }
    pub fn delete_all(&self) -> Result<(), StorageError> {
        self.state
            .try_borrow_mut()
            .map_err(borrowed)?
            .images
            .clear();
        let mut cursor = None;
        loop {
            let next = self
                .state
                .try_borrow()
                .map_err(borrowed)?
                .forward
                .after(cursor)
                .map(|(ordinal, _, value)| (ordinal, *value));
            let Some((ordinal, storage_id)) = next else {
                break;
            };
            cursor = Some(ordinal);
            self.host.delete_image(storage_id)?;
        }
        let mut state = self.state.try_borrow_mut().map_err(borrowed)?;
        state.forward.clear();
        state.reverse.clear();
        Ok(())
    }
    pub fn images(&self) -> KittyImages {
        KittyImages {
            state: Rc::clone(&self.state),
        }
    }
    pub fn placements(&self) -> KittyPlacements {
        KittyPlacements {
            state: Rc::clone(&self.state),
        }
    }
    pub fn last_image_id(&self) -> Result<f64, StorageError> {
        Ok(self.state.try_borrow().map_err(borrowed)?.next_image_id - 1.0)
    }
}
impl Drop for KittyImageStorage {
    fn drop(&mut self) {
        self.restore_callback();
    }
}
