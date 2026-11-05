// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Typed range adapter over the same fallible/live-payload priority algorithm.
//! A browser payload retains callbacks with the data, including during disposal reentry.
use super::{Link, ProviderReplies, ProviderRequest};
use std::{collections::BTreeSet, convert::Infallible, rc::Rc};
fn infallible<T>(result: Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => match error {},
    }
}
impl<T: AsRef<Link>> ProviderReplies<T> {
    pub fn clear(&self) -> Vec<Rc<T>> {
        self.reset()
    }
    pub fn receive(
        &self,
        index: usize,
        links: Option<Vec<Rc<T>>>,
        request: &ProviderRequest,
        count: usize,
        cols: usize,
    ) -> Option<Rc<T>> {
        let result = self.receive_unfiltered(index, links, request, count, cols);
        self.filter_complete(request.position.y, count, cols);
        result
    }
    pub fn receive_unfiltered(
        &self,
        index: usize,
        links: Option<Vec<Rc<T>>>,
        request: &ProviderRequest,
        count: usize,
        cols: usize,
    ) -> Option<Rc<T>> {
        infallible(
            self.try_receive_unfiltered(index, links, request, count, |link| {
                Ok(link.is_some_and(|link| {
                    link.as_ref()
                        .as_ref()
                        .range
                        .contains(request.position, cols)
                }))
            }),
        )
    }
    pub fn from_cache_index(
        &self,
        index: usize,
        request: &ProviderRequest,
        count: usize,
        cols: usize,
    ) -> Option<Rc<T>> {
        infallible(self.try_cached_index(index, request, count, |link| {
            Ok(link.is_some_and(|link| {
                link.as_ref()
                    .as_ref()
                    .range
                    .contains(request.position, cols)
            }))
        }))
    }
    pub fn from_cache(&self, request: &ProviderRequest, count: usize, cols: usize) -> Vec<Rc<T>> {
        (0..count)
            .filter_map(|index| self.from_cache_index(index, request, count, cols))
            .collect()
    }
    pub fn filter_complete(&self, y: usize, count: usize, cols: usize) {
        let mut occupied = BTreeSet::new();
        infallible(self.try_filter_complete(count, |link| {
            let link = link.as_ref().as_ref();
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
