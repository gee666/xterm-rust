// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Native snapshot adapter over the same fallible/live-payload priority algorithm.
use super::{Link, ProviderReplies, ProviderRequest};
use std::{collections::BTreeSet, convert::Infallible, rc::Rc};
fn infallible<T>(result: Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => match error {},
    }
}
impl ProviderReplies<Link> {
    pub fn clear(&mut self) -> Vec<Rc<Link>> {
        self.reset()
    }
    pub fn receive(
        &mut self,
        index: usize,
        links: Option<Vec<Rc<Link>>>,
        request: &ProviderRequest,
        count: usize,
        cols: usize,
    ) -> Option<Rc<Link>> {
        let result = self.receive_unfiltered(index, links, request, count, cols);
        self.filter_complete(request.position.y, count, cols);
        result
    }
    pub fn receive_unfiltered(
        &mut self,
        index: usize,
        links: Option<Vec<Rc<Link>>>,
        request: &ProviderRequest,
        count: usize,
        cols: usize,
    ) -> Option<Rc<Link>> {
        infallible(
            self.try_receive_unfiltered(index, links, request, count, |link| {
                Ok(link.is_some_and(|link| link.range.contains(request.position, cols)))
            }),
        )
    }
    pub fn from_cache_index(
        &self,
        index: usize,
        request: &ProviderRequest,
        count: usize,
        cols: usize,
    ) -> Option<Rc<Link>> {
        infallible(self.try_cached_index(index, request, count, |link| {
            Ok(link.is_some_and(|link| link.range.contains(request.position, cols)))
        }))
    }
    pub fn from_cache(
        &self,
        request: &ProviderRequest,
        count: usize,
        cols: usize,
    ) -> Vec<Rc<Link>> {
        (0..count)
            .filter_map(|index| self.from_cache_index(index, request, count, cols))
            .collect()
    }
    pub fn filter_complete(&mut self, y: usize, count: usize, cols: usize) {
        let mut occupied = BTreeSet::new();
        infallible(self.try_filter_complete(count, |link| {
            let start = if link.range.start.y < y as u64 {
                0
            } else {
                link.range.start.x
            };
            let end = if link.range.end.y > y as u64 {
                cols as u64
            } else {
                link.range.end.x
            };
            // Retain cells inserted before a rejected link's first overlap.
            Ok((start..=end).all(|x| occupied.insert(x)))
        }));
    }
}
