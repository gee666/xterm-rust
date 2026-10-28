// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! Source print-call snapshots with callback boundaries outside state borrows.
use super::{TerminalError, TerminalEvent, TerminalState};
use crate::{
    buffer::{AttributeData, Buffer, BufferLine, BufferService},
    charset::Charset,
    input::text_decoder::string_from_code_point,
    unicode::UnicodeCharProperties,
    unicode_service::UnicodeService,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[path = "print_render.rs"]
mod render;
#[cfg(test)]
#[path = "print_tests.rs"]
mod tests;

type RcLine = Rc<RefCell<BufferLine>>;
type AttributeIdentity = Rc<RefCell<Option<AttributeData>>>;

struct PrintRun {
    shared: Rc<RefCell<TerminalState>>,
    service: Rc<BufferService>,
    unicode: Rc<UnicodeService>,
    charset: Option<Charset>,
    attributes: AttributeIdentity,
    row: RcLine,
    cols: usize,
    wraparound: bool,
    insert: bool,
    a11y: bool,
    info: UnicodeCharProperties,
}

pub(super) fn print(
    shared: &Rc<RefCell<TerminalState>>,
    data: &[u32],
    preceding: &Cell<u32>,
) -> Result<(), TerminalError> {
    let Some(mut run) = PrintRun::new(shared, data, preceding)? else {
        return Ok(());
    };
    for &original in data {
        if original == 0xad {
            continue;
        }
        let code = if original < 127 {
            match &run.charset {
                Some(map) => map.get_first(original as u16)?.map_or(original, u32::from),
                None => original,
            }
        } else {
            original
        };
        // Unicode providers can themselves call back into the terminal. No
        // terminal, buffer, attribute or row borrow crosses their invocation.
        let current = run.unicode.char_properties(code, run.info)?;
        let old_width = if current.should_join() {
            usize::from(run.info.width())
        } else {
            0
        };
        run.info = current;
        if run.a11y {
            {
                let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
                state
                    .events
                    .borrow_mut()
                    .push(TerminalEvent::AccessibilityChar(string_from_code_point(
                        code,
                    )));
            }
            super::dispatch_events(shared)?;
        }
        run.track_link()?;
        if !run.render(code, current, old_width)? {
            return Ok(());
        }
    }
    preceding.set(run.info.raw());
    run.finish(!data.is_empty())
}

impl PrintRun {
    fn new(
        shared: &Rc<RefCell<TerminalState>>,
        data: &[u32],
        preceding: &Cell<u32>,
    ) -> Result<Option<Self>, TerminalError> {
        let mut state = shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        let active = state.active_buffer();
        let buffer = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        let index = buffer
            .ybase
            .checked_add(buffer.y)
            .ok_or(TerminalError::InvalidDimensions)?;
        // Upstream intentionally returns when async resizing removed this row.
        let Some(row) = buffer.lines.get(index) else {
            return Ok(None);
        };
        let run = Self {
            shared: Rc::clone(shared),
            service: Rc::clone(&state.buffer_service),
            unicode: Rc::clone(&state.unicode),
            charset: state.charset.charset.clone(),
            attributes: Rc::clone(&state.attributes_identity),
            row,
            cols: state.cols(),
            wraparound: state.core.dec_private_modes.borrow().wraparound,
            insert: state.core.modes.borrow().insert_mode,
            a11y: state.option_bool("screenReaderMode")?,
            info: UnicodeCharProperties::from_raw(preceding.get()),
        };
        state.dirty_row(buffer.y);
        let x = buffer.x;
        drop(buffer);
        drop(state);
        if x > 0
            && !data.is_empty()
            && run
                .row
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .get_width(x - 1)
                == Some(2)
        {
            let attrs = run.attrs()?;
            run.row
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?
                .set_cell_from_codepoint(x - 1, 0, 1, &attrs);
        }
        Ok(Some(run))
    }

    fn active(&self) -> Rc<RefCell<Buffer>> {
        self.service.buffers.active()
    }

    fn row_for_active(&self) -> Result<Option<RcLine>, TerminalError> {
        let active = self.active();
        let buffer = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        let index = buffer
            .ybase
            .checked_add(buffer.y)
            .ok_or(TerminalError::InvalidDimensions)?;
        Ok(buffer.lines.get(index))
    }

    fn attrs(&self) -> Result<AttributeData, TerminalError> {
        // ExtendedAttrs::clone intentionally drops custom payload. Printing
        // shares the source attribute object's extended data instead.
        super::sgr_dispatch::with_target(&self.shared, &self.attributes, |attrs| AttributeData {
            fg: attrs.fg,
            bg: attrs.bg,
            extended: attrs.extended.share(),
        })
    }

    fn track_link(&self) -> Result<(), TerminalError> {
        let (link_id, links) = {
            let state = self
                .shared
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?;
            (state.attributes.extended.url_id(), Rc::clone(&state.links))
        };
        if link_id != 0 {
            let active = self.active();
            let line = {
                let buffer = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
                buffer
                    .ybase
                    .checked_add(buffer.y)
                    .ok_or(TerminalError::InvalidDimensions)?
            };
            links.add_line_to_link(
                link_id,
                i64::try_from(line).map_err(|_| TerminalError::InvalidDimensions)?,
            )?;
        }
        Ok(())
    }

    fn finish(&self, nonempty: bool) -> Result<(), TerminalError> {
        let active = self.active();
        let buffer = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        if buffer.x < self.cols && nonempty {
            let fix = {
                let row = self.row.try_borrow().map_err(|_| TerminalError::Borrowed)?;
                row.get_width(buffer.x) == Some(0) && row.has_content(buffer.x) == Some(0)
            };
            if fix {
                let attrs = self.attrs()?;
                self.row
                    .try_borrow_mut()
                    .map_err(|_| TerminalError::Borrowed)?
                    .set_cell_from_codepoint(buffer.x, 0, 1, &attrs);
            }
        }
        self.shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?
            .dirty_row(buffer.y);
        Ok(())
    }
}
