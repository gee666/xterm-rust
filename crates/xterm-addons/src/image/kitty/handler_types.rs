// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Kitty graphics state and host contracts.

use super::command::KittyCommand;
use super::storage::{KittyImageData, KittyImageStorage};
use crate::image::base64::PayloadDecoder;
use crate::image::storage::StorageError;
use std::cell::{Cell, RefCell};
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

pub type CommandRef = Rc<RefCell<KittyCommand>>;
pub type DecoderRef = Rc<RefCell<PayloadDecoder>>;
pub type DisplayFuture = Pin<Box<dyn Future<Output = Result<(), StorageError>>>>;

/// The host must perform real decode/crop/scale/storage/cursor work. Begin any
/// source-eager platform prefix before returning its future. Do not cancel this
/// owned request on reset/dispose: upstream promises may complete afterward.
pub struct KittyDisplayRequest {
    pub image: Rc<KittyImageData>,
    pub command: CommandRef,
    pub storage: Rc<KittyImageStorage>,
    pub pixel_limit: f64,
}

pub trait KittyGraphicsHost {
    /// Route through the actual non-user CoreService data path, including its
    /// disableStdin policy. A listener failure is not a successful false result.
    fn send_response(&self, data: &[u16]) -> Result<(), StorageError>;
    fn begin_display(&self, request: KittyDisplayRequest) -> DisplayFuture;
}

#[derive(Clone, Copy, Debug)]
pub struct KittyOptions {
    pub size_limit: f64,
    pub pixel_limit: f64,
}
impl Default for KittyOptions {
    fn default() -> Self {
        Self {
            size_limit: 33_554_432.0,
            pixel_limit: 16_777_216.0,
        }
    }
}

/// Source Map values are mutable records, not detached diagnostics. The first
/// command is a shallow object copy; replacing that reference does not change a
/// command already captured by a display future. Decoder aliases stay shared.
pub struct PendingTransmission {
    pub command: RefCell<CommandRef>,
    pub decoder: RefCell<DecoderRef>,
    pub total_encoded_size: Cell<f64>,
    pub decode_error: Cell<bool>,
}

#[derive(Clone, Debug)]
pub struct DecoderSnapshot {
    pub loaded_bytes: usize,
    pub free_bytes: i128,
    pub data: Vec<u8>,
}

pub fn decoder_snapshot(decoder: &DecoderRef) -> Result<DecoderSnapshot, StorageError> {
    let decoder = decoder
        .try_borrow()
        .map_err(|_| StorageError::host(KittyProtocolError::BorrowedDecoder))?;
    let mut data = Vec::new();
    data.try_reserve_exact(decoder.data().len())
        .map_err(StorageError::host)?;
    data.extend_from_slice(decoder.data());
    Ok(DecoderSnapshot {
        loaded_bytes: decoder.loaded_bytes(),
        free_bytes: decoder.free_bytes(),
        data,
    })
}

#[derive(Clone, Debug)]
pub struct KittyDiagnostics {
    pub aborted: bool,
    pub decode_error: bool,
    pub in_control: bool,
    pub control: Vec<u32>,
    pub parsed: Option<KittyCommand>,
    pub encoded_limit: f64,
    pub total_encoded: f64,
    pub last_pending: Option<f64>,
    pub active_decoder: Option<DecoderSnapshot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KittyProtocolError {
    BorrowedHandler,
    BorrowedCommand,
    BorrowedDecoder,
    MissingCommand,
    MissingPayload,
    InvalidControlCodepoint(u32),
}
impl fmt::Display for KittyProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BorrowedHandler => f.write_str("Kitty graphics state is already borrowed"),
            Self::BorrowedCommand => f.write_str("Kitty control command is already borrowed"),
            Self::BorrowedDecoder => f.write_str("Kitty payload decoder is already borrowed"),
            Self::MissingCommand => f.write_str("Kitty payload has no parsed command"),
            Self::MissingPayload => {
                f.write_str("Kitty transmit command has no captured payload bytes")
            }
            Self::InvalidControlCodepoint(code) => write!(f, "Invalid code point {code}"),
        }
    }
}
impl std::error::Error for KittyProtocolError {}

pub(super) fn command_snapshot(command: &CommandRef) -> Result<KittyCommand, StorageError> {
    Ok(command
        .try_borrow()
        .map_err(|_| StorageError::host(KittyProtocolError::BorrowedCommand))?
        .clone())
}
pub(super) fn is_ascii(value: Option<&[u16]>, expected: &[u8]) -> bool {
    value.is_some_and(|value| {
        value.len() == expected.len()
            && value
                .iter()
                .zip(expected)
                .all(|(unit, byte)| *unit == u16::from(*byte))
    })
}
pub(super) fn action_is(command: &KittyCommand, expected: u8) -> bool {
    if command.action.is_none() {
        expected == b't'
    } else {
        is_ascii(command.action.as_deref(), &[expected])
    }
}
pub(super) fn direct(command: &KittyCommand) -> bool {
    command.transmission.is_none() || is_ascii(command.transmission.as_deref(), b"d")
}
pub(super) fn truthy(value: f64) -> bool {
    value != 0.0 && !value.is_nan()
}
