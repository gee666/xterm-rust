// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Bounded source-unit decoding. No source chunk borrow crosses parser callbacks.
use super::{dispatch::dispatch_events, CoreTerminal, TerminalError, TerminalEvent};
use crate::{
    event::EmitError,
    input::parse_source::{OwnedParseInput, ParseChunk, ParseInputSource},
};
use std::{fmt, rc::Rc};

pub const INITIAL_PARSE_BUFFER_LENGTH: usize = 4096;
pub const MAX_PARSE_BUFFER_LENGTH: usize = 131_072;
/// Inspect the actual fixed storage and decoder's logical length immediately
/// before each parser invocation, including continuations. This replaces JS
/// monkey-patching for diagnostics without replacing the real parser.
pub type ParseBufferObserver = Rc<dyn Fn(&[u32], usize) -> Result<(), TerminalError>>;

/// The reference can return a logical length beyond its fixed typed array after
/// a split unpaired high surrogate. Rust reports this rather than reading out
/// of bounds or copying a possible reference nontermination bug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeBufferOverflow {
    pub decoded_length: usize,
    pub buffer_length: usize,
}
impl fmt::Display for DecodeBufferOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "decoded input length {} exceeds fixed parse buffer {}",
            self.decoded_length, self.buffer_length
        )
    }
}
impl std::error::Error for DecodeBufferOverflow {}

impl CoreTerminal {
    pub fn set_parse_buffer_observer(&mut self, observer: Option<ParseBufferObserver>) {
        self.parse_observer = observer;
    }
    pub fn write_utf16(&mut self, data: &[u16]) -> Result<(), TerminalError> {
        self.write_source(Rc::new(OwnedParseInput::Text(data.to_vec())))
    }
    pub fn write_bytes(&mut self, data: &[u8]) -> Result<(), TerminalError> {
        self.write_source(Rc::new(OwnedParseInput::Bytes(data.to_vec())))
    }
    pub fn write(&mut self, data: &str) -> Result<(), TerminalError> {
        self.write_source(Rc::new(OwnedParseInput::Text(
            data.encode_utf16().collect(),
        )))
    }
    /// A live host view remains owned across pauses and is read a bounded range
    /// at a time, only after diagnostics and preceding handlers have finished.
    pub fn write_source(&mut self, source: Rc<dyn ParseInputSource>) -> Result<(), TerminalError> {
        self.ensure_live()?;
        if self.awaiting {
            return Err(TerminalError::Pending);
        }
        let state = self.state.borrow();
        let active = state.active_buffer();
        let active = active.borrow();
        self.cursor_start = (active.x, active.y);
        drop(active);
        drop(state);
        self.source = Some(source);
        self.source_position = 0;
        self.decoded_length = 0;
        self.drive_input(false)
    }
    /// Take and await the handler future, then resume the same decoded range.
    pub fn take_pending(&mut self) -> Option<crate::parser::handler::HandlerResult> {
        self.pending.take()
    }
    pub fn take_cleanup(&mut self) -> Vec<crate::parser::handler::HandlerResult> {
        self.parser.take_cleanup()
    }
    pub fn resume(&mut self, result: bool) -> Result<(), TerminalError> {
        self.ensure_live()?;
        if !self.awaiting {
            return Err(TerminalError::Parse(
                crate::parser::escape::ParseError::ImproperContinuation,
            ));
        }
        self.pending = None;
        self.awaiting = false;
        self.parse_current(Some(result))?;
        self.drive_input(true)
    }
    fn source_handle(&self) -> Result<Rc<dyn ParseInputSource>, TerminalError> {
        self.source
            .as_ref()
            .map(Rc::clone)
            .ok_or(TerminalError::InvalidParameter(
                "parse input source is unavailable",
            ))
    }
    fn drive_input(&mut self, was_paused: bool) -> Result<(), TerminalError> {
        let source = self.source_handle()?;
        // Source computes this resume offset before callbacks can resize a view.
        let mut start = 0;
        if was_paused && source.len()? > MAX_PARSE_BUFFER_LENGTH {
            start = next_range(self.source_position)?;
        }
        let logger = Rc::clone(&self.state.borrow().logger);
        super::diagnostics::log_source(&logger, source.as_ref())?;
        if self.decoded.len() < source.len()? && self.decoded.len() < MAX_PARSE_BUFFER_LENGTH {
            self.decoded = vec![0; source.len()?.min(MAX_PARSE_BUFFER_LENGTH)];
        }
        if !was_paused {
            let state = self.state.borrow();
            let active = state.active_buffer();
            let y = active.borrow().y;
            drop(state);
            self.state.borrow_mut().dirty = Some((y, y));
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
                self.decode_range(source.as_ref(), position..end)?;
                self.source_position = position;
                self.parse_current(None)?;
                position = candidate;
            }
        } else if !was_paused {
            self.decode_all(source.as_ref())?;
            self.source_position = 0;
            self.parse_current(None)?;
        }
        self.finish_input()
    }
    fn decode_range(
        &mut self,
        source: &dyn ParseInputSource,
        range: std::ops::Range<usize>,
    ) -> Result<(), TerminalError> {
        self.decode_chunk(source.read_range(range)?);
        Ok(())
    }
    fn decode_all(&mut self, source: &dyn ParseInputSource) -> Result<(), TerminalError> {
        self.decode_chunk(source.read_all()?);
        Ok(())
    }
    fn decode_chunk(&mut self, chunk: ParseChunk<'_>) {
        // The chunk and any host borrow end here, before parse_current is called.
        self.decoded_length = match chunk {
            ParseChunk::Text(data) => self.utf16.decode_to_slice(&data, &mut self.decoded),
            ParseChunk::Bytes(data) => self.utf8.decode_to_slice(&data, &mut self.decoded),
        };
    }
    fn parse_current(&mut self, previous: Option<bool>) -> Result<(), TerminalError> {
        if let Some(observer) = self.parse_observer.clone() {
            observer(&self.decoded, self.decoded_length)?;
        }
        let stored = self.decoded_length.min(self.decoded.len());
        let parsed = self.parser.parse(&self.decoded[..stored], previous);
        if let Some(error) = self.state.borrow_mut().error.take() {
            return Err(error);
        }
        if let Some(pending) = parsed? {
            self.pending = Some(pending);
            self.awaiting = true;
            dispatch_events(&self.state)?;
            return Err(TerminalError::Pending);
        }
        if self.decoded_length > self.decoded.len() {
            return Err(TerminalError::Event(EmitError::callback(
                DecodeBufferOverflow {
                    decoded_length: self.decoded_length,
                    buffer_length: self.decoded.len(),
                },
            )));
        }
        Ok(())
    }
    fn finish_input(&mut self) -> Result<(), TerminalError> {
        let mut state = self.state.borrow_mut();
        let active = state.active_buffer();
        let active = active.borrow();
        if self.cursor_start != (active.x, active.y) {
            state.events.borrow_mut().push(TerminalEvent::CursorMove);
        }
        if let Some((start, end)) = state.dirty.take() {
            let offset = active.ybase.saturating_sub(active.ydisp);
            if start.saturating_add(offset) < state.rows() {
                state.events.borrow_mut().push(TerminalEvent::Render {
                    start: start.saturating_add(offset).min(state.rows() - 1),
                    end: end.saturating_add(offset).min(state.rows() - 1),
                });
            }
        }
        state.events.borrow_mut().push(TerminalEvent::WriteParsed);
        drop(active);
        drop(state);
        self.source = None;
        dispatch_events(&self.state)
    }
}
fn next_range(position: usize) -> Result<usize, TerminalError> {
    position
        .checked_add(MAX_PARSE_BUFFER_LENGTH)
        .ok_or(TerminalError::InvalidParameter(
            "parse input offset exceeds supported range",
        ))
}
