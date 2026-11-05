// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Shared operation phases for owned components and callback-capable live state.
//! Borrows cover only field access, never user coercion or host callbacks.
use super::{
    animation::{AnimationUpdate, Curve, SmoothOperation},
    state::{Position, PositionUpdate, ScrollState},
    value::ScrollValue,
};
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
};

pub(super) fn read<V, T: Borrow<SmoothOperation<V>>, R>(
    cell: &RefCell<T>,
    read: impl FnOnce(&SmoothOperation<V>) -> R,
) -> R {
    read((*cell.borrow()).borrow())
}
fn write<V, T: BorrowMut<SmoothOperation<V>>>(
    cell: &RefCell<T>,
    write: impl FnOnce(&mut SmoothOperation<V>),
) {
    write((*cell.borrow_mut()).borrow_mut());
}

pub(super) fn accept_dimensions<V: ScrollValue, T: BorrowMut<SmoothOperation<V>>>(
    cell: &RefCell<T>,
    state: ScrollState<V>,
) -> Result<(), V::Error> {
    let to = read(cell, |operation| operation.to.clone());
    let next = state.try_with_position(PositionUpdate {
        left: (!to.left.is_undefined()).then_some(to.left),
        top: (!to.top.is_undefined()).then_some(to.top),
    })?;
    write(cell, |operation| operation.to = next);
    let (from, to, viewport) = read(cell, |operation| {
        (
            operation.from.left.clone(),
            operation.to.left.clone(),
            operation.to.width.clone(),
        )
    });
    let curve = Curve::new(from, &to, &viewport)?;
    write(cell, |operation| operation.horizontal = curve);
    // Horizontal coercion may recursively replace the target/vertical curve.
    // Source selects the vertical arguments only after that coercion returns.
    let (from, to, viewport) = read(cell, |operation| {
        (
            operation.from.top.clone(),
            operation.to.top.clone(),
            operation.to.height.clone(),
        )
    });
    let curve = Curve::new(from, &to, &viewport)?;
    write(cell, |operation| operation.vertical = curve);
    Ok(())
}

pub(super) fn tick<V: ScrollValue, T: Borrow<SmoothOperation<V>>>(
    cell: &RefCell<T>,
    now: impl FnOnce() -> Result<V, V::Error>,
) -> Result<AnimationUpdate<V>, V::Error> {
    let now = now()?;
    let start_time = read(cell, |operation| operation.start_time.clone());
    let elapsed = now.subtract(&start_time)?;
    let duration = read(cell, |operation| operation.duration.clone());
    let completion = elapsed.divide(&duration)?;
    if completion.less(&V::number(1.0))? {
        let horizontal = read(cell, |operation| operation.horizontal.clone());
        let left = horizontal.value(&completion)?;
        let vertical = read(cell, |operation| operation.vertical.clone());
        let top = vertical.value(&completion)?;
        Ok(AnimationUpdate {
            position: Position { left, top },
            done: false,
        })
    } else {
        Ok(AnimationUpdate {
            position: read(cell, |operation| operation.to.position()),
            done: true,
        })
    }
}
