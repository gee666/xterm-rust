// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use std::{error::Error, fmt, rc::Rc};
use xterm_core::{buffer::BufferError, event::EmitError};

/// Storage failures retain mutations committed before the source operation threw.
#[derive(Debug)]
pub enum StorageError {
    InvalidLimit,
    InvalidGeometry,
    MissingLine(usize),
    BorrowedBuffer,
    DisconnectedTerminal,
    IdentifierOrder,
    Buffer(BufferError),
    Event(EmitError),
    Host(Rc<dyn Error>),
}
impl StorageError {
    pub fn host(error: impl Error + 'static) -> Self {
        Self::Host(Rc::new(error))
    }
}
impl From<BufferError> for StorageError {
    fn from(error: BufferError) -> Self {
        Self::Buffer(error)
    }
}
impl From<EmitError> for StorageError {
    fn from(error: EmitError) -> Self {
        Self::Event(error)
    }
}
impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimit => {
                f.write_str("invalid storageLimit, should be at least 0.5 MB and not exceed 1G")
            }
            Self::InvalidGeometry => {
                f.write_str("image placement requires finite positive cell dimensions")
            }
            Self::MissingLine(row) => write!(f, "image buffer row {row} is absent"),
            Self::BorrowedBuffer => f.write_str("image buffer is already borrowed"),
            Self::DisconnectedTerminal => {
                f.write_str("image terminal owner is no longer available")
            }
            Self::IdentifierOrder => {
                f.write_str("image identifiers no longer advance in insertion order")
            }
            Self::Buffer(error) => error.fmt(f),
            Self::Event(error) => error.fmt(f),
            Self::Host(error) => error.fmt(f),
        }
    }
}
impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Buffer(error) => Some(error),
            Self::Event(error) => Some(error),
            Self::Host(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}
