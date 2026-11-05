// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{ImageSpec, ImageStorage};
use std::rc::Rc;

/// A live, insertion-ordered traversal of stored images.
///
/// Each step releases the map borrow before returning a spec. Appends and resets
/// between steps are visible until the iterator first reports exhaustion.
pub struct ImageEntries {
    storage: ImageStorage,
    cursor: Option<u64>,
    done: bool,
}
impl ImageStorage {
    pub fn image_entries(&self) -> ImageEntries {
        ImageEntries {
            storage: self.clone(),
            cursor: None,
            done: false,
        }
    }
}
impl Iterator for ImageEntries {
    type Item = (u64, Rc<ImageSpec>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let next = self.storage.next_image(&mut self.cursor);
        self.done = next.is_none();
        next
    }
}
