// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{TerminalError, TerminalEvent, TerminalState};
use crate::{input::text_decoder::string_from_code_point, unicode::UnicodeCharProperties};
use std::cell::Cell;

impl TerminalState {
    pub(crate) fn print(
        &mut self,
        data: &[u32],
        preceding: &Cell<u32>,
    ) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let cols = self.cols();
        let rows = self.rows();
        let modes = self.core.dec_private_modes.borrow().clone();
        let insert = self.core.modes.borrow().insert_mode;
        let a11y = self.option_bool("screenReaderMode")?;
        let attrs = self.attributes.clone();
        let mut info = UnicodeCharProperties::from_raw(preceding.get());
        let mut row = self.line(active.borrow().y)?;
        self.dirty_row(active.borrow().y);
        {
            let b = active.borrow();
            if b.x > 0 && row.borrow().get_width(b.x - 1) == Some(2) {
                row.borrow_mut()
                    .set_cell_from_codepoint(b.x - 1, 0, 1, &attrs);
            }
        }
        for &original in data {
            if original == 0xad {
                continue;
            }
            let code = if original < 127 {
                self.charset
                    .charset
                    .as_ref()
                    .and_then(|map| map.get(&(original as u16)))
                    .filter(|value| !value.is_empty())
                    .map_or(original, |value| u32::from(value[0]))
            } else {
                original
            };
            let current = self.unicode.char_properties(code, info)?;
            let width = usize::from(current.width());
            let old_width = if current.should_join() {
                usize::from(info.width())
            } else {
                0
            };
            info = current;
            if a11y {
                self.events
                    .borrow_mut()
                    .push(TerminalEvent::AccessibilityChar(string_from_code_point(
                        code,
                    )));
            }
            let link_id = self.attributes.extended.url_id();
            if link_id != 0 {
                let b = active.borrow();
                let line = b
                    .ybase
                    .checked_add(b.y)
                    .ok_or(TerminalError::InvalidDimensions)?;
                let line = i64::try_from(line).map_err(|_| TerminalError::InvalidDimensions)?;
                drop(b);
                self.links.add_line_to_link(link_id, line)?;
            }
            if active.borrow().x + width > cols + old_width {
                if modes.wraparound {
                    let old_row = RcLine::clone(&row);
                    let old_col = active.borrow().x.saturating_sub(old_width);
                    let scroll = {
                        let mut b = active.borrow_mut();
                        b.x = old_width;
                        b.y += 1;
                        if b.y == b.scroll_bottom + 1 {
                            b.y -= 1;
                            true
                        } else {
                            b.y = b.y.min(rows - 1);
                            b.set_wrapped(b.ybase + b.y, true)
                                .map_err(|e| TerminalError::Buffer(e.to_string()))?;
                            false
                        }
                    };
                    if scroll {
                        self.scroll(true)?;
                    }
                    row = self.line(active.borrow().y)?;
                    if old_width > 0 {
                        // A joining mark may widen the prior grapheme across the margin.
                        let saved = old_row.borrow().clone();
                        row.borrow_mut()
                            .copy_cells_from(&saved, old_col, 0, old_width, false);
                    }
                    for column in old_col..cols {
                        old_row
                            .borrow_mut()
                            .set_cell_from_codepoint(column, 0, 1, &attrs);
                    }
                } else {
                    active.borrow_mut().x = cols - 1;
                    if width == 2 {
                        continue;
                    }
                }
            }
            let x = active.borrow().x;
            if current.should_join() && x != 0 {
                let offset = if row.borrow().get_width(x - 1).is_some_and(|w| w != 0) {
                    1
                } else {
                    2
                };
                row.borrow_mut().add_codepoint_to_cell(
                    x.saturating_sub(offset),
                    code,
                    width as i32,
                );
                for column in 0..width.saturating_sub(old_width) {
                    row.borrow_mut()
                        .set_cell_from_codepoint(x + column, 0, 0, &attrs);
                    active.borrow_mut().x += 1;
                }
                continue;
            }
            if insert {
                let fill = active.borrow().get_null_cell(Some(&attrs));
                row.borrow_mut()
                    .insert_cells(x, width.saturating_sub(old_width), &fill);
                if row.borrow().get_width(cols - 1) == Some(2) {
                    row.borrow_mut()
                        .set_cell_from_codepoint(cols - 1, 0, 1, &attrs);
                }
            }
            row.borrow_mut()
                .set_cell_from_codepoint(x, code, width as i32, &attrs);
            active.borrow_mut().x += 1;
            for column in 1..width {
                row.borrow_mut()
                    .set_cell_from_codepoint(x + column, 0, 0, &attrs);
                active.borrow_mut().x += 1;
            }
        }
        preceding.set(info.raw());
        let b = active.borrow();
        if b.x < cols
            && !data.is_empty()
            && row.borrow().get_width(b.x) == Some(0)
            && row.borrow().has_content(b.x) == Some(0)
        {
            row.borrow_mut().set_cell_from_codepoint(b.x, 0, 1, &attrs);
        }
        self.dirty_row(b.y);
        Ok(())
    }
}
use std::{cell::RefCell, rc::Rc};
type RcLine = Rc<RefCell<crate::buffer::BufferLine>>;
