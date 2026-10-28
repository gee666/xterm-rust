// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{copy_cells, Buffer, BufferError, LineRef};
use crate::{
    buffer::{reflow::*, AttributeData},
    circular_list::ListChange,
};

struct Insertion {
    start: usize,
    lines: Vec<LineRef>,
}
impl Buffer {
    pub(super) fn reflow_smaller(&mut self, cols: usize, rows: usize) -> Result<(), BufferError> {
        let null = self.get_null_cell(Some(&AttributeData::default()));
        let mut insertions = Vec::new();
        let mut count_to_insert = 0;
        let mut y = self.lines.len();
        while y > 0 {
            y -= 1;
            let Some(line) = self.lines.get(y) else {
                continue;
            };
            {
                let line = line.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
                if !line.is_wrapped && line.get_trimmed_length() <= cols {
                    continue;
                }
            }
            let mut wrapped = vec![line];
            while y > 0
                && wrapped[0]
                    .try_borrow()
                    .map_err(|_| BufferError::BorrowedLine)?
                    .is_wrapped
            {
                y -= 1;
                wrapped.insert(0, self.line(y)?);
            }
            let cursor = self
                .ybase
                .checked_add(self.y)
                .ok_or(BufferError::SizeOverflow)?;
            if !self.reflow_cursor_line()? && cursor >= y && cursor - y < wrapped.len() {
                continue;
            }
            let original_count = wrapped.len();
            let last_length = wrapped[original_count - 1]
                .try_borrow()
                .map_err(|_| BufferError::BorrowedLine)?
                .get_trimmed_length();
            let lengths = reflow_smaller_get_new_line_lengths(&wrapped, self.cols, cols)?;
            let signed_added = i64::try_from(lengths.len())
                .map_err(|_| BufferError::SizeOverflow)?
                - i64::try_from(original_count).map_err(|_| BufferError::SizeOverflow)?;
            let lines_to_add =
                usize::try_from(signed_added.max(0)).map_err(|_| BufferError::SizeOverflow)?;
            let trim_base = if self.ybase == 0 && self.y != self.lines.len() - 1 {
                self.y
            } else {
                self.lines.len()
            };
            let trimmed = usize::try_from(
                (i128::try_from(trim_base).map_err(|_| BufferError::SizeOverflow)?
                    + i128::from(signed_added)
                    - i128::try_from(self.lines.max_length())
                        .map_err(|_| BufferError::SizeOverflow)?)
                .max(0),
            )
            .map_err(|_| BufferError::SizeOverflow)?;
            let new_lines = (0..lines_to_add)
                .map(|_| self.get_blank_line(&AttributeData::default(), true))
                .collect::<Vec<_>>();
            if !new_lines.is_empty() {
                insertions.push(Insertion {
                    start: y
                        .checked_add(original_count)
                        .and_then(|n| n.checked_add(count_to_insert))
                        .ok_or(BufferError::SizeOverflow)?,
                    lines: new_lines.clone(),
                });
                count_to_insert = count_to_insert
                    .checked_add(lines_to_add)
                    .ok_or(BufferError::SizeOverflow)?;
            }
            wrapped.extend(new_lines);
            let mut destination = lengths.len().checked_sub(1);
            let mut dest_col = destination.map_or(0, |index| lengths[index]);
            if dest_col == 0 {
                destination = destination.and_then(|index| index.checked_sub(1));
                dest_col = destination.map_or(0, |index| lengths[index]);
            }
            let mut source_count = original_count;
            let mut src_col = last_length;
            while source_count > 0 {
                // Upstream deliberately stops when an unexpected destination is missing.
                let Some(dest) = destination.filter(|&index| index < wrapped.len()) else {
                    break;
                };
                let copy = src_col.min(dest_col);
                copy_cells(
                    &wrapped,
                    source_count - 1,
                    src_col - copy,
                    dest,
                    dest_col - copy,
                    copy,
                )?;
                dest_col -= copy;
                if dest_col == 0 {
                    destination = dest.checked_sub(1);
                    dest_col = destination.map_or(0, |index| lengths[index]);
                }
                src_col -= copy;
                if src_col == 0 {
                    source_count -= 1;
                    src_col = get_wrapped_line_trimmed_length(
                        &wrapped,
                        source_count.saturating_sub(1),
                        self.cols,
                    )?;
                }
            }
            for (index, line) in wrapped.iter().enumerate() {
                if let Some(&length) = lengths.get(index).filter(|&&length| length < cols) {
                    line.try_borrow_mut()
                        .map_err(|_| BufferError::BorrowedLine)?
                        .set_cell(length, &null);
                }
            }
            for _ in 0..lines_to_add.saturating_sub(trimmed) {
                if self.ybase == 0 {
                    if self.y < rows - 1 {
                        self.y += 1;
                        self.lines.pop()?;
                    } else {
                        self.ybase += 1;
                        self.ydisp = self.ydisp.checked_add(1).ok_or(BufferError::SizeOverflow)?;
                    }
                } else {
                    let maximum = self
                        .lines
                        .max_length()
                        .min(
                            self.lines
                                .len()
                                .checked_add(count_to_insert)
                                .ok_or(BufferError::SizeOverflow)?,
                        )
                        .saturating_sub(rows);
                    if self.ybase < maximum {
                        if self.ybase == self.ydisp {
                            self.ydisp += 1;
                        }
                        self.ybase += 1;
                    }
                }
            }
            self.saved_y = self
                .saved_y
                .checked_add(signed_added)
                .ok_or(BufferError::SizeOverflow)?
                .min(
                    i64::try_from(
                        self.ybase
                            .checked_add(rows - 1)
                            .ok_or(BufferError::SizeOverflow)?,
                    )
                    .map_err(|_| BufferError::SizeOverflow)?,
                );
        }
        if !insertions.is_empty() {
            self.apply_insertions(insertions, count_to_insert)?;
        }
        Ok(())
    }
    fn apply_insertions(
        &self,
        insertions: Vec<Insertion>,
        count: usize,
    ) -> Result<(), BufferError> {
        let original = (0..self.lines.len())
            .map(|index| self.line(index))
            .collect::<Result<Vec<_>, _>>()?;
        let original_length = original.len();
        let expanded = original_length
            .checked_add(count)
            .ok_or(BufferError::SizeOverflow)?;
        self.lines.set_length(self.lines.max_length().min(expanded));
        let mut source = i64::try_from(original_length).map_err(|_| BufferError::SizeOverflow)? - 1;
        let mut target = i64::try_from(self.lines.max_length().min(expanded))
            .map_err(|_| BufferError::SizeOverflow)?
            - 1;
        let mut next = 0;
        let mut inserted = 0;
        let mut events = Vec::new();
        while target >= 0 {
            if let Some(insertion) = insertions.get(next).filter(|insertion| {
                i64::try_from(insertion.start).is_ok_and(|start| start > source + inserted)
            }) {
                for line in insertion.lines.iter().rev() {
                    self.lines.set_signed(target, Some(line.clone()))?;
                    target -= 1;
                }
                events.push(ListChange {
                    index: usize::try_from(source + 1).map_err(|_| BufferError::SizeOverflow)?,
                    amount: insertion.lines.len(),
                });
                inserted = inserted
                    .checked_add(
                        i64::try_from(insertion.lines.len())
                            .map_err(|_| BufferError::SizeOverflow)?,
                    )
                    .ok_or(BufferError::SizeOverflow)?;
                next += 1;
            } else {
                let line = usize::try_from(source)
                    .ok()
                    .and_then(|index| original.get(index))
                    .cloned();
                self.lines.set_signed(target, line)?;
                source -= 1;
                target -= 1;
            }
        }
        let mut emitted = 0;
        for mut event in events.into_iter().rev() {
            event.index = event
                .index
                .checked_add(emitted)
                .ok_or(BufferError::SizeOverflow)?;
            self.lines.on_insert_emitter.fire(&event)?;
            emitted = emitted
                .checked_add(event.amount)
                .ok_or(BufferError::SizeOverflow)?;
        }
        let trim = expanded.saturating_sub(self.lines.max_length());
        if trim > 0 {
            self.lines.on_trim_emitter.fire(&trim)?;
        }
        Ok(())
    }
}
