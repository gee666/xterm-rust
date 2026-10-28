// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Callback checkpoints for shared Buffer handles. Ordinary owned Buffer methods
//! use the same marker loop; shared callers release RefCell borrows first.
use super::{resize::ResizeTarget, AttributeData, Buffer, BufferError};
use crate::buffer::marker::Marker;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

impl Buffer {
    pub(crate) fn fill_viewport_rows_shared(
        buffer: &Rc<RefCell<Self>>,
        attr: Option<&AttributeData>,
    ) -> Result<(), BufferError> {
        fill_rows(&ResizeTarget::Shared(buffer), attr)
    }
    pub(crate) fn clear_all_markers_shared(buffer: &Rc<RefCell<Self>>) -> Result<(), BufferError> {
        let (markers, clearing) = {
            let state = buffer.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
            (Rc::clone(&state.markers), Rc::clone(&state.is_clearing))
        };
        clear_markers(&markers, &clearing)
    }

    pub(crate) fn clear_markers_shared(
        buffer: &Rc<RefCell<Self>>,
        row: i64,
    ) -> Result<(), BufferError> {
        let (markers, clearing) = {
            let state = buffer.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
            (Rc::clone(&state.markers), Rc::clone(&state.is_clearing))
        };
        clear_markers_on_line(&markers, &clearing, row)
    }

    pub(crate) fn dispose_shared(buffer: &Rc<RefCell<Self>>) -> Result<(), BufferError> {
        let cleanup = {
            let mut state = buffer
                .try_borrow_mut()
                .map_err(|_| BufferError::BorrowedLine)?;
            if state.disposed {
                return Ok(());
            }
            state.disposed = true;
            state.cleanup_pending = false;
            Rc::clone(&state.cleanup_events)
        };
        // Host cancellation can synchronously inspect terminal state too.
        cleanup.fire(&false)?;
        Self::clear_all_markers_shared(buffer)
    }
}

pub(super) fn fill_viewport_rows_owned(
    buffer: &mut Buffer,
    attr: Option<&AttributeData>,
) -> Result<(), BufferError> {
    fill_rows(&ResizeTarget::Owned(RefCell::new(buffer)), attr)
}
fn fill_rows(target: &ResizeTarget<'_>, attr: Option<&AttributeData>) -> Result<(), BufferError> {
    let rows = target.read(|buffer| {
        if buffer.lines.is_empty() {
            buffer.rows
        } else {
            0
        }
    })?;
    let default = AttributeData::default();
    // Source captures its row count, but reloads the receiver's line collection
    // and live host dimensions for every push after synchronous callbacks.
    for _ in 0..rows {
        let (lines, line) = target.read(|buffer| {
            (
                buffer.lines.share(),
                buffer.get_blank_line(attr.unwrap_or(&default), false),
            )
        })?;
        lines.push(line)?;
    }
    Ok(())
}

pub(super) fn clear_markers_on_line(
    markers: &RefCell<Vec<Marker>>,
    clearing: &Cell<bool>,
    row: i64,
) -> Result<(), BufferError> {
    clearing.set(true);
    let mut index = 0;
    loop {
        let marker = markers.borrow().get(index).cloned();
        let Some(marker) = marker else {
            break;
        };
        if marker.line_value().strict_eq_number(row as f64) {
            marker.dispose()?;
            if index < markers.borrow().len() {
                markers.borrow_mut().remove(index);
            }
        } else {
            index += 1;
        }
    }
    clearing.set(false);
    Ok(())
}

pub(super) fn clear_markers(
    markers: &RefCell<Vec<Marker>>,
    clearing: &Cell<bool>,
) -> Result<(), BufferError> {
    clearing.set(true);
    let mut index = 0;
    loop {
        let marker = markers.borrow().get(index).cloned();
        let Some(marker) = marker else {
            break;
        };
        marker.dispose()?;
        index += 1;
    }
    // Preserve the source live iteration and failure state. A callback that
    // throws leaves the list and clearing flag exactly where dispatch stopped.
    markers.borrow_mut().clear();
    clearing.set(false);
    Ok(())
}
