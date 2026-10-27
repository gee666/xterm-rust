// Copyright (c) 2017-2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of common/parser/Constants.ts and TransitionTable.

use std::{error::Error, fmt};

#[path = "vt500_table.rs"]
mod data;

/// Non-ASCII input is classified into one table slot, as in xterm.js.
pub const NON_ASCII_PRINTABLE: u8 = 0xa0;
pub const STATE_COUNT: usize = 17;
pub const PAYLOAD_LIMIT: usize = 10_000_000;
const STATE_SHIFT: usize = 8;
const STATE_MASK: u16 = 255;

/// States in the xterm.js VT500 parser, retaining upstream discriminants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ParserState {
    Ground,
    Escape,
    EscapeIntermediate,
    CsiEntry,
    CsiParam,
    CsiIntermediate,
    CsiIgnore,
    SosPmString,
    OscString,
    DcsEntry,
    DcsParam,
    DcsIgnore,
    DcsIntermediate,
    DcsPassthrough,
    ApcEntry,
    ApcIntermediate,
    ApcPassthrough,
}

impl ParserState {
    pub const ALL: [Self; STATE_COUNT] = [
        Self::Ground,
        Self::Escape,
        Self::EscapeIntermediate,
        Self::CsiEntry,
        Self::CsiParam,
        Self::CsiIntermediate,
        Self::CsiIgnore,
        Self::SosPmString,
        Self::OscString,
        Self::DcsEntry,
        Self::DcsParam,
        Self::DcsIgnore,
        Self::DcsIntermediate,
        Self::DcsPassthrough,
        Self::ApcEntry,
        Self::ApcIntermediate,
        Self::ApcPassthrough,
    ];

    pub fn from_code(code: u8) -> Option<Self> {
        Self::ALL.get(usize::from(code)).copied()
    }
}

/// Actions in the xterm.js parser, retaining upstream discriminants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ParserAction {
    Ignore,
    Error,
    Print,
    Execute,
    OscStart,
    OscPut,
    OscEnd,
    CsiDispatch,
    Param,
    Collect,
    EscDispatch,
    Clear,
    DcsHook,
    DcsPut,
    DcsUnhook,
    ApcStart,
    ApcPut,
    ApcEnd,
}

impl ParserAction {
    pub const ALL: [Self; 18] = [
        Self::Ignore,
        Self::Error,
        Self::Print,
        Self::Execute,
        Self::OscStart,
        Self::OscPut,
        Self::OscEnd,
        Self::CsiDispatch,
        Self::Param,
        Self::Collect,
        Self::EscDispatch,
        Self::Clear,
        Self::DcsHook,
        Self::DcsPut,
        Self::DcsUnhook,
        Self::ApcStart,
        Self::ApcPut,
        Self::ApcEnd,
    ];

    pub fn from_code(code: u8) -> Option<Self> {
        Self::ALL.get(usize::from(code)).copied()
    }
}

/// One transition's action and resulting state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transition {
    pub action: ParserAction,
    pub next: ParserState,
}

impl Transition {
    pub const fn new(action: ParserAction, next: ParserState) -> Self {
        Self { action, next }
    }

    pub const fn packed(self) -> u16 {
        ((self.action as u16) << STATE_SHIFT) | self.next as u16
    }

    pub fn from_packed(value: u16) -> Option<Self> {
        Some(Self {
            action: ParserAction::from_code((value >> STATE_SHIFT) as u8)?,
            next: ParserState::from_code((value & STATE_MASK) as u8)?,
        })
    }
}

/// An attempted mutation outside the allocated custom transition table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableIndexError {
    pub index: usize,
    pub length: usize,
}

impl fmt::Display for TableIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "transition index {} exceeds table length {}",
            self.index, self.length
        )
    }
}

impl Error for TableIndexError {}

/// Packed VT500 transitions with explicit validation for custom table writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransitionTable {
    table: Vec<u16>,
}

impl Default for TransitionTable {
    fn default() -> Self {
        Self::vt500()
    }
}

impl TransitionTable {
    /// A zero-filled custom table. Zero means Ignore/Ground, as in Uint16Array.
    pub fn new(length: usize) -> Self {
        Self {
            table: vec![0; length],
        }
    }

    /// The exact 4257-entry table generated from the original transition rules.
    pub fn vt500() -> Self {
        Self {
            table: data::VT500.to_vec(),
        }
    }

    pub fn packed(&self) -> &[u16] {
        &self.table
    }

    pub fn set_default(&mut self, transition: Transition) {
        self.table.fill(transition.packed());
    }

    pub fn add(
        &mut self,
        code: u8,
        state: ParserState,
        transition: Transition,
    ) -> Result<(), TableIndexError> {
        let index = Self::index(code, state);
        let length = self.table.len();
        let entry = self
            .table
            .get_mut(index)
            .ok_or(TableIndexError { index, length })?;
        *entry = transition.packed();
        Ok(())
    }

    /// Apply a group atomically. Failed validation leaves every entry unchanged.
    pub fn add_many(
        &mut self,
        codes: &[u8],
        state: ParserState,
        transition: Transition,
    ) -> Result<(), TableIndexError> {
        for &code in codes {
            let index = Self::index(code, state);
            if index >= self.table.len() {
                return Err(TableIndexError {
                    index,
                    length: self.table.len(),
                });
            }
        }
        for &code in codes {
            self.table[Self::index(code, state)] = transition.packed();
        }
        Ok(())
    }

    /// Raw byte lookup, including slots not used by the normal parser.
    pub fn get(&self, code: u8, state: ParserState) -> Option<Transition> {
        self.table
            .get(Self::index(code, state))
            .and_then(|&value| Transition::from_packed(value))
    }

    /// Lookup for a decoded UTF-32 value, mapping values >= U+00A0 to one slot.
    pub fn for_codepoint(&self, codepoint: u32, state: ParserState) -> Option<Transition> {
        let code = if codepoint >= u32::from(NON_ASCII_PRINTABLE) {
            NON_ASCII_PRINTABLE
        } else {
            codepoint as u8
        };
        self.get(code, state)
    }

    const fn index(code: u8, state: ParserState) -> usize {
        ((state as usize) << STATE_SHIFT) | code as usize
    }
}
