// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use crate::{
    core_service::CoreError,
    options::OptionError,
    parser::escape::{IdentifierError, ParseError},
    unicode_service::UnicodeServiceError,
};
use std::fmt;

#[derive(Debug)]
pub enum TerminalError {
    Disposed,
    Event(crate::event::EmitError),
    Pending,
    Borrowed,
    MissingLine,
    InvalidDimensions,
    InvalidParameter(&'static str),
    AllocationFailed,
    Parse(ParseError),
    Identifier(IdentifierError),
    Option(OptionError),
    Core(CoreError),
    Unicode(UnicodeServiceError),
    Buffer(String),
    BufferState(crate::buffer::BufferError),
    Mouse(crate::mouse_state::MouseStateError),
    Log(crate::log_service::LogError),
    Link(crate::osc_link_service::OscLinkError),
    Charset(crate::charset::CharsetError),
}
impl fmt::Display for TerminalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disposed => f.write_str("terminal is disposed"),
            Self::Event(error) => error.fmt(f),
            Self::Pending => f.write_str("terminal has an unresolved asynchronous handler"),
            Self::Borrowed => f.write_str("terminal state is already borrowed"),
            Self::MissingLine => f.write_str("terminal buffer line is missing"),
            Self::InvalidParameter(message) => f.write_str(message),
            Self::AllocationFailed => f.write_str("terminal allocation failed"),
            Self::InvalidDimensions => f.write_str("terminal dimensions exceed supported range"),
            Self::Parse(error) => error.fmt(f),
            Self::Identifier(error) => error.fmt(f),
            Self::Option(error) => error.fmt(f),
            Self::Core(error) => error.fmt(f),
            Self::Unicode(error) => error.fmt(f),
            Self::Buffer(error) => f.write_str(error),
            Self::BufferState(error) => error.fmt(f),
            Self::Mouse(error) => error.fmt(f),
            Self::Log(error) => error.fmt(f),
            Self::Link(error) => error.fmt(f),
            Self::Charset(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for TerminalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Event(error) => Some(error),
            Self::Parse(error) => Some(error),
            Self::Identifier(error) => Some(error),
            Self::Option(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::BufferState(error) => Some(error),
            Self::Unicode(error) => Some(error),
            Self::Mouse(error) => Some(error),
            Self::Log(error) => Some(error),
            Self::Link(error) => Some(error),
            Self::Charset(error) => Some(error),
            _ => None,
        }
    }
}
impl From<crate::charset::CharsetError> for TerminalError {
    fn from(error: crate::charset::CharsetError) -> Self {
        Self::Charset(error)
    }
}
impl From<crate::osc_link_service::OscLinkError> for TerminalError {
    fn from(error: crate::osc_link_service::OscLinkError) -> Self {
        Self::Link(error)
    }
}
impl From<crate::buffer::BufferError> for TerminalError {
    fn from(error: crate::buffer::BufferError) -> Self {
        Self::BufferState(error)
    }
}
impl From<ParseError> for TerminalError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}
impl From<IdentifierError> for TerminalError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}
impl From<OptionError> for TerminalError {
    fn from(error: OptionError) -> Self {
        Self::Option(error)
    }
}
impl From<CoreError> for TerminalError {
    fn from(error: CoreError) -> Self {
        Self::Core(error)
    }
}
impl From<UnicodeServiceError> for TerminalError {
    fn from(error: UnicodeServiceError) -> Self {
        Self::Unicode(error)
    }
}
