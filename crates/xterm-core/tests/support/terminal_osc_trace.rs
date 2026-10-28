// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
// Shared actual-engine snapshot protocol for OSC differential and fixture tests.
use std::{cell::RefCell, error::Error, fmt::Write as _, rc::Rc};
use xterm_core::terminal::{ColorRequest, CoreTerminal, TerminalEvent};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub struct Runner {
    terminal: CoreTerminal,
    events: Rc<RefCell<Vec<String>>>,
}
impl Runner {
    pub fn new() -> Result<Self> {
        let mut terminal = CoreTerminal::new(4, 3, 2)?;
        let events = Rc::new(RefCell::new(Vec::new()));
        let event_log = Rc::clone(&events);
        let state = Rc::downgrade(&terminal.state_handle());
        terminal.set_event_handler(Some(Rc::new(move |event| {
            match event {
                TerminalEvent::Color(batch) => event_log.borrow_mut().push(color_batch(&batch)),
                TerminalEvent::TitleChange(title) => {
                    if let Some(state) = state.upgrade() {
                        let state = state.borrow();
                        event_log.borrow_mut().push(format!(
                            "t{}:{}:{}",
                            hex(&title),
                            hex(&state.title),
                            hex(&state.icon_name)
                        ));
                    }
                }
                _ => {}
            }
            Ok(())
        })));
        Ok(Self { terminal, events })
    }
    pub fn step(&mut self, operation: &str) -> Result<String> {
        if operation == "reset" {
            self.terminal.dispose()?;
            *self = Self::new()?;
        } else {
            self.terminal.write_utf16(&decode(operation)?)?;
        }
        let state = self.terminal.state();
        let attrs = &state.attributes;
        let active = if Rc::ptr_eq(&state.active_buffer(), &state.buffers.normal()) {
            "n"
        } else {
            "a"
        };
        let mut out = format!(
            "{active};{};{};{},{},{},{};{};",
            hex(&state.title),
            hex(&state.icon_name),
            attrs.fg,
            attrs.bg,
            attrs.extended.ext(),
            attrs.extended.url_id(),
            self.events.borrow().join("|")
        );
        self.events.borrow_mut().clear();
        for reference in [state.buffers.normal(), state.buffers.alt()] {
            let buffer = reference.borrow();
            write!(
                &mut out,
                "{},{},{},{},{},{}/",
                buffer.x,
                buffer.y,
                buffer.ybase,
                buffer.ydisp,
                buffer.scroll_top,
                buffer.scroll_bottom
            )?;
            for row in 0..buffer.lines.len() {
                let line = buffer.lines.get(row).ok_or("missing trace line")?;
                let line = line.borrow();
                write!(&mut out, "{}[", u8::from(line.is_wrapped))?;
                for column in 0..line.len() {
                    let cell = line.get_cell(column).ok_or("missing trace cell")?;
                    write!(
                        &mut out,
                        "{},{},{},{},{},{}|",
                        cell.content,
                        cell.fg,
                        cell.bg,
                        cell.extended.ext(),
                        cell.extended.url_id(),
                        hex(&cell.get_chars())
                    )?;
                }
                out.push(']');
            }
            out.push('m');
            for marker in buffer.markers.borrow().iter() {
                write!(
                    &mut out,
                    "{}:{}:{},",
                    marker.id(),
                    marker.line_number(),
                    u8::from(marker.is_disposed())
                )?;
            }
            out.push(';');
        }
        for id in state.links.link_ids() {
            if let Some(data) = state.links.get_link_data(id) {
                let data = data.borrow();
                write!(
                    &mut out,
                    "{id}:{}/{}|",
                    data.id.as_deref().map(hex).unwrap_or_else(|| "-".into()),
                    hex(&data.uri)
                )?;
            }
        }
        Ok(out)
    }
}
fn color_batch(batch: &[ColorRequest]) -> String {
    let mut out = "c[".to_owned();
    for request in batch {
        match request {
            ColorRequest::Report { index } => write!(&mut out, "r{},", index.protocol_index()),
            ColorRequest::Set {
                index,
                color: [r, g, b],
            } => write!(&mut out, "s{}:{r}.{g}.{b},", index.protocol_index()),
            ColorRequest::Restore { index } => write!(
                &mut out,
                "x{},",
                index
                    .map(|index| index.protocol_index().to_string())
                    .unwrap_or_else(|| "*".into())
            ),
        }
        .expect("writing to a String cannot fail");
    }
    out.push(']');
    out
}
fn decode(text: &str) -> Result<Vec<u16>> {
    if text == "_" {
        return Ok(Vec::new());
    }
    if !text.is_ascii() || text.len() % 4 != 0 {
        return Err("invalid UTF16 operation".into());
    }
    (0..text.len())
        .step_by(4)
        .map(|i| u16::from_str_radix(&text[i..i + 4], 16).map_err(Into::into))
        .collect()
}
fn hex(units: &[u16]) -> String {
    if units.is_empty() {
        return "_".into();
    }
    let mut out = String::new();
    for unit in units {
        write!(&mut out, "{unit:04x}").expect("writing to a String cannot fail");
    }
    out
}
