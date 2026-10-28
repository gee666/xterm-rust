// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Callback checkpoints for shared Buffer handles. Ordinary owned Buffer methods
//! use the same marker loop; shared callers release RefCell borrows first.
use super::{Buffer, BufferError};
use crate::buffer::marker::Marker;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

impl Buffer {
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
        {
            let mut state = buffer
                .try_borrow_mut()
                .map_err(|_| BufferError::BorrowedLine)?;
            state.cleanup_pending = false;
            // These private host scheduler callbacks never access Buffer state.
            state.cleanup_events.fire(&false)?;
        }
        Self::clear_all_markers_shared(buffer)
    }
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
        if marker.line() == row {
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
