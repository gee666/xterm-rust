// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use std::cell::Cell;

impl KittyGraphicsHandler {
    pub fn end(&self, success: bool) -> Result<HandlerResult, StorageError> {
        let mut state = self.state.try_borrow_mut().map_err(handler_borrow)?;
        if state.aborted || !success {
            if let Some(decoder) = state.active_decoder.as_ref().cloned() {
                decoder.try_borrow_mut().map_err(decoder_borrow)?.release();
                state.active_decoder = None;
            }
            return Ok(HandlerResult::Ready(true));
        }
        if state.in_control {
            let command = Rc::new(RefCell::new(parse_command(&state.control_string()?)));
            drop(state);
            return self.no_payload(command);
        }
        let command = state
            .parsed
            .as_ref()
            .cloned()
            .ok_or_else(|| StorageError::host(KittyProtocolError::MissingCommand))?;
        let info = command_snapshot(&command)?;
        if action_is(&info, b'd') {
            drop(state);
            self.delete_command(&command)?;
            return Ok(HandlerResult::Ready(true));
        }
        let key = info.id.or(state.last_pending).unwrap_or(0.0);
        let pending = state.pending.get(key).cloned();
        if info.more == Some(1.0) {
            if let Some(decoder) = state.active_decoder.as_ref().cloned() {
                if let Some(pending) = &pending {
                    pending
                        .total_encoded_size
                        .set(pending.total_encoded_size.get() + state.total_encoded);
                    pending
                        .decode_error
                        .set(pending.decode_error.get() || state.decode_error);
                } else {
                    let first = Rc::new(RefCell::new(info));
                    let pending = Rc::new(PendingTransmission {
                        command: RefCell::new(first),
                        decoder: RefCell::new(decoder),
                        total_encoded_size: Cell::new(state.total_encoded),
                        decode_error: Cell::new(state.decode_error),
                    });
                    state
                        .pending
                        .set(key, pending)
                        .map_err(StorageError::host)?;
                }
                state.last_pending = Some(key);
                state.active_decoder = None;
            }
            return Ok(HandlerResult::Ready(true));
        }
        if pending.is_some() {
            state.last_pending = None;
        }
        let mut decode_error = state.decode_error;
        let mut final_command = command;
        let mut decoder = state.active_decoder.as_ref().cloned();
        if let Some(pending) = &pending {
            {
                let command = pending.command.try_borrow().map_err(command_borrow)?;
                final_command = Rc::clone(&command);
            }
            {
                let pending_decoder = pending.decoder.try_borrow().map_err(decoder_borrow)?;
                decoder = Some(Rc::clone(&pending_decoder));
            }
            decode_error |= pending.decode_error.get();
            state.pending.remove(key);
        }
        let length = if let Some(decoder) = &decoder {
            let mut decoder = decoder.try_borrow_mut().map_err(decoder_borrow)?;
            if decoder.end().map_err(StorageError::host)? != DecodeStatus::Ok {
                decode_error = true;
            }
            decoder.data().len()
        } else {
            0
        };
        state.active_decoder = None;
        drop(state);

        let info = command_snapshot(&final_command)?;
        let needs_bytes = !decode_error
            && length != 0
            && direct(&info)
            && (action_is(&info, b't') || action_is(&info, b'T'));
        let owned = if needs_bytes {
            if let Some(decoder) = &decoder {
                let decoder = decoder.try_borrow().map_err(decoder_borrow)?;
                let mut owned = Vec::new();
                owned
                    .try_reserve_exact(decoder.data().len())
                    .map_err(StorageError::host)?;
                owned.extend_from_slice(decoder.data());
                Some(owned)
            } else {
                None
            }
        } else {
            None
        };
        // Source command handling captures Blob data before release. No decoder
        // or handler borrow spans a host callback, and a throwing command skips
        // release just as the original control flow does.
        let result = self.payload_command(final_command, owned, length, decode_error)?;
        if let Some(decoder) = decoder {
            decoder.try_borrow_mut().map_err(decoder_borrow)?.release();
        }
        Ok(result)
    }
}
