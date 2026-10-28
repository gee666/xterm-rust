// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;

pub(super) const CONTROL_LIMIT: usize = 512;
pub(super) const DECODER_KEEP: usize = 4_194_304;
pub(super) const DECODER_INITIAL: f64 = 4_194_304.0;

pub(super) struct State {
    pub aborted: bool,
    pub decode_error: bool,
    pub in_control: bool,
    pub control: [u32; CONTROL_LIMIT],
    pub control_length: usize,
    pub parsed: Option<CommandRef>,
    pub encoded_limit: f64,
    pub total_encoded: f64,
    pub active_decoder: Option<DecoderRef>,
    pub pending: NumberMap<Rc<PendingTransmission>>,
    pub last_pending: Option<f64>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            aborted: false,
            decode_error: false,
            in_control: true,
            control: [0; CONTROL_LIMIT],
            control_length: 0,
            parsed: None,
            encoded_limit: 0.0,
            total_encoded: 0.0,
            active_decoder: None,
            pending: NumberMap::default(),
            last_pending: None,
        }
    }
}
impl State {
    pub fn control_string(&self) -> Result<Vec<u16>, StorageError> {
        let mut output = Vec::new();
        output
            .try_reserve(self.control_length * 2)
            .map_err(StorageError::host)?;
        for &code in &self.control[..self.control_length] {
            if code <= 0xffff {
                output.push(code as u16);
            } else if code <= 0x10ffff {
                let code = code - 0x10000;
                output.push(0xd800 | (code >> 10) as u16);
                output.push(0xdc00 | (code & 0x3ff) as u16);
            } else {
                return Err(StorageError::host(
                    KittyProtocolError::InvalidControlCodepoint(code),
                ));
            }
        }
        Ok(output)
    }
    pub fn remove_pending(&mut self, key: f64) {
        self.pending.remove(key);
        // Deliberately strict equality: NaN does not clear a NaN last key,
        // although the Map deletion itself uses SameValueZero.
        if self.last_pending.is_some_and(|last| last == key) {
            self.last_pending = None;
        }
    }
    pub fn cleanup_pending(&mut self) -> Result<(), StorageError> {
        let mut cursor = None;
        while let Some((ordinal, _, pending)) = self.pending.after(cursor) {
            cursor = Some(ordinal);
            let decoder = {
                let decoder = pending.decoder.try_borrow().map_err(decoder_borrow)?;
                Rc::clone(&decoder)
            };
            decoder.try_borrow_mut().map_err(decoder_borrow)?.release();
        }
        self.pending.clear();
        self.last_pending = None;
        Ok(())
    }
}
