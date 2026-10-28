// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{SearchBuffer, SearchError};
use crate::{buffer::Buffer, circular_list::CircularListError};
use std::{collections::HashMap, rc::Rc};

/// Search the live buffer with the same unbounded ring reads as BufferApiView.
///
/// The callback must remain read-only. Immutable guards cover allocated physical
/// slots, including stale slots beyond logical length, without scanning unused
/// scrollback capacity or copying cell data. This allocation of handle snapshots
/// is intentional: returned line references cannot outlive an owning Rc/Ref pair.
pub fn with_buffer<R>(
    buffer: &Buffer,
    cols: usize,
    run: impl FnOnce(SearchBuffer<'_>) -> Result<R, SearchError>,
) -> Result<R, SearchError> {
    let stored = buffer.lines.stored_values().map_err(SearchError::Storage)?;
    let mut guards = Vec::new();
    guards
        .try_reserve_exact(stored.len())
        .map_err(|_| SearchError::Storage(CircularListError::AllocationFailed))?;
    let mut indices = HashMap::new();
    indices
        .try_reserve(stored.len())
        .map_err(|_| SearchError::Storage(CircularListError::AllocationFailed))?;
    for (index, line) in stored.iter().enumerate() {
        guards.push(line.try_borrow().map_err(|_| SearchError::BorrowedLine)?);
        indices.insert(Rc::as_ptr(line), index);
    }
    let lookup = |row: isize| {
        // BufferApiView indexes with JS Numbers, including rounding beyond 2^53.
        let line = buffer.lines.get_number(row as f64)?;
        let index = *indices.get(&Rc::as_ptr(&line))?;
        Some(&*guards[index])
    };
    let search_end = buffer
        .ybase
        .checked_add(buffer.host_dimensions.rows.get())
        .ok_or(SearchError::IndexOverflow)?;
    run(SearchBuffer::from_ring_lookup(
        search_end,
        cols,
        buffer.lines.max_length(),
        &lookup,
    )?)
}
