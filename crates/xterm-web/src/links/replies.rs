// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{Link, Position};
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

/// A reply is absent while pending; `None` means the callback returned undefined.
/// `Some([])` remains a defined, truthy reply, unlike an empty Rust collection test.
#[derive(Default)]
pub struct ProviderReplies {
    replies: BTreeMap<usize, Option<Vec<Rc<Link>>>>,
}

/// Each provider lookup captures its original position and a shared selection flag.
/// Late replies deliberately use that request, even after another row became active,
/// because the upstream callbacks do not carry a cancellation generation.
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

impl ProviderReplies {
    /// Extract every retained link for host disposal before starting a new cache.
    pub fn clear(&mut self) -> Vec<Rc<Link>> {
        std::mem::take(&mut self.replies)
            .into_values()
            .flatten()
            .flatten()
            .collect()
    }

    pub fn receive(
        &mut self,
        index: usize,
        links: Option<Vec<Rc<Link>>>,
        request: &ProviderRequest,
        provider_count: usize,
        cols: usize,
    ) -> Option<Rc<Link>> {
        let result = self.receive_unfiltered(index, links, request, provider_count, cols);
        self.filter_complete(request.position.y, provider_count, cols);
        result
    }

    /// Browser hosts dispatch hover callbacks before overlap filtering.
    pub fn receive_unfiltered(
        &mut self,
        index: usize,
        links: Option<Vec<Rc<Link>>>,
        request: &ProviderRequest,
        provider_count: usize,
        cols: usize,
    ) -> Option<Rc<Link>> {
        self.replies.insert(index, links);
        self.check(index, request, provider_count, cols)
    }
    pub fn filter_complete(&mut self, y: usize, provider_count: usize, cols: usize) {
        if self.replies.len() == provider_count {
            self.remove_intersections(y, cols);
        }
    }
    pub fn from_cache_index(
        &self,
        index: usize,
        request: &ProviderRequest,
        provider_count: usize,
        cols: usize,
    ) -> Option<Rc<Link>> {
        if !matches!(self.replies.get(&index), Some(Some(_))) {
            return None;
        }
        self.check(index, request, provider_count, cols)
    }

    pub fn from_cache(
        &self,
        request: &ProviderRequest,
        provider_count: usize,
        cols: usize,
    ) -> Vec<Rc<Link>> {
        // Undefined/pending replies are not checked in the source's cached path.
        (0..provider_count)
            .filter(|index| matches!(self.replies.get(index), Some(Some(_))))
            .filter_map(|index| self.check(index, request, provider_count, cols))
            .collect()
    }

    pub fn links(&self, index: usize) -> Option<&[Rc<Link>]> {
        self.replies.get(&index)?.as_deref()
    }

    fn check(
        &self,
        index: usize,
        request: &ProviderRequest,
        provider_count: usize,
        cols: usize,
    ) -> Option<Rc<Link>> {
        let has_before = (0..index).any(|prior| !matches!(self.replies.get(&prior), Some(None)));
        if !has_before {
            if let Some(link) = self.matching(index, request.position, cols) {
                request.link_provided.set(true);
                return Some(link);
            }
        }
        if self.replies.len() == provider_count && !request.link_provided.get() {
            for index in 0..self.replies.len() {
                if let Some(link) = self.matching(index, request.position, cols) {
                    request.link_provided.set(true);
                    return Some(link);
                }
            }
        }
        None
    }

    fn matching(&self, index: usize, position: Position, cols: usize) -> Option<Rc<Link>> {
        self.links(index)?
            .iter()
            .find(|link| link.range.contains(position, cols))
            .cloned()
    }

    fn remove_intersections(&mut self, y: usize, cols: usize) {
        let mut occupied = BTreeSet::new();
        for index in 0..self.replies.len() {
            let Some(Some(links)) = self.replies.get_mut(&index) else {
                continue;
            };
            links.retain(|link| {
                let start = if link.range.start.y < y {
                    0
                } else {
                    link.range.start.x
                };
                let end = if link.range.end.y > y {
                    cols
                } else {
                    link.range.end.x
                };
                // Deliberately keep cells inserted BEFORE the first overlap. Source
                // splice removes the link but does not roll back those insertions.
                (start..=end).all(|x| occupied.insert(x))
            });
        }
    }
}
