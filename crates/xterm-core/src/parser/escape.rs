// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of common/parser/EscapeSequenceParser.ts.

//! VT500 parser over decoded UTF-32 chunks. Await a returned pending future,
//! then call `parse` with the same chunk and its boolean result. Reset cleanup
//! futures must also be driven by the caller. See `docs/escape-parser.md`.

use super::{
    apc::ApcParser,
    dcs::DcsParser,
    handler::{DispatchError, HandlerResult},
    osc::OscParser,
    params::{Params, ParamsError},
    table::{ParserAction, ParserState, Transition, TransitionTable},
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    error::Error,
    fmt,
    rc::Rc,
};

#[path = "escape_handlers.rs"]
mod handlers;
#[path = "escape_parse.rs"]
mod parse;
#[path = "escape_resume.rs"]
mod resume;
pub use handlers::{CommandRegistrar, FunctionIdentifier, RegistrarError};
use handlers::{HandlerList, Registry};

/// Same identity can be registered repeatedly; disposal removes its first match.
pub type CommandHandler = Rc<RefCell<dyn FnMut(&Params) -> HandlerResult>>;
type PrintHandler = Box<dyn FnMut(&[u32]) -> Result<(), ParseError>>;
type ExecuteHandler = Box<dyn FnMut(Option<u32>) -> Result<bool, ParseError>>;
type CsiFallback = Box<dyn FnMut(u32, &Params)>;
type ErrorHandler = Box<dyn FnMut(ParsingState<'_>) -> bool>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentifierError {
    PrefixLength,
    PrefixRange,
    IntermediatesLength,
    IntermediatesRange,
    FinalLength,
    FinalRange(u8),
}
impl fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrefixLength => f.write_str("only one byte as prefix supported"),
            Self::PrefixRange => f.write_str("prefix must be in range 0x3c .. 0x3f"),
            Self::IntermediatesLength => {
                f.write_str("only two bytes as intermediates are supported")
            }
            Self::IntermediatesRange => f.write_str("intermediate must be in range 0x20 .. 0x2f"),
            Self::FinalLength => f.write_str("final must be a single byte"),
            Self::FinalRange(lower) => write!(f, "final must be in range {lower} .. 126"),
        }
    }
}
impl Error for IdentifierError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    ImproperContinuation,
    MissingTransition,
    HandlerAborted,
    Dispatch(DispatchError),
    Params(ParamsError),
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ImproperContinuation => f.write_str(
                "improper continuation due to previous async handler, giving up parsing",
            ),
            Self::MissingTransition => {
                f.write_str("custom transition table has no entry for this state and code")
            }
            Self::HandlerAborted => f.write_str("terminal callback aborted parsing"),
            Self::Dispatch(error) => error.fmt(f),
            Self::Params(error) => error.fmt(f),
        }
    }
}
impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Dispatch(error) => Some(error),
            Self::Params(error) => Some(error),
            _ => None,
        }
    }
}
impl From<DispatchError> for ParseError {
    fn from(value: DispatchError) -> Self {
        Self::Dispatch(value)
    }
}
impl From<ParamsError> for ParseError {
    fn from(value: ParamsError) -> Self {
        Self::Params(value)
    }
}

/// Borrowed diagnostic state. Upstream only honors the callback's abort flag.
pub struct ParsingState<'a> {
    pub position: usize,
    pub code: u32,
    pub current_state: ParserState,
    pub collect: u32,
    pub params: &'a Params,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandlerKind {
    Csi,
    Esc,
    Dcs,
    Osc,
    Apc,
}
type PendingKind = HandlerKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PauseInfo {
    pub kind: HandlerKind,
    pub handler_position: usize,
    pub chunk_position: usize,
    pub transition: Transition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseStackState {
    None,
    Paused,
    Failed,
    Reset,
}

#[derive(Clone)]
struct Pause {
    kind: PendingKind,
    handlers: HandlerList,
    next: usize,
    transition: Transition,
    position: usize,
}
enum Stack {
    None,
    Paused(Pause),
    Failed(Pause),
    Reset(usize),
}

pub struct EscapeSequenceParser {
    pub initial_state: ParserState,
    pub current_state: ParserState,
    /// Shared with PRINT callbacks so grapheme state can change without an alias
    /// of the mutable parser borrow. Other valid commands reset it to zero.
    pub preceding_join_state: Rc<Cell<u32>>,
    pub osc: OscParser,
    pub dcs: DcsParser,
    pub apc: ApcParser,
    transitions: TransitionTable,
    params: Params,
    collect: u32,
    print: Option<PrintHandler>,
    execute: HashMap<u32, ExecuteHandler>,
    execute_fallback: Option<Box<dyn FnMut(u32)>>,
    csi: Registry,
    esc: Registry,
    csi_fallback: Option<CsiFallback>,
    esc_fallback: Option<Box<dyn FnMut(u32)>>,
    error: Option<ErrorHandler>,
    action_boundary: Option<Box<dyn FnMut() -> Result<(), ParseError>>>,
    stack: Stack,
}
impl Default for EscapeSequenceParser {
    fn default() -> Self {
        Self::new()
    }
}
impl EscapeSequenceParser {
    pub fn new() -> Self {
        Self::with_transitions(TransitionTable::vt500())
    }
    pub fn with_transitions(transitions: TransitionTable) -> Self {
        let mut params = Params::default();
        params
            .reset_zdm()
            .expect("default Params has nonzero capacity");
        let parser = Self {
            initial_state: ParserState::Ground,
            current_state: ParserState::Ground,
            preceding_join_state: Rc::default(),
            osc: OscParser::new(),
            dcs: DcsParser::new(),
            apc: ApcParser::new(),
            transitions,
            params,
            collect: 0,
            print: None,
            execute: HashMap::new(),
            execute_fallback: None,
            csi: Registry::default(),
            esc: Registry::default(),
            csi_fallback: None,
            esc_fallback: None,
            error: None,
            action_boundary: None,
            stack: Stack::None,
        };
        // The default ST handler is a real registration and can be cleared.
        parser.esc.register(
            u32::from(b'\\'),
            Rc::new(RefCell::new(|_: &Params| HandlerResult::Ready(true))),
        );
        parser
    }
    /// Adapter event delivery after completed synchronous actions, before the
    /// next codepoint. A pending action is delivered by the caller after parse
    /// returns, so a delivery error cannot discard an owned future.
    pub fn set_action_boundary_handler(
        &mut self,
        handler: impl FnMut() -> Result<(), ParseError> + 'static,
    ) {
        self.action_boundary = Some(Box::new(handler));
    }
    fn action_boundary(&mut self) -> Result<(), ParseError> {
        if let Some(handler) = &mut self.action_boundary {
            handler()?;
        }
        Ok(())
    }
    pub fn parse_stack_state(&self) -> ParseStackState {
        match self.stack {
            Stack::None => ParseStackState::None,
            Stack::Paused(_) => ParseStackState::Paused,
            Stack::Failed(_) => ParseStackState::Failed,
            Stack::Reset(_) => ParseStackState::Reset,
        }
    }
    pub fn pause_info(&self) -> Option<PauseInfo> {
        match &self.stack {
            Stack::Paused(pause) | Stack::Failed(pause) => Some(PauseInfo {
                kind: pause.kind,
                handler_position: pause.next,
                chunk_position: pause.position,
                transition: pause.transition,
            }),
            _ => None,
        }
    }
    pub fn params(&self) -> &Params {
        &self.params
    }
    pub fn collect(&self) -> u32 {
        self.collect
    }
    pub fn transitions(&self) -> &TransitionTable {
        &self.transitions
    }
    /// Abort active payloads but retain registered handlers. A paused chunk
    /// continues at its next codepoint on the next parse call, as upstream does.
    pub fn reset(&mut self) -> Result<(), ParseError> {
        self.current_state = self.initial_state;
        self.osc.reset()?;
        self.dcs.reset()?;
        self.apc.reset()?;
        self.clear_sequence()?;
        self.preceding_join_state.set(0);
        self.stack = match std::mem::replace(&mut self.stack, Stack::None) {
            Stack::Paused(pause) | Stack::Failed(pause) => Stack::Reset(pause.position),
            other => other,
        };
        Ok(())
    }
    pub fn take_cleanup(&mut self) -> Vec<HandlerResult> {
        let mut cleanup = self.osc.take_cleanup();
        cleanup.extend(self.dcs.take_cleanup());
        cleanup.extend(self.apc.take_cleanup());
        cleanup
    }
    /// Explicit upstream-style disposal does not invoke active end callbacks.
    /// Dropping the Rust object releases all parser-owned resources via RAII.
    pub fn dispose(&mut self) {
        self.osc.dispose();
        self.dcs.dispose();
        self.apc.dispose();
        self.csi.clear_all();
        self.esc.clear_all();
        self.execute.clear();
    }
    fn clear_sequence(&mut self) -> Result<(), ParseError> {
        self.params.reset_zdm()?;
        self.collect = 0;
        Ok(())
    }
    fn execute_code(&mut self, code: u32, fast: bool) -> Result<(), ParseError> {
        if let Some(handler) = self.execute.get_mut(&code) {
            handler(fast.then_some(code))?;
        } else if let Some(fallback) = &mut self.execute_fallback {
            fallback(code);
        }
        self.preceding_join_state.set(0);
        Ok(())
    }
    fn parameter(&mut self, code: u32) -> Result<(), ParseError> {
        match code {
            0x3b => self.params.add_param(0)?,
            0x3a => self.params.add_sub_param(-1)?,
            0x30..=0x39 => self.params.add_digit((code - 0x30) as u8)?,
            // A caller-defined table can send other bytes to PARAM. Reject them
            // before subtraction rather than panicking on untrusted input.
            _ => return Err(ParamsError::InvalidDigit.into()),
        }
        Ok(())
    }
}
