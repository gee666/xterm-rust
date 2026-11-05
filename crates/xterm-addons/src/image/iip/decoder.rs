// Copyright (c) 2023 The xterm.js authors. MIT license.
//! IIP constructs its decoder before allocating memory. Preserve that boundary
//! for raw numeric settings without allowing invalid values into Rust addresses.
use crate::image::base64::{DecodeStatus, InvalidSettings, PayloadDecoder, MAX_BYTES};
use std::collections::TryReserveError;

pub(super) struct IipDecoder {
    inner: Option<PayloadDecoder>,
}
impl IipDecoder {
    pub(super) fn new(encoded_limit: f64) -> Result<Self, InvalidSettings> {
        // Source's constructor rejects the upper bound, but accepts NaN and
        // negative limits. Their unsafe memory/decoder paths are not emulated.
        if encoded_limit > MAX_BYTES as f64 {
            return Err(InvalidSettings);
        }
        let inner = if encoded_limit.is_finite() && encoded_limit >= 0.0 {
            // The checked cap fits usize even on wasm32.
            let limit = encoded_limit as usize;
            Some(PayloadDecoder::new(
                super::types::DECODER_KEEP_BYTES,
                limit,
                super::types::DECODER_INITIAL_BYTES.min(limit),
            )?)
        } else {
            None
        };
        Ok(Self { inner })
    }
    pub(super) fn init(&mut self) -> Result<(), InvalidSettings> {
        self.inner.as_mut().ok_or(InvalidSettings)?.init(None, None)
    }
    pub(super) fn release(&mut self) {
        if let Some(decoder) = &mut self.inner {
            decoder.release();
        }
    }
    pub(super) fn loaded_bytes(&self) -> usize {
        self.inner.as_ref().map_or(0, PayloadDecoder::loaded_bytes)
    }
    pub(super) fn data(&self) -> &[u8] {
        self.inner.as_ref().map_or(&[], PayloadDecoder::data)
    }
    pub(super) fn put(&mut self, input: &[u32]) -> Result<DecodeStatus, TryReserveError> {
        self.inner
            .as_mut()
            .map_or(Ok(DecodeStatus::NotInitialized), |decoder| {
                decoder.put(input)
            })
    }
    pub(super) fn end(&mut self) -> Result<DecodeStatus, TryReserveError> {
        self.inner
            .as_mut()
            .map_or(Ok(DecodeStatus::NotInitialized), PayloadDecoder::end)
    }
}
