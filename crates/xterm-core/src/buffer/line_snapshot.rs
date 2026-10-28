// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Owned line inspection without holding a line borrow across host callbacks.
use super::{BufferLine, ExtendedAttrs};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct LineSnapshot {
    pub words: Vec<[u32; 3]>,
    pub combined: BTreeMap<usize, Vec<u16>>,
    /// Shared values retain payload and identity, unlike ExtendedAttrs::clone.
    pub extended: BTreeMap<usize, ExtendedAttrs>,
    pub length: usize,
    pub is_wrapped: bool,
    pub cache: Option<(Vec<u16>, bool)>,
}
impl BufferLine {
    pub fn snapshot(&self) -> LineSnapshot {
        LineSnapshot {
            words: self.data[..self.length].to_vec(),
            combined: self.combined.clone(),
            extended: self
                .extended
                .iter()
                .map(|(&index, value)| (index, value.share()))
                .collect(),
            length: self.length,
            is_wrapped: self.is_wrapped,
            cache: self.cache.borrow().clone(),
        }
    }
}
