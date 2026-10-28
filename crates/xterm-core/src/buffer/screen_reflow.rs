// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{resize::ResizeTarget, Buffer, BufferError, LineRef};
use crate::buffer::{reflow::*, AttributeData, BufferLine};
use std::{cell::RefCell, rc::Rc};
#[path = "screen_reflow_smaller.rs"]
mod smaller;

pub(super) fn reflow(
    target: &ResizeTarget<'_>,
    cols: usize,
    rows: usize,
) -> Result<(), BufferError> {
    let old = target.read(|b| b.cols)?;
    if old == cols {
        return Ok(());
    }
    if cols < old {
        let plan = target.update(|b| b.prepare_reflow_smaller(cols, rows))?;
        return plan.apply(|| target.lines());
    }
    let (absolute, null, cursor_line) = target.read(|b| {
        Ok::<_, BufferError>((
            b.ybase.checked_add(b.y).ok_or(BufferError::SizeOverflow)?,
            b.get_null_cell(Some(&AttributeData::default())),
            b.reflow_cursor_line()?,
        ))
    })??;
    let remove = reflow_larger_get_lines_to_remove(
        &target.lines()?,
        old,
        cols,
        absolute,
        &null.snapshot(),
        cursor_line,
    )?;
    if remove.is_empty() {
        return Ok(());
    }
    // The source captures a list for each helper call, but reads `this.lines`
    // again afterward. A callback may replace the buffer's list synchronously.
    let layout = reflow_larger_create_new_layout(&target.lines()?, &remove)?;
    reflow_larger_apply_new_layout(&target.lines()?, &layout.layout)?;
    let null = target.read(|b| b.get_null_cell(Some(&AttributeData::default())))?;
    for _ in 0..layout.count_removed {
        let append = target.update(|b| {
            if b.ybase == 0 {
                b.y = b.y.saturating_sub(1);
                Ok(b.lines.len() < rows)
            } else {
                if b.ydisp == b.ybase {
                    b.ydisp -= 1;
                }
                b.ybase -= 1;
                Ok(false)
            }
        })?;
        if append {
            target.lines()?.push(Rc::new(RefCell::new(BufferLine::new(
                cols,
                Some(&null.snapshot()),
                false,
            ))))?;
        }
    }
    target.update(|b| {
        b.saved_y = b
            .saved_y
            .saturating_sub(
                i64::try_from(layout.count_removed).map_err(|_| BufferError::SizeOverflow)?,
            )
            .max(0);
        Ok(())
    })
}
fn copy_cells(
    lines: &[LineRef],
    source: usize,
    from: usize,
    destination: usize,
    to: usize,
    count: usize,
) -> Result<(), BufferError> {
    let src = lines.get(source).ok_or(BufferError::MissingLine(source))?;
    let dest = lines
        .get(destination)
        .ok_or(BufferError::MissingLine(destination))?;
    if Rc::ptr_eq(src, dest) {
        dest.try_borrow_mut()
            .map_err(|_| BufferError::BorrowedLine)?
            .copy_cells_within(from, to, count, true);
    } else {
        let source = src.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
        dest.try_borrow_mut()
            .map_err(|_| BufferError::BorrowedLine)?
            .copy_cells_from(&source, from, to, count, true);
    }
    Ok(())
}
