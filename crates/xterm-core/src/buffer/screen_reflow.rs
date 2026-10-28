// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{Buffer, BufferError, LineRef};
use crate::buffer::{reflow::*, AttributeData, BufferLine};
use std::{cell::RefCell, rc::Rc};
#[path = "screen_reflow_smaller.rs"]
mod smaller;

impl Buffer {
    pub(super) fn reflow(&mut self, cols: usize, rows: usize) -> Result<(), BufferError> {
        if self.cols == cols {
            return Ok(());
        }
        if cols > self.cols {
            self.reflow_larger(cols, rows)
        } else {
            self.reflow_smaller(cols, rows)
        }
    }
    fn reflow_larger(&mut self, cols: usize, rows: usize) -> Result<(), BufferError> {
        let null = self.get_null_cell(Some(&AttributeData::default()));
        let absolute_y = self
            .ybase
            .checked_add(self.y)
            .ok_or(BufferError::SizeOverflow)?;
        let remove = reflow_larger_get_lines_to_remove(
            &self.lines,
            self.cols,
            cols,
            absolute_y,
            &null,
            self.reflow_cursor_line()?,
        )?;
        if !remove.is_empty() {
            let layout = reflow_larger_create_new_layout(&self.lines, &remove)?;
            reflow_larger_apply_new_layout(&self.lines, &layout.layout)?;
            for _ in 0..layout.count_removed {
                if self.ybase == 0 {
                    self.y = self.y.saturating_sub(1);
                    if self.lines.len() < rows {
                        self.lines.push(Rc::new(RefCell::new(BufferLine::new(
                            cols,
                            Some(&null),
                            false,
                        ))))?;
                    }
                } else {
                    if self.ydisp == self.ybase {
                        self.ydisp -= 1;
                    }
                    self.ybase -= 1;
                }
            }
            self.saved_y = self
                .saved_y
                .saturating_sub(
                    i64::try_from(layout.count_removed).map_err(|_| BufferError::SizeOverflow)?,
                )
                .max(0);
        }
        Ok(())
    }
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
