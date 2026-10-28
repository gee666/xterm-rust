// Copyright (c) 2022 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from common/services/OscLinkService.ts.

//! OSC8 link data remains shared and mutable, as in the JavaScript source.

use crate::buffer::{marker::Marker, BufferService};
use crate::event::EmitError;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// Lossless JavaScript strings. `None` id differs from a present empty id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OscLinkData {
    pub id: Option<Vec<u16>>,
    pub uri: Vec<u16>,
}

pub type LinkDataRef = Rc<RefCell<OscLinkData>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OscLinkError {
    BorrowedBuffer,
    BorrowedData,
    LineOverflow,
}

impl fmt::Display for OscLinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BorrowedBuffer => "OSC link buffer is already mutably borrowed",
            Self::BorrowedData => "OSC link data is already mutably borrowed",
            Self::LineOverflow => "OSC link line exceeds signed coordinate range",
        })
    }
}
impl std::error::Error for OscLinkError {}

struct Entry {
    id: u64,
    key: Option<Vec<u16>>,
    data: LinkDataRef,
    lines: RefCell<Vec<Marker>>,
}

struct State {
    next_id: Cell<u64>,
    named: RefCell<HashMap<Vec<u16>, Rc<Entry>>>,
    by_id: RefCell<HashMap<u64, Rc<Entry>>>,
    overwritten: RefCell<Vec<Rc<Entry>>>,
}

/// Shared link store with a live active-buffer provider.
/// Marker callbacks hold weak references, not ownership cycles.
#[derive(Clone)]
pub struct OscLinkService {
    buffers: Rc<BufferService>,
    state: Rc<State>,
}

impl OscLinkService {
    pub fn new(buffers: Rc<BufferService>) -> Self {
        Self {
            buffers,
            state: Rc::new(State {
                next_id: Cell::new(1),
                named: RefCell::new(HashMap::new()),
                by_id: RefCell::new(HashMap::new()),
                overwritten: RefCell::new(Vec::new()),
            }),
        }
    }

    pub fn register_link(&self, data: LinkDataRef) -> Result<u64, OscLinkError> {
        let buffer = self.buffers.buffer();
        let line = {
            let buffer = buffer
                .try_borrow()
                .map_err(|_| OscLinkError::BorrowedBuffer)?;
            let line = buffer
                .ybase
                .checked_add(buffer.y)
                .ok_or(OscLinkError::LineOverflow)?;
            i64::try_from(line).map_err(|_| OscLinkError::LineOverflow)?
        };
        let key = {
            let data = data.try_borrow().map_err(|_| OscLinkError::BorrowedData)?;
            data.id.as_ref().map(|id| {
                // Deliberately do not escape separators: upstream keys collide.
                let mut key = id.clone();
                key.extend([u16::from(b';'); 2]);
                key.extend(&data.uri);
                key
            })
        };
        if let Some(key) = &key {
            let existing = self.state.named.borrow().get(key).cloned();
            if let Some(existing) = existing {
                self.add_line_to_link(existing.id, line)?;
                return Ok(existing.id);
            }
        }
        let marker = buffer
            .try_borrow_mut()
            .map_err(|_| OscLinkError::BorrowedBuffer)?
            .add_marker(line);
        let id = self.state.next_id.get();
        self.state.next_id.set(following_id(id));
        let entry = Rc::new(Entry {
            id,
            key,
            data,
            lines: RefCell::new(vec![marker.clone()]),
        });
        self.track_marker(&entry, &marker);
        if let Some(key) = &entry.key {
            self.state
                .named
                .borrow_mut()
                .insert(key.clone(), Rc::clone(&entry));
        }
        let overwritten = self.state.by_id.borrow_mut().insert(id, entry);
        if let Some(overwritten) = overwritten {
            // At the JS numeric limit, an older marker must still be able to
            // remove the newer id entry. Retain it without a callback cycle.
            self.state.overwritten.borrow_mut().push(overwritten);
        }
        Ok(id)
    }

    /// Deduplicate by current marker line only, even across active buffers.
    pub fn add_line_to_link(&self, link_id: u64, line: i64) -> Result<(), OscLinkError> {
        let entry = self.state.by_id.borrow().get(&link_id).cloned();
        let Some(entry) = entry else {
            return Ok(());
        };
        if entry
            .lines
            .borrow()
            .iter()
            .any(|marker| marker.line_value().strict_eq_number(line as f64))
        {
            return Ok(());
        }
        let marker = self
            .buffers
            .buffer()
            .try_borrow_mut()
            .map_err(|_| OscLinkError::BorrowedBuffer)?
            .add_marker(line);
        entry.lines.borrow_mut().push(marker.clone());
        self.track_marker(&entry, &marker);
        Ok(())
    }

    /// Return the original data identity, not a snapshot or a recomputed key.
    pub fn get_link_data(&self, link_id: u64) -> Option<LinkDataRef> {
        self.state
            .by_id
            .borrow()
            .get(&link_id)
            .map(|entry| Rc::clone(&entry.data))
    }

    /// Deterministic diagnostic inventory for complete-state differential checks.
    /// This Rust-only snapshot has no side effects and does not expose entries.
    pub fn link_ids(&self) -> Vec<u64> {
        let mut ids: Vec<_> = self.state.by_id.borrow().keys().copied().collect();
        ids.sort_unstable();
        ids
    }

    fn track_marker(&self, entry: &Rc<Entry>, marker: &Marker) {
        let state = Rc::downgrade(&self.state);
        let entry = Rc::downgrade(entry);
        let marker_weak = marker.downgrade();
        marker.on_dispose().try_subscribe(move |_| {
            if let (Some(state), Some(entry), Some(marker)) =
                (state.upgrade(), entry.upgrade(), marker_weak.upgrade())
            {
                let empty = {
                    let mut lines = entry.lines.borrow_mut();
                    let Some(index) = lines.iter().position(|line| line.ptr_eq(&marker)) else {
                        return Ok(());
                    };
                    lines.remove(index);
                    lines.is_empty()
                };
                if empty {
                    // Removal checks the current data.id, not the captured key.
                    // Mutating it to absent intentionally leaves a stale named entry.
                    let named = entry
                        .data
                        .try_borrow()
                        .map_err(|_| EmitError::callback(OscLinkError::BorrowedData))?
                        .id
                        .is_some();
                    if named {
                        if let Some(key) = &entry.key {
                            state.named.borrow_mut().remove(key);
                        }
                    }
                    let removed = state.by_id.borrow_mut().remove(&entry.id);
                    if let Some(removed) = removed {
                        if !Rc::ptr_eq(&removed, &entry) && !removed.lines.borrow().is_empty() {
                            // A stale saturated-id callback can unlink another
                            // live entry. Its later callback must remain active.
                            state.overwritten.borrow_mut().push(removed);
                        }
                    }
                    state
                        .overwritten
                        .borrow_mut()
                        .retain(|old| !Rc::ptr_eq(old, &entry));
                }
            }
            Ok(())
        });
    }
}

fn following_id(id: u64) -> u64 {
    (id + 1).min(1 << 53)
}

#[cfg(test)]
mod tests {
    use super::{following_id, OscLinkData, OscLinkService};
    use crate::{buffer::BufferService, options::OptionsService};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn ids_follow_javascript_safe_integer_boundary() {
        assert_eq!(following_id(1), 2);
        assert_eq!(following_id((1 << 53) - 1), 1 << 53);
        assert_eq!(following_id(1 << 53), 1 << 53);
    }

    #[test]
    fn overwritten_limit_entry_still_removes_the_current_numeric_id() {
        let buffers = Rc::new(
            BufferService::new(Rc::new(OptionsService::new(
                false,
                std::iter::empty::<(&str, crate::options::OptionValue)>(),
            )))
            .unwrap(),
        );
        let links = OscLinkService::new(Rc::clone(&buffers));
        links.state.next_id.set(1 << 53);
        let data = || {
            Rc::new(RefCell::new(OscLinkData {
                id: None,
                uri: Vec::new(),
            }))
        };
        let first = data();
        let second = data();
        let id = links.register_link(first).unwrap();
        assert_eq!(links.register_link(Rc::clone(&second)), Ok(id));
        assert!(Rc::ptr_eq(&links.get_link_data(id).unwrap(), &second));
        let old_marker = buffers.buffer().borrow().markers.borrow()[0].clone();
        let second_marker = buffers.buffer().borrow().markers.borrow()[1].clone();
        old_marker.dispose().unwrap();
        assert!(links.get_link_data(id).is_none());
        assert_eq!(links.state.overwritten.borrow().len(), 1);
        let third = data();
        assert_eq!(links.register_link(Rc::clone(&third)), Ok(id));
        assert!(Rc::ptr_eq(&links.get_link_data(id).unwrap(), &third));
        let third_marker = buffers.buffer().borrow().markers.borrow()[1].clone();
        second_marker.dispose().unwrap();
        assert!(links.get_link_data(id).is_none());
        third_marker.dispose().unwrap();
        assert!(links.state.overwritten.borrow().is_empty());
    }
}
