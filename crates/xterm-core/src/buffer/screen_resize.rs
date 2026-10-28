// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{Buffer, BufferError, LineRef};
use crate::{
    buffer::{AttributeData, BufferLine, CellData},
    circular_list::CircularList,
};
use std::{cell::RefCell, rc::Rc};

// Both ownership paths execute one algorithm. The shared path must end every
// state borrow before a list/marker/host callback can synchronously reenter.
pub(super) enum ResizeTarget<'a> {
    Owned(RefCell<&'a mut Buffer>),
    Shared(&'a Rc<RefCell<Buffer>>),
}
impl ResizeTarget<'_> {
    pub(super) fn read<R>(&self, action: impl FnOnce(&Buffer) -> R) -> Result<R, BufferError> {
        match self {
            Self::Owned(state) => {
                let state = state.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
                Ok(action(&state))
            }
            Self::Shared(state) => {
                let state = state.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
                Ok(action(&state))
            }
        }
    }
    pub(super) fn update<R>(
        &self,
        action: impl FnOnce(&mut Buffer) -> Result<R, BufferError>,
    ) -> Result<R, BufferError> {
        match self {
            Self::Owned(state) => {
                let mut state = state
                    .try_borrow_mut()
                    .map_err(|_| BufferError::BorrowedLine)?;
                action(&mut state)
            }
            Self::Shared(state) => {
                let mut state = state
                    .try_borrow_mut()
                    .map_err(|_| BufferError::BorrowedLine)?;
                action(&mut state)
            }
        }
    }
    pub(super) fn lines(&self) -> Result<CircularList<LineRef>, BufferError> {
        self.read(|b| b.lines.share())
    }
}
impl Buffer {
    pub fn resize(&mut self, cols: usize, rows: usize) -> Result<(), BufferError> {
        resize_target(&ResizeTarget::Owned(RefCell::new(self)), cols, rows)
    }
    /// Browser/service callers use this path so synchronous observers can read
    /// or mutate this very same Buffer handle at the source event checkpoints.
    pub fn resize_shared(
        buffer: &Rc<RefCell<Self>>,
        cols: usize,
        rows: usize,
    ) -> Result<(), BufferError> {
        resize_target(&ResizeTarget::Shared(buffer), cols, rows)
    }
}
fn resize_target(target: &ResizeTarget<'_>, cols: usize, rows: usize) -> Result<(), BufferError> {
    if cols == 0 || rows == 0 {
        return Err(BufferError::InvalidDimensions);
    }
    let (options, scrollback, null) = target.read(|b| {
        (
            Rc::clone(&b.options),
            b.has_scrollback,
            b.get_null_cell(Some(&AttributeData::default())),
        )
    })?;
    let maximum = super::options::correct_buffer_length(scrollback, rows, &options)?;
    if maximum > target.lines()?.max_length() {
        target.lines()?.set_max_length(maximum)?;
    }
    let mut dirty = 0;
    let filled = !target.lines()?.is_empty();
    if filled {
        if target.read(|b| b.cols)? < cols {
            dirty += resize_lines(target, cols, &null)?;
        }
        let old_rows = target.read(|b| b.rows)?;
        let add = if old_rows < rows {
            grow_rows(target, old_rows, cols, rows, &null)?
        } else {
            shrink_rows(target, old_rows, rows)?;
            0
        };
        if maximum < target.lines()?.max_length() {
            let trim = target.lines()?.len().saturating_sub(maximum);
            if trim > 0 {
                target.lines()?.trim_start(trim)?;
                target.update(|b| {
                    b.ybase = b.ybase.saturating_sub(trim);
                    b.ydisp = b.ydisp.saturating_sub(trim);
                    b.saved_y = b
                        .saved_y
                        .saturating_sub(i64::try_from(trim).map_err(|_| BufferError::SizeOverflow)?)
                        .max(0);
                    Ok(())
                })?;
            }
            target.lines()?.set_max_length(maximum)?;
        }
        target.update(|b| {
            b.x = b.x.min(cols - 1);
            b.y =
                b.y.min(rows - 1)
                    .checked_add(add)
                    .ok_or(BufferError::SizeOverflow)?;
            b.saved_x = b.saved_x.min(cols - 1);
            b.scroll_top = 0;
            Ok(())
        })?;
    }
    target.update(|b| {
        b.scroll_bottom = rows - 1;
        Ok(())
    })?;
    let (options, scrollback) = target.read(|b| (Rc::clone(&b.options), b.has_scrollback))?;
    if super::options::is_reflow_enabled(scrollback, &options)? {
        super::reflow::reflow(target, cols, rows)?;
        if target.read(|b| b.cols)? > cols {
            dirty += resize_lines(target, cols, &null)?;
        }
    }
    let cleanup = target.update(|b| {
        b.cols = cols;
        b.rows = rows;
        if !b.lines.is_empty() {
            b.y =
                b.y.min(b.lines.len().saturating_sub(b.ybase).saturating_sub(1));
        }
        b.cleanup_pending = false;
        Ok(Rc::clone(&b.cleanup_events))
    })?;
    cleanup.fire(&false)?;
    if dirty as f64 > 0.1 * target.lines()?.len() as f64 {
        target.update(|b| {
            b.cleanup_position = 0;
            b.cleanup_pending = true;
            Ok(())
        })?;
        cleanup.fire(&true)?;
    }
    Ok(())
}
fn resize_lines(
    target: &ResizeTarget<'_>,
    cols: usize,
    null: &CellData,
) -> Result<usize, BufferError> {
    let mut dirty = 0;
    for index in 0..target.lines()?.len() {
        let line = target.read(|b| b.line(index))??;
        dirty += usize::from(
            line.try_borrow_mut()
                .map_err(|_| BufferError::BorrowedLine)?
                .resize(cols, null),
        );
    }
    Ok(dirty)
}
fn grow_rows(
    target: &ResizeTarget<'_>,
    old_rows: usize,
    cols: usize,
    rows: usize,
    null: &CellData,
) -> Result<usize, BufferError> {
    let mut add = 0;
    for _ in old_rows..rows {
        let base = target.read(|b| b.ybase)?;
        if target.lines()?.len() >= rows.checked_add(base).ok_or(BufferError::SizeOverflow)? {
            continue;
        }
        let options = target.read(|b| Rc::clone(&b.options))?;
        let configured = super::options::windows_pty_configured(&options)?;
        let reveal = if configured {
            false
        } else {
            target.read(|b| {
                if b.ybase == 0 {
                    return Ok(false);
                }
                let end = b
                    .ybase
                    .checked_add(b.y)
                    .and_then(|n| n.checked_add(add))
                    .and_then(|n| n.checked_add(1))
                    .ok_or(BufferError::SizeOverflow)?;
                Ok(b.lines.len() <= end)
            })??
        };
        if reveal {
            target.update(|b| {
                b.ybase -= 1;
                b.ydisp = b.ydisp.saturating_sub(1);
                Ok(())
            })?;
            add += 1;
        } else {
            target.lines()?.push(Rc::new(RefCell::new(BufferLine::new(
                cols,
                Some(null),
                false,
            ))))?;
        }
    }
    Ok(add)
}
fn shrink_rows(target: &ResizeTarget<'_>, old_rows: usize, rows: usize) -> Result<(), BufferError> {
    for _ in (rows..old_rows).rev() {
        let (base, y) = target.read(|b| (b.ybase, b.y))?;
        if target.lines()?.len() > rows.checked_add(base).ok_or(BufferError::SizeOverflow)? {
            if target.lines()?.len()
                > base
                    .checked_add(y)
                    .and_then(|n| n.checked_add(1))
                    .ok_or(BufferError::SizeOverflow)?
            {
                target.lines()?.pop()?;
            } else {
                target.update(|b| {
                    b.ybase = b.ybase.checked_add(1).ok_or(BufferError::SizeOverflow)?;
                    b.ydisp = b.ydisp.checked_add(1).ok_or(BufferError::SizeOverflow)?;
                    Ok(())
                })?;
            }
        }
    }
    Ok(())
}
