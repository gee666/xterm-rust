// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{CoreTerminal, TerminalError, TerminalEvent, TerminalState};
use crate::buffer::AttributeData;

impl CoreTerminal {
    /// Run the actual InputHandler LF operation without writing another parser chunk.
    pub fn line_feed(&mut self) -> Result<(), TerminalError> {
        self.ensure_live()?;
        super::line_feed_state(&self.state)
    }
    /// Mark a viewport row for the next parsed/render batch, as addon handlers do.
    pub fn mark_dirty(&mut self, row: usize) -> Result<(), TerminalError> {
        self.ensure_live()?;
        mark_dirty_state(&self.state, row)
    }
    pub fn resize(&mut self, cols: usize, rows: usize) -> Result<(), TerminalError> {
        self.ensure_live()?;
        if self.awaiting {
            return Err(TerminalError::Pending);
        }
        if cols > i32::MAX as usize || rows > i32::MAX as usize {
            return Err(TerminalError::InvalidDimensions);
        }
        let service = std::rc::Rc::clone(
            &self
                .state
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .buffer_service,
        );
        service.resize(cols.max(2), rows.max(1))?;
        super::dispatch_events(&self.state)
    }
    pub fn scroll_lines(&mut self, lines: i64) -> Result<(), TerminalError> {
        self.ensure_live()?;
        let service = std::rc::Rc::clone(
            &self
                .state
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .buffer_service,
        );
        let old = service.buffer().borrow().ydisp;
        service.scroll_lines(lines, false)?;
        super::dispatch_events(&self.state)?;
        if service.buffer().borrow().ydisp != old {
            let mut state = self
                .state
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?;
            let active = state.active_buffer();
            let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
            state.dirty_row(b.scroll_top);
            state.dirty_row(b.scroll_bottom);
        }
        Ok(())
    }
    pub fn scroll_pages(&mut self, pages: i64) -> Result<(), TerminalError> {
        let lines = pages.saturating_mul((self.rows() - 1) as i64);
        self.scroll_lines(lines)
    }
    pub fn scroll_to_top(&mut self) -> Result<(), TerminalError> {
        self.scroll_to_line(0)
    }
    pub fn scroll_to_bottom(&mut self) -> Result<(), TerminalError> {
        let ybase = self.state.borrow().active_buffer().borrow().ybase;
        self.scroll_to_line(ybase)
    }
    pub fn scroll_to_line(&mut self, line: usize) -> Result<(), TerminalError> {
        let ydisp = self.state.borrow().active_buffer().borrow().ydisp;
        let difference = line as i128 - ydisp as i128;
        self.scroll_lines(difference.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64)
    }
    pub fn reset(&mut self) -> Result<(), TerminalError> {
        self.ensure_live()?;
        // Public reset is out-of-band and deliberately retains parser/decoder state.
        reset_state_handle(&self.state)
    }
    pub fn clear(&mut self) -> Result<(), TerminalError> {
        self.ensure_live()?;
        clear_state_handle(&self.state)
    }
}
/// The native wrapper may clear again from a marker callback while the outer
/// engine is already borrowed. Both entry points share these callback phases.
/// Host entry points must enforce their disposed-terminal contract first.
pub fn clear_state_handle(
    shared: &std::rc::Rc<std::cell::RefCell<TerminalState>>,
) -> Result<(), TerminalError> {
    let active = shared
        .try_borrow()
        .map_err(|_| TerminalError::Borrowed)?
        .active_buffer();
    crate::buffer::Buffer::clear_all_markers_shared(&active)?;
    // A marker callback can reset or switch buffers before row selection.
    let (active, rows) = {
        let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        (state.active_buffer(), state.rows())
    };
    let mut buffer = active
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?;
    let row = buffer
        .lines
        .get(buffer.ybase + buffer.y)
        .ok_or(TerminalError::MissingLine)?;
    buffer
        .lines
        .set(0, Some(row))
        .map_err(crate::buffer::BufferError::from)?;
    buffer.lines.set_length(1);
    buffer.ydisp = 0;
    buffer.ybase = 0;
    buffer.y = 0;
    for _ in 1..rows {
        buffer
            .lines
            .push(buffer.get_blank_line(&AttributeData::default(), false))
            .map_err(crate::buffer::BufferError::from)?;
    }
    drop(buffer);
    shared
        .try_borrow()
        .map_err(|_| TerminalError::Borrowed)?
        .events
        .borrow_mut()
        .push(TerminalEvent::Scroll(0));
    super::dispatch_events(shared)
}
/// State-handle form for addon callbacks invoked while the outer parser is borrowed.
pub fn mark_dirty_state(
    shared: &std::rc::Rc<std::cell::RefCell<TerminalState>>,
    row: usize,
) -> Result<(), TerminalError> {
    shared
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?
        .dirty_row(row);
    Ok(())
}
/// Common reset with callback phases separated from terminal-state borrows.
pub fn reset_state_handle(
    shared: &std::rc::Rc<std::cell::RefCell<TerminalState>>,
) -> Result<(), TerminalError> {
    let (service, options, wrapping) = {
        let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        (
            std::rc::Rc::clone(&state.buffer_service),
            std::rc::Rc::clone(&state.options),
            std::rc::Rc::clone(&state.windows_wrapping),
        )
    };
    // The public terminal bootstrap carries live dimensions into constructor-only
    // raw options before resetting the common services.
    options.set(
        "rows",
        crate::options::OptionValue::Number(service.rows() as f64),
    )?;
    options.set(
        "cols",
        crate::options::OptionValue::Number(service.cols() as f64),
    )?;
    wrapping.set(super::controls::windows_heuristics_enabled(
        &options.get("windowsPty")?,
    )?);
    shared
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?
        .replace_attributes(AttributeData::default());
    service.reset()?;
    let (core, mouse) = {
        let mut state = shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        state.charset.reset();
        state.dirty_all();
        (
            std::rc::Rc::clone(&state.core),
            std::rc::Rc::clone(&state.mouse),
        )
    };
    core.reset();
    mouse.reset()?;
    super::dispatch_events(shared)
}
/// In-band RIS/DECCOLM reset. Host observers receive the notification after the
/// terminal's built-in reset, matching the first registered source listener.
pub fn request_reset_state(
    shared: &std::rc::Rc<std::cell::RefCell<TerminalState>>,
) -> Result<(), TerminalError> {
    let slot = {
        let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        std::rc::Rc::clone(&state.reset_request_handler)
    };
    let handler = slot.borrow().clone();
    if let Some(handler) = handler {
        handler().map_err(TerminalError::Event)?;
    } else {
        reset_state_handle(shared)?;
    }
    shared
        .try_borrow()
        .map_err(|_| TerminalError::Borrowed)?
        .events
        .borrow_mut()
        .push(TerminalEvent::RequestReset);
    super::dispatch_events(shared)
}
pub(super) fn clear_reset_request_handler(
    shared: &std::rc::Rc<std::cell::RefCell<TerminalState>>,
) -> Result<(), TerminalError> {
    let slot = {
        let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        std::rc::Rc::clone(&state.reset_request_handler)
    };
    let previous = slot.borrow_mut().take();
    drop(previous);
    Ok(())
}
impl TerminalState {
    pub(crate) fn set_cursor(&mut self, x: i64, y: i64) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let mut b = active.borrow_mut();
        self.dirty_row(b.y);
        let origin = self.core.dec_private_modes.borrow().origin;
        b.x = x.clamp(0, (self.cols() - 1) as i64) as usize;
        let (min, max, y) = if origin {
            (
                b.scroll_top,
                b.scroll_bottom,
                y.saturating_add(b.scroll_top as i64),
            )
        } else {
            (0, self.rows() - 1, y)
        };
        b.y = y.clamp(min as i64, max as i64) as usize;
        self.dirty_row(b.y);
        Ok(())
    }
    pub(crate) fn save_cursor(&mut self) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let mut b = active.borrow_mut();
        b.saved_x = b.x;
        b.saved_y = i64::try_from(
            b.ybase
                .checked_add(b.y)
                .ok_or(TerminalError::InvalidDimensions)?,
        )
        .map_err(|_| TerminalError::InvalidDimensions)?;
        b.saved_cur_attr_data.fg = self.attributes.fg;
        b.saved_cur_attr_data.bg = self.attributes.bg;
        b.saved_charset = self.charset.charset.clone();
        b.saved_charsets = self.charset.charsets().to_vec();
        b.saved_glevel = self.charset.glevel;
        let modes = self.core.dec_private_modes.borrow();
        b.saved_origin_mode = modes.origin;
        b.saved_wraparound_mode = modes.wraparound;
        Ok(())
    }
    pub(crate) fn restore_cursor(&mut self) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let b = active.borrow();
        let x = b.saved_x;
        let y = b
            .saved_y
            .saturating_sub(i64::try_from(b.ybase).map_err(|_| TerminalError::InvalidDimensions)?)
            .max(0);
        self.attributes.fg = b.saved_cur_attr_data.fg;
        self.attributes.bg = b.saved_cur_attr_data.bg;
        for (level, charset) in b.saved_charsets.iter().enumerate() {
            self.charset.set_g_charset(level, charset.clone())?;
        }
        self.charset.set_g_level(b.saved_glevel);
        {
            let mut modes = self.core.dec_private_modes.borrow_mut();
            modes.origin = b.saved_origin_mode;
            modes.wraparound = b.saved_wraparound_mode;
        }
        drop(b);
        let mut b = active.borrow_mut();
        b.x = x.min(self.cols() - 1);
        b.y = if self.core.dec_private_modes.borrow().origin {
            y.clamp(b.scroll_top as i64, b.scroll_bottom as i64) as usize
        } else {
            y.min((self.rows() - 1) as i64) as usize
        };
        self.dirty_row(b.y);
        Ok(())
    }
}
impl From<crate::mouse_state::MouseStateError> for TerminalError {
    fn from(e: crate::mouse_state::MouseStateError) -> Self {
        Self::Mouse(e)
    }
}
impl From<crate::log_service::LogError> for TerminalError {
    fn from(e: crate::log_service::LogError) -> Self {
        Self::Log(e)
    }
}
