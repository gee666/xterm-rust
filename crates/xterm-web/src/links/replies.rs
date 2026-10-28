// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{Link, Position};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};
#[path = "replies_snapshot.rs"]
mod snapshot;

pub type ReplyGroup<T> = Rc<RefCell<Vec<Rc<T>>>>;
struct Replies<T> {
    entries: BTreeMap<usize, Option<ReplyGroup<T>>>,
    order: Vec<usize>,
}
impl<T> Default for Replies<T> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            order: Vec::new(),
        }
    }
}
/// Payloads remain shared handles. Browser callers can retain original JS link
/// objects instead of freezing their fields. No table/group borrow spans a probe.
pub struct ProviderReplies<T = Link> {
    state: RefCell<Replies<T>>,
}
impl<T> Default for ProviderReplies<T> {
    fn default() -> Self {
        Self {
            state: RefCell::new(Replies::default()),
        }
    }
}
/// A request captures its position and the outer callback's shared selection flag.
/// Older requests deliberately remain usable after the active row changes.
#[derive(Clone)]
pub struct ProviderRequest {
    pub position: Position,
    link_provided: Rc<Cell<bool>>,
}
impl ProviderRequest {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            link_provided: Rc::new(Cell::new(false)),
        }
    }
}
impl<T> ProviderReplies<T> {
    pub fn len(&self) -> usize {
        self.state.borrow().entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.state.borrow().entries.is_empty()
    }
    pub fn group(&self, index: usize) -> Option<Option<ReplyGroup<T>>> {
        self.state.borrow().entries.get(&index).cloned()
    }
    /// Map insertion order, including undefined replies; replacing a key does
    /// not move it. Retain the returned group during callback-driven traversal.
    pub fn disposal_group(&self, ordinal: usize) -> Option<Option<ReplyGroup<T>>> {
        let state = self.state.borrow();
        let index = *state.order.get(ordinal)?;
        state.entries.get(&index).cloned()
    }
    /// Owned array snapshot of shared identities, not cloned link data.
    pub fn links(&self, index: usize) -> Option<Vec<Rc<T>>> {
        self.group(index)?.map(|group| {
            let links = group.borrow().clone();
            links
        })
    }
    pub fn cached_links(&self) -> Vec<Rc<T>> {
        let mut result = Vec::new();
        let mut ordinal = 0;
        while let Some(group) = self.disposal_group(ordinal) {
            if let Some(group) = group {
                result.extend(group.borrow().iter().cloned());
            }
            ordinal += 1;
        }
        result
    }
    pub fn reset(&self) -> Vec<Rc<T>> {
        let links = self.cached_links();
        let old = std::mem::take(&mut *self.state.borrow_mut());
        drop(old);
        links
    }
    pub fn try_receive_unfiltered<E>(
        &self,
        index: usize,
        links: Option<Vec<Rc<T>>>,
        request: &ProviderRequest,
        count: usize,
        mut probe: impl FnMut(Option<&Rc<T>>) -> Result<bool, E>,
    ) -> Result<Option<Rc<T>>, E> {
        let old = {
            let mut state = self.state.borrow_mut();
            if !state.entries.contains_key(&index) {
                state.order.push(index);
            }
            state
                .entries
                .insert(index, links.map(|links| Rc::new(RefCell::new(links))))
        };
        drop(old);
        self.check(index, request, count, &mut probe)
    }
    pub fn try_cached_index<E>(
        &self,
        index: usize,
        request: &ProviderRequest,
        count: usize,
        mut probe: impl FnMut(Option<&Rc<T>>) -> Result<bool, E>,
    ) -> Result<Option<Rc<T>>, E> {
        if !matches!(self.group(index), Some(Some(_))) {
            return Ok(None);
        }
        self.check(index, request, count, &mut probe)
    }
    fn check<E>(
        &self,
        index: usize,
        request: &ProviderRequest,
        count: usize,
        probe: &mut impl FnMut(Option<&Rc<T>>) -> Result<bool, E>,
    ) -> Result<Option<Rc<T>>, E> {
        // Source passes linkProvided by value to this method. A reentrant reply
        // can change the outer flag, but normal return writes this local copy back.
        let provided = request.link_provided.get();
        let links = self.group(index).flatten();
        let before = (0..index).any(|prior| !matches!(self.group(prior), Some(None)));
        if !before {
            if let Some(link) = self.find(links, probe)? {
                request.link_provided.set(true);
                return Ok(Some(link));
            }
        }
        if self.len() == count && !provided {
            let mut current = 0;
            while current < self.len() {
                if let Some(link) = self.find(self.group(current).flatten(), probe)? {
                    request.link_provided.set(true);
                    return Ok(Some(link));
                }
                current += 1;
            }
        }
        request.link_provided.set(provided);
        Ok(None)
    }
    fn find<E>(
        &self,
        group: Option<ReplyGroup<T>>,
        probe: &mut impl FnMut(Option<&Rc<T>>) -> Result<bool, E>,
    ) -> Result<Option<Rc<T>>, E> {
        let Some(group) = group else { return Ok(None) };
        // Array.find captures length, but reads each slot when reached. The probe
        // handles a missing slot so a browser adapter can reproduce its TypeError.
        let length = group.borrow().len();
        for index in 0..length {
            let link = group.borrow().get(index).cloned();
            if probe(link.as_ref())? {
                return Ok(link);
            }
        }
        Ok(None)
    }
    pub fn try_filter_complete<E>(
        &self,
        count: usize,
        mut keep: impl FnMut(&Rc<T>) -> Result<bool, E>,
    ) -> Result<(), E> {
        if self.len() != count {
            return Ok(());
        }
        let mut provider = 0;
        while provider < self.len() {
            if let Some(group) = self.group(provider).flatten() {
                let mut index = 0;
                loop {
                    let link = group.borrow().get(index).cloned();
                    let Some(link) = link else { break };
                    if keep(&link)? {
                        index += 1;
                    } else {
                        // User getters can mutate this same array. JS splice at
                        // an index beyond its new length is a valid no-op.
                        let removed = {
                            let mut links = group.borrow_mut();
                            (index < links.len()).then(|| links.remove(index))
                        };
                        drop(removed);
                    }
                }
            }
            provider += 1;
        }
        Ok(())
    }
}
