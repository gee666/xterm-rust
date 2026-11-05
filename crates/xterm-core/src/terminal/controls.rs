// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{TerminalError, TerminalEvent, TerminalState};
use std::{cell::RefCell, rc::Rc};

/// Direct InputHandler line feed for parser handlers and addon hosts. No parser reentry.
pub fn line_feed_state(shared: &Rc<RefCell<TerminalState>>) -> Result<(), TerminalError> {
    let should_scroll = {
        let mut state = shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        let active = state.active_buffer();
        let mut b = active
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        state.dirty_row(b.y);
        if state.option_bool("convertEol")? {
            b.x = 0;
        }
        b.y += 1;
        let scroll = b.y == b.scroll_bottom + 1;
        if scroll {
            b.y -= 1;
        } else if b.y >= state.rows() {
            b.y = state.rows() - 1;
        } else {
            b.set_wrapped(b.ybase + b.y, false)?;
        }
        scroll
    };
    if should_scroll {
        scroll_state(shared, false)?;
    }
    {
        let mut state = shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        let active = state.active_buffer();
        let mut b = active
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        if b.x >= state.cols() {
            b.x -= 1;
        }
        state.dirty_row(b.y);
        drop(b);
    }
    let line_feed = Rc::clone(&shared.borrow().line_feed);
    line_feed.fire(&()).map_err(TerminalError::Event)?;
    shared
        .borrow()
        .events
        .borrow_mut()
        .push(TerminalEvent::LineFeed);
    super::dispatch_events(shared)
}

pub(super) fn scroll_state(
    shared: &Rc<RefCell<TerminalState>>,
    wrapped: bool,
) -> Result<(), TerminalError> {
    let (service, attrs) = {
        let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        (Rc::clone(&state.buffer_service), state.erase_attributes())
    };
    service.scroll(&attrs, wrapped)?;
    super::dispatch_events(shared)?;
    let mut state = shared
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?;
    let active = state.active_buffer();
    let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
    state.dirty_row(b.scroll_top);
    state.dirty_row(b.scroll_bottom);
    Ok(())
}

/// IND and NEL reload the active buffer after synchronous scroll observers.
/// In particular, a marker or scroll listener may reset the terminal in scroll.
/// Keep both TerminalState and Buffer borrows out of that callback boundary.
pub(super) fn index_state(
    shared: &Rc<RefCell<TerminalState>>,
    next_line: bool,
) -> Result<(), TerminalError> {
    let scroll = {
        let mut state = shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        let max_col = state.cols() - 1;
        if next_line {
            state
                .active_buffer()
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?
                .x = 0;
        }
        state.restrict_csi_cursor(max_col)?;
        let active = state.active_buffer();
        let mut buffer = active
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        buffer.y += 1;
        let scroll = buffer.y == buffer.scroll_bottom + 1;
        if scroll || buffer.y >= state.rows() {
            buffer.y -= 1;
        }
        scroll
    };
    if scroll {
        scroll_state(shared, false)?;
    }
    let mut state = shared
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?;
    let max_col = state.cols() - 1;
    state.restrict_csi_cursor(max_col)
}

impl TerminalState {
    pub(crate) fn execute(&mut self, code: u32) -> Result<(), TerminalError> {
        match code {
            0x07 => self.events.borrow_mut().push(TerminalEvent::Bell),
            0x0d => self.active_buffer().borrow_mut().x = 0,
            0x08 => self.backspace()?,
            0x09 => self.tab()?,
            0x0e => self.charset.set_g_level(1),
            0x0f => self.charset.set_g_level(0),
            0x88 => {
                let b = self.active_buffer();
                let mut b = b.borrow_mut();
                let x = b.x;
                b.tabs.insert(x);
            }
            _ => {}
        }
        Ok(())
    }
    fn tab(&mut self) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let distance = {
            let mut b = active.borrow_mut();
            if b.x >= self.cols() {
                return Ok(());
            }
            let original = b.x;
            b.x = b.next_stop(None);
            b.x - original
        };
        if self.option_bool("screenReaderMode")? {
            self.events
                .borrow_mut()
                .push(TerminalEvent::AccessibilityTab(distance));
        }
        Ok(())
    }
    fn backspace(&mut self) -> Result<(), TerminalError> {
        let reverse = self.core.dec_private_modes.borrow().reverse_wraparound;
        self.restrict_csi_cursor(if reverse {
            self.cols()
        } else {
            self.cols() - 1
        })?;
        let active = self.active_buffer();
        let mut b = active.borrow_mut();
        if b.x > 0 {
            b.x -= 1;
        } else if reverse && b.y > b.scroll_top && b.y <= b.scroll_bottom {
            let line = b
                .lines
                .get(b.ybase + b.y)
                .ok_or(TerminalError::MissingLine)?;
            if line.borrow().is_wrapped {
                line.borrow_mut().is_wrapped = false;
                b.y -= 1;
                b.x = self.cols() - 1;
                let previous = b
                    .lines
                    .get(b.ybase + b.y)
                    .ok_or(TerminalError::MissingLine)?;
                let previous = previous.borrow();
                if previous.has_width(b.x) != Some(0) && previous.has_content(b.x) == Some(0) {
                    b.x -= 1;
                }
            }
        }
        drop(b);
        if reverse {
            self.restrict_csi_cursor(self.cols() - 1)?;
        }
        Ok(())
    }
}
