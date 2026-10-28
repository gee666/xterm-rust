// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! Typed service construction and ownership, separate from streaming execution.
use super::{controls, register, CoreTerminal, TerminalError, TerminalEvent, TerminalState};
use crate::{
    buffer::BufferService,
    charset::CharsetService,
    core_service::{BufferPosition, CoreService},
    input::text_decoder::{StringToUtf32, Utf8ToUtf32},
    log_service::{LogError, LogRecord, LogService, LogSink},
    mouse_state::MouseStateService,
    options::{OptionValue, OptionsService},
    parser::escape::EscapeSequenceParser,
    unicode_service::UnicodeService,
    unicode_v6::UnicodeV6,
};
use std::{cell::RefCell, rc::Rc};
struct MemoryLog(Rc<RefCell<Vec<LogRecord>>>);
impl LogSink for MemoryLog {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        self.0.borrow_mut().push(record);
        Ok(())
    }
}

impl CoreTerminal {
    pub fn new(cols: usize, rows: usize, scrollback: usize) -> Result<Self, TerminalError> {
        if cols > i32::MAX as usize || rows > i32::MAX as usize || scrollback > i32::MAX as usize {
            return Err(TerminalError::InvalidDimensions);
        }
        let options = Rc::new(OptionsService::new(
            false,
            [
                ("cols", OptionValue::Number(cols.max(2) as f64)),
                ("rows", OptionValue::Number(rows.max(1) as f64)),
                ("scrollback", OptionValue::Number(scrollback as f64)),
            ],
        ));
        Self::with_options(options)
    }
    pub fn with_options(options: Rc<OptionsService>) -> Result<Self, TerminalError> {
        let service = Rc::new(
            BufferService::new(Rc::clone(&options))
                .map_err(|e| TerminalError::Buffer(e.to_string()))?,
        );
        let logs = Rc::new(RefCell::new(Vec::new()));
        let logger = Rc::new(LogService::new(
            Rc::clone(&options),
            Rc::new(MemoryLog(Rc::clone(&logs))),
        )?);
        let position_service = Rc::clone(&service);
        let core = Rc::new(CoreService::new(
            Rc::clone(&options),
            Rc::clone(&logger),
            move || {
                let active = position_service.buffer();
                let b = active.borrow();
                BufferPosition {
                    ybase: b.ybase,
                    ydisp: b.ydisp,
                }
            },
        )?);
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut listeners = Vec::new();
        let scrolling = Rc::downgrade(&service);
        listeners.push(core.on_request_scroll_to_bottom().try_subscribe(move |_| {
            let Some(service) = scrolling.upgrade() else {
                return Ok(());
            };
            let active = service.buffer();
            let b = active.borrow();
            let difference = b.ybase.saturating_sub(b.ydisp) as i64;
            drop(b);
            service
                .scroll_lines(difference, false)
                .map_err(crate::event::EmitError::callback)
        }));
        let queue = Rc::clone(&events);
        listeners.push(
            core.on_data()
                .subscribe(move |data| queue.borrow_mut().push(TerminalEvent::Data(data.clone()))),
        );
        let queue = Rc::clone(&events);
        listeners.push(
            core.on_binary().subscribe(move |data| {
                queue.borrow_mut().push(TerminalEvent::Binary(data.clone()))
            }),
        );
        let queue = Rc::clone(&events);
        listeners.push(
            service.on_scroll().subscribe(move |position| {
                queue.borrow_mut().push(TerminalEvent::Scroll(*position))
            }),
        );
        let queue = Rc::clone(&events);
        listeners.push(service.on_resize().subscribe(move |event| {
            queue.borrow_mut().push(TerminalEvent::Resize {
                cols: event.cols,
                rows: event.rows,
            })
        }));
        let windows_wrapping = Rc::new(std::cell::Cell::new(controls::windows_heuristics_enabled(
            &options.get("windowsPty")?,
        )?));
        let wrapping = Rc::downgrade(&windows_wrapping);
        let option_ref = Rc::downgrade(&options);
        listeners.push(options.on_option_change().try_subscribe(move |changed| {
            if *changed != "windowsPty" {
                return Ok(());
            }
            if let (Some(wrapping), Some(options)) = (wrapping.upgrade(), option_ref.upgrade()) {
                wrapping.set(
                    controls::windows_heuristics_enabled(
                        &options
                            .get("windowsPty")
                            .map_err(crate::event::EmitError::callback)?,
                    )
                    .map_err(crate::event::EmitError::callback)?,
                );
            }
            Ok(())
        }));
        let unicode = Rc::new(UnicodeService::new());
        unicode.register(Rc::new(UnicodeV6::new()))?;
        let links = Rc::new(crate::osc_link_service::OscLinkService::new(Rc::clone(
            &service,
        )));
        let state = Rc::new(RefCell::new(TerminalState {
            buffer_service: service,
            links,
            options,
            core,
            unicode,
            logger,
            mouse: Rc::new(MouseStateService::new()),
            charset: CharsetService::default(),
            attributes: Default::default(),
            title: Vec::new(),
            icon_name: Vec::new(),
            title_stack: Vec::new(),
            icon_stack: Vec::new(),
            events,
            event_handler: Rc::new(RefCell::new(None)),
            error: None,
            dirty: None,
            preceding_join_state: Rc::default(),
            windows_wrapping,
            core_actions: Default::default(),
        }));
        let mut parser = EscapeSequenceParser::new();
        state.borrow_mut().preceding_join_state = Rc::clone(&parser.preceding_join_state);
        register::register(&mut parser, &state)?;
        Ok(Self {
            state,
            parser,
            utf16: StringToUtf32::new(),
            utf8: Utf8ToUtf32::new(),
            decoded: Vec::new(),
            pending: None,
            awaiting: false,
            cursor_start: (0, 0),
            disposed: false,
            listeners,
            logs,
        })
    }
}
