// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{TerminalError, TerminalEvent, TerminalState};
impl TerminalState {
    pub(crate) fn escape(&mut self, final_byte: &str) -> Result<(), TerminalError> {
        match final_byte {
            "7" => self.save_cursor()?,
            "8" => self.restore_cursor()?,
            "D" => self.index()?,
            "E" => {
                self.active_buffer().borrow_mut().x = 0;
                self.index()?;
            }
            "H" => self.execute(0x88)?,
            "M" => self.reverse_index()?,
            "=" | ">" => {
                self.core.dec_private_modes.borrow_mut().application_keypad = final_byte == "=";
                self.events.borrow_mut().push(TerminalEvent::SyncScrollBar);
            }
            "n" | "}" => self.charset.set_g_level(2),
            "o" | "|" => self.charset.set_g_level(3),
            "~" => self.charset.set_g_level(1),
            _ => {}
        }
        Ok(())
    }
    fn reverse_index(&mut self) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let mut b = active.borrow_mut();
        b.x = b.x.min(self.cols() - 1);
        b.y = b.y.min(self.rows() - 1);
        if b.y == b.scroll_top {
            let top = b.ybase + b.y;
            let height = b.scroll_bottom - b.scroll_top;
            b.lines
                .shift_elements(top as isize, height, 1)
                .map_err(crate::buffer::BufferError::from)?;
            b.lines
                .set(top, Some(b.get_blank_line(&self.erase_attributes(), false)))
                .map_err(crate::buffer::BufferError::from)?;
            self.dirty_row(b.scroll_top);
            self.dirty_row(b.scroll_bottom);
        } else {
            b.y = b.y.saturating_sub(1);
            self.dirty_row(b.y);
        }
        Ok(())
    }
    pub(crate) fn alignment_pattern(&mut self) -> Result<(), TerminalError> {
        self.set_cursor(0, 0)?;
        let cell = crate::buffer::CellData {
            content: (1 << crate::buffer::constants::content::WIDTH_SHIFT) | u32::from(b'E'),
            // DECALN copies packed fg/bg, but not the current extended object.
            attributes: crate::buffer::AttributeData {
                fg: self.attributes.fg,
                bg: self.attributes.bg,
                ..Default::default()
            },
            ..Default::default()
        };
        let active = self.active_buffer();
        let b = active.borrow();
        for y in 0..self.rows() {
            if let Some(row) = b.lines.get(b.ybase + b.y + y) {
                let mut row = row.borrow_mut();
                row.is_wrapped = false;
                row.fill(&cell, false);
            }
        }
        drop(b);
        self.dirty_all();
        self.set_cursor(0, 0)
    }
}
