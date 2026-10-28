// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Native KittyGraphicsHandler protocol controller.

use super::command::parse_command;
use super::handler_types::{action_is, command_snapshot, direct, is_ascii, truthy};
use super::storage::{KittyImageBytes, KittyImageStorage, KittyImages, KittyPlacements};
use crate::image::base64::{DecodeStatus, InvalidSettings, PayloadDecoder};
use crate::image::encoded::EncodedSourceFactory;
use crate::image::number_map::{NumberMap, NumberMapView};
use crate::image::storage::StorageError;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use xterm_core::parser::handler::{DispatchError, HandlerResult};

pub use super::handler_types::{
    decoder_snapshot, CommandRef, DecoderRef, DecoderSnapshot, DisplayFuture, KittyDiagnostics,
    KittyDisplayRequest, KittyGraphicsHost, KittyOptions, KittyProtocolError, PendingTransmission,
};
#[path = "commands.rs"]
mod commands;
#[path = "finalize.rs"]
mod finalize;
#[path = "handler_state.rs"]
mod state;
#[path = "transmission.rs"]
mod transmission;
use state::{State, CONTROL_LIMIT, DECODER_INITIAL, DECODER_KEEP};
#[path = "apc_adapter.rs"]
mod apc_adapter;
pub use apc_adapter::KittyApcAdapter;

pub struct KittyGraphicsHandler {
    state: Rc<RefCell<State>>,
    host: Rc<dyn KittyGraphicsHost>,
    storage: Rc<KittyImageStorage>,
    factory: Rc<dyn EncodedSourceFactory>,
    options: KittyOptions,
    max_encoded: f64,
    initial_encoded: f64,
    pending_view: NumberMapView<Rc<PendingTransmission>>,
}
impl KittyGraphicsHandler {
    pub fn new(
        host: Rc<dyn KittyGraphicsHost>,
        storage: Rc<KittyImageStorage>,
        factory: Rc<dyn EncodedSourceFactory>,
        options: KittyOptions,
    ) -> Self {
        let max_encoded = (options.size_limit * 4.0 / 3.0).ceil();
        let initial_encoded = if max_encoded.is_nan() {
            f64::NAN
        } else {
            DECODER_INITIAL.min(max_encoded)
        };
        let state = Rc::new(RefCell::new(State::default()));
        let pending_view = NumberMapView::new(Rc::clone(&state), |state| &state.pending);
        Self {
            state,
            host,
            storage,
            factory,
            options,
            max_encoded,
            initial_encoded,
            pending_view,
        }
    }

    pub fn start(&self) -> Result<(), StorageError> {
        let mut state = self.state.try_borrow_mut().map_err(handler_borrow)?;
        state.aborted = false;
        state.decode_error = false;
        state.in_control = true;
        state.control_length = 0;
        state.parsed = None;
        state.encoded_limit = self.max_encoded;
        state.total_encoded = 0.0;
        state.active_decoder = None;
        Ok(())
    }
    pub fn reset(&self) -> Result<(), StorageError> {
        {
            let mut state = self.state.try_borrow_mut().map_err(handler_borrow)?;
            state.cleanup_pending()?;
            if let Some(decoder) = state.active_decoder.as_ref().cloned() {
                decoder.try_borrow_mut().map_err(decoder_borrow)?.release();
                state.active_decoder = None;
            }
            // Source reset deliberately retains control flags, parsed command
            // and current counters. The following start resets those fields.
        }
        self.storage.reset()
    }
    pub fn dispose(&self) -> Result<(), StorageError> {
        self.reset()
    }
    pub fn images(&self) -> KittyImages {
        self.storage.images()
    }
    pub fn placements(&self) -> KittyPlacements {
        self.storage.placements()
    }
    pub fn pending_transmissions(&self) -> NumberMapView<Rc<PendingTransmission>> {
        self.pending_view.clone()
    }

    pub fn diagnostics(&self) -> Result<KittyDiagnostics, StorageError> {
        let state = self.state.try_borrow().map_err(handler_borrow)?;
        Ok(KittyDiagnostics {
            aborted: state.aborted,
            decode_error: state.decode_error,
            in_control: state.in_control,
            control: state.control[..state.control_length].to_vec(),
            parsed: state.parsed.as_ref().map(command_snapshot).transpose()?,
            encoded_limit: state.encoded_limit,
            total_encoded: state.total_encoded,
            last_pending: state.last_pending,
            active_decoder: state
                .active_decoder
                .as_ref()
                .map(decoder_snapshot)
                .transpose()?,
        })
    }

    fn decoder(&self) -> Result<DecoderRef, StorageError> {
        if !self.max_encoded.is_finite()
            || !self.initial_encoded.is_finite()
            || self.max_encoded < 0.0
            || self.initial_encoded < 0.0
            || self.max_encoded > usize::MAX as f64
            || self.initial_encoded > usize::MAX as f64
        {
            return Err(StorageError::host(InvalidSettings));
        }
        // Positive integer limits come from ceil/min. PayloadDecoder performs
        // the original maximum-address check before any allocation.
        let mut decoder = PayloadDecoder::new(
            DECODER_KEEP,
            self.max_encoded as usize,
            self.initial_encoded as usize,
        )
        .map_err(StorageError::host)?;
        decoder.init(None, None).map_err(StorageError::host)?;
        Ok(Rc::new(RefCell::new(decoder)))
    }

    fn response(
        &self,
        id: f64,
        message: &str,
        quiet: f64,
        placement_id: Option<f64>,
    ) -> Result<(), StorageError> {
        response_to(self.host.as_ref(), id, message, quiet, placement_id)
    }
}

fn response_to(
    host: &dyn KittyGraphicsHost,
    id: f64,
    message: &str,
    quiet: f64,
    placement_id: Option<f64>,
) -> Result<(), StorageError> {
    let ok = message == "OK";
    if (ok && quiet >= 1.0) || (!ok && quiet >= 2.0) {
        return Ok(());
    }
    let mut response = String::from("\x1b_Gi=");
    let mut number = ryu_js::Buffer::new();
    response.push_str(number.format(id));
    if let Some(placement) = placement_id.filter(|value| truthy(*value)) {
        response.push_str(",p=");
        response.push_str(number.format(placement));
    }
    response.push(';');
    response.push_str(message);
    response.push_str("\x1b\\");
    host.send_response(&response.encode_utf16().collect::<Vec<_>>())
}

fn handler_borrow(_: impl fmt::Debug) -> StorageError {
    StorageError::host(KittyProtocolError::BorrowedHandler)
}
fn decoder_borrow(_: impl fmt::Debug) -> StorageError {
    StorageError::host(KittyProtocolError::BorrowedDecoder)
}
fn command_borrow(_: impl fmt::Debug) -> StorageError {
    StorageError::host(KittyProtocolError::BorrowedCommand)
}
fn dispatch_error(error: StorageError) -> DispatchError {
    match error {
        StorageError::Event(error) => DispatchError::Callback(error),
        other => DispatchError::Callback(xterm_core::event::EmitError::callback(other)),
    }
}
