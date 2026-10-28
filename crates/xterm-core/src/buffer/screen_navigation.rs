// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{Buffer, BufferError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WrappedRange {
    pub first: usize,
    pub last: usize,
}

impl Buffer {
    pub fn get_wrapped_range_for_line(&self, y: usize) -> Result<WrappedRange, BufferError> {
        let mut first = y;
        let mut last = y;
        while first > 0
            && self
                .line(first)?
                .try_borrow()
                .map_err(|_| BufferError::BorrowedLine)?
                .is_wrapped
        {
            first -= 1;
        }
        while last
            .checked_add(1)
            .is_some_and(|next| next < self.lines.len())
            && self
                .line(last + 1)?
                .try_borrow()
                .map_err(|_| BufferError::BorrowedLine)?
                .is_wrapped
        {
            last += 1;
        }
        Ok(WrappedRange { first, last })
    }
    pub fn setup_tab_stops(&mut self, start: Option<usize>) -> Result<(), BufferError> {
        let width = self.positive_option("tabStopWidth", false)?;
        let mut position = if let Some(start) = start {
            if self.tabs.contains(&start) {
                start
            } else {
                self.prev_stop(Some(start))
            }
        } else {
            self.tabs.clear();
            0
        };
        while position < self.cols {
            self.tabs.insert(position);
            position = position
                .checked_add(width)
                .ok_or(BufferError::SizeOverflow)?;
        }
        Ok(())
    }
    pub fn prev_stop(&self, x: Option<usize>) -> usize {
        self.tabs
            .range(..x.unwrap_or(self.x))
            .next_back()
            .copied()
            .unwrap_or(0)
            .min(self.cols - 1)
    }
    pub fn next_stop(&self, x: Option<usize>) -> usize {
        use std::ops::Bound::{Excluded, Unbounded};
        self.tabs
            .range((Excluded(x.unwrap_or(self.x)), Unbounded))
            .next()
            .copied()
            .unwrap_or(self.cols - 1)
            .min(self.cols - 1)
    }
}
