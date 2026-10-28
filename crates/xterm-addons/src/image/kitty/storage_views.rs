// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Live read-only views of the source Kitty Maps.

use super::super::number_map::NumberMap;
use super::{borrowed, KittyImageData, State};
use crate::image::storage::StorageError;
use std::cell::RefCell;
use std::rc::Rc;

/// Each next call observes current map contents. Appends and clears before
/// exhaustion follow JS Map iteration; an exhausted iterator stays exhausted.
pub struct KittyMapIterator<T> {
    state: Rc<RefCell<State>>,
    select: fn(&State) -> &NumberMap<T>,
    cursor: Option<u64>,
    done: bool,
}
impl<T: Clone> Iterator for KittyMapIterator<T> {
    type Item = Result<(f64, T), StorageError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let state = match self.state.try_borrow() {
            Ok(state) => state,
            Err(error) => return Some(Err(borrowed(error))),
        };
        if let Some((ordinal, id, value)) = (self.select)(&state).after(self.cursor) {
            self.cursor = Some(ordinal);
            Some(Ok((id, value.clone())))
        } else {
            self.done = true;
            None
        }
    }
}

#[derive(Clone)]
pub struct KittyImages {
    pub(super) state: Rc<RefCell<State>>,
}
impl KittyImages {
    pub fn same_identity(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }
    pub fn len(&self) -> Result<usize, StorageError> {
        Ok(self.state.try_borrow().map_err(borrowed)?.images.len())
    }
    pub fn is_empty(&self) -> Result<bool, StorageError> {
        Ok(self.len()? == 0)
    }
    pub fn get(&self, id: f64) -> Result<Option<Rc<KittyImageData>>, StorageError> {
        Ok(self
            .state
            .try_borrow()
            .map_err(borrowed)?
            .images
            .get(id)
            .cloned())
    }
    pub fn iter(&self) -> KittyMapIterator<Rc<KittyImageData>> {
        KittyMapIterator {
            state: Rc::clone(&self.state),
            select: |state| &state.images,
            cursor: None,
            done: false,
        }
    }
    pub fn entries_snapshot(&self) -> Result<Vec<(f64, Rc<KittyImageData>)>, StorageError> {
        self.iter().collect()
    }
}

#[derive(Clone)]
pub struct KittyPlacements {
    pub(super) state: Rc<RefCell<State>>,
}
impl KittyPlacements {
    pub fn same_identity(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }
    pub fn len(&self) -> Result<usize, StorageError> {
        Ok(self.state.try_borrow().map_err(borrowed)?.forward.len())
    }
    pub fn is_empty(&self) -> Result<bool, StorageError> {
        Ok(self.len()? == 0)
    }
    pub fn get(&self, id: f64) -> Result<Option<u64>, StorageError> {
        Ok(self
            .state
            .try_borrow()
            .map_err(borrowed)?
            .forward
            .get(id)
            .copied())
    }
    pub fn iter(&self) -> KittyMapIterator<u64> {
        KittyMapIterator {
            state: Rc::clone(&self.state),
            select: |state| &state.forward,
            cursor: None,
            done: false,
        }
    }
    pub fn entries_snapshot(&self) -> Result<Vec<(f64, u64)>, StorageError> {
        self.iter().collect()
    }
}
