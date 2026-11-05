// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
//! Native queued parsing releases engine ownership at parser action boundaries.
//! The direct mutable CoreTerminal entrypoint stays available for embeddings
//! without shared callbacks. Both paths use the same decoders and parser steps.
use super::{
    dispatch_events, parse::next_range, CoreTerminal, DecodeBufferOverflow, TerminalError,
    MAX_PARSE_BUFFER_LENGTH,
};
use crate::{
    input::parse_source::ParseInputSource,
    parser::{
        escape::{ParseError, ParseStep},
        handler::HandlerResult,
    },
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

struct Depth(Rc<Cell<usize>>);
impl Drop for Depth {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}
struct BellDelivery(Rc<Cell<bool>>, bool);
impl Drop for BellDelivery {
    fn drop(&mut self) {
        self.0.set(self.1);
    }
}

impl CoreTerminal {
    pub(crate) fn queued_source(&self) -> Result<Rc<dyn ParseInputSource>, TerminalError> {
        self.source
            .as_ref()
            .cloned()
            .ok_or(TerminalError::InvalidParameter(
                "parse input source is unavailable",
            ))
    }
    pub(crate) fn write_source_shared(
        engine: &RefCell<Self>,
        source: Rc<dyn ParseInputSource>,
        previous: Option<bool>,
    ) -> Result<(), TerminalError> {
        let (state, cursor_start, was_paused, source, saved_position, saved_length) = {
            let mut core = engine
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?;
            core.ensure_live()?;
            let was_paused = core.awaiting;
            if previous.is_some() != was_paused {
                return Err(if was_paused {
                    TerminalError::Pending
                } else {
                    ParseError::ImproperContinuation.into()
                });
            }
            let cursor_start = if was_paused {
                core.cursor_start
            } else {
                let state = core.state.borrow();
                let active = state.active_buffer();
                let active = active.borrow();
                (active.x, active.y)
            };
            let source = if was_paused {
                core.source
                    .as_ref()
                    .cloned()
                    .ok_or(TerminalError::InvalidParameter(
                        "parse input source is unavailable",
                    ))?
            } else {
                source
            };
            let result = (
                Rc::clone(&core.state),
                cursor_start,
                was_paused,
                source,
                core.source_position,
                core.decoded_length,
            );
            core.pending = None;
            core.awaiting = false;
            result
        };
        let depth = Rc::clone(&state.borrow().shared_parse_depth);
        depth.set(depth.get() + 1);
        let _depth = Depth(depth);
        let mut start = 0;
        if was_paused {
            parse_current(
                engine,
                saved_length,
                previous,
                &source,
                saved_position,
                cursor_start,
            )?;
            if source.len()? > MAX_PARSE_BUFFER_LENGTH {
                start = next_range(saved_position)?;
            }
        }
        let logger = Rc::clone(&state.borrow().logger);
        super::diagnostics::log_source(&logger, source.as_ref())?;
        {
            let mut core = engine
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?;
            if core.decoded.borrow().len() < source.len()?
                && core.decoded.borrow().len() < MAX_PARSE_BUFFER_LENGTH
            {
                core.decoded =
                    Rc::new(RefCell::new(vec![
                        0;
                        source.len()?.min(MAX_PARSE_BUFFER_LENGTH)
                    ]));
            }
            if !was_paused {
                let active = state.borrow().active_buffer();
                let y = active.borrow().y;
                state.borrow_mut().dirty = Some((y, y));
            }
        }
        if source.len()? > MAX_PARSE_BUFFER_LENGTH {
            let mut position = start;
            while position < source.len()? {
                let candidate = next_range(position)?;
                let end = if candidate < source.len()? {
                    candidate
                } else {
                    source.len()?
                };
                let length = {
                    let mut core = engine
                        .try_borrow_mut()
                        .map_err(|_| TerminalError::Borrowed)?;
                    core.decode_range(source.as_ref(), position..end)?;
                    core.decoded_length
                };
                parse_current(engine, length, None, &source, position, cursor_start)?;
                position = candidate;
            }
        } else if !was_paused {
            let length = {
                let mut core = engine
                    .try_borrow_mut()
                    .map_err(|_| TerminalError::Borrowed)?;
                core.decode_all(source.as_ref())?;
                core.decoded_length
            };
            parse_current(engine, length, None, &source, 0, cursor_start)?;
        }
        {
            let mut core = engine
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?;
            core.enqueue_input_end(cursor_start)?;
            if !core.awaiting {
                core.source = None;
            }
        }
        dispatch_events(&state)
    }
}

fn parse_current(
    engine: &RefCell<CoreTerminal>,
    length: usize,
    previous: Option<bool>,
    source: &Rc<dyn ParseInputSource>,
    position: usize,
    cursor_start: (usize, usize),
) -> Result<(), TerminalError> {
    // A nested decode mutates this same allocation unless it grows the buffer.
    // In that case the active outer parse retains its original typed-array identity.
    let (storage, state, observer) = {
        let core = engine.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        (
            Rc::clone(&core.decoded),
            Rc::clone(&core.state),
            core.parse_observer.clone(),
        )
    };
    if let Some(observer) = observer {
        let snapshot = storage.borrow().clone();
        observer(&snapshot, length)?;
    }
    let (mut index, pending) = {
        let mut core = engine
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        let data = storage.borrow();
        let result = core
            .parser
            .begin_steps(&data[..length.min(data.len())], previous);
        check_error(&state, result)?
    };
    if let Some(pending) = pending {
        return pause(engine, pending, source, position, length, cursor_start);
    }
    loop {
        let step = {
            let mut core = engine
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?;
            let flag = Rc::clone(&state.borrow().defer_bell);
            let previous = flag.replace(true);
            let _delivery = BellDelivery(flag, previous);
            let data = storage.borrow();
            check_error(
                &state,
                core.parser
                    .parse_step(&data[..length.min(data.len())], index),
            )?
        };
        match step {
            ParseStep::Continue { next, executed } => {
                dispatch_events(&state)?;
                engine
                    .try_borrow()
                    .map_err(|_| TerminalError::Borrowed)?
                    .parser
                    .finish_step(executed);
                index = next;
            }
            ParseStep::Pending(pending) => {
                return pause(engine, pending, source, position, length, cursor_start)
            }
            ParseStep::Complete => break,
        }
    }
    if length > storage.borrow().len() {
        return Err(TerminalError::Event(crate::event::EmitError::callback(
            DecodeBufferOverflow {
                decoded_length: length,
                buffer_length: storage.borrow().len(),
            },
        )));
    }
    Ok(())
}

fn check_error<T>(
    state: &Rc<RefCell<super::TerminalState>>,
    result: Result<T, ParseError>,
) -> Result<T, TerminalError> {
    if let Some(error) = state.borrow_mut().error.take() {
        return Err(error);
    }
    result.map_err(Into::into)
}

fn pause(
    engine: &RefCell<CoreTerminal>,
    pending: HandlerResult,
    source: &Rc<dyn ParseInputSource>,
    position: usize,
    length: usize,
    cursor_start: (usize, usize),
) -> Result<(), TerminalError> {
    let state = {
        let mut core = engine
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        core.source = Some(Rc::clone(source));
        core.source_position = position;
        core.decoded_length = length;
        core.cursor_start = cursor_start;
        core.pending = Some(pending);
        core.awaiting = true;
        Rc::clone(&core.state)
    };
    dispatch_events(&state)?;
    Err(TerminalError::Pending)
}
