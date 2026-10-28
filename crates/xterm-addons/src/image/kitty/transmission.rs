// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;

impl KittyGraphicsHandler {
    pub fn put(&self, data: &[u32]) -> Result<(), StorageError> {
        let mut state = self.state.try_borrow_mut().map_err(handler_borrow)?;
        if state.aborted {
            return Ok(());
        }
        if !state.in_control {
            drop(state);
            return self.stream_payload(data);
        }
        let separator = data.iter().position(|code| *code == u32::from(b';'));
        let end = separator.unwrap_or(data.len());
        if separator.is_some() {
            state.in_control = false;
        }
        let Some(length) = state
            .control_length
            .checked_add(end)
            .filter(|length| *length <= CONTROL_LIMIT)
        else {
            state.aborted = true;
            return Ok(());
        };
        let start = state.control_length;
        state.control[start..length].copy_from_slice(&data[..end]);
        state.control_length = length;
        if state.in_control {
            return Ok(());
        }
        let command = Rc::new(RefCell::new(parse_command(&state.control_string()?)));
        state.parsed = Some(Rc::clone(&command));
        let (id, image_number, quiet, delete) = {
            let command = command.try_borrow().map_err(command_borrow)?;
            (
                command.id,
                command.image_number,
                command.quiet.unwrap_or(0.0),
                action_is(&command, b'd'),
            )
        };
        drop(state);
        if let (Some(id), Some(_)) = (id, image_number) {
            self.response(id, "EINVAL:cannot specify both i and I keys", quiet, None)?;
            // A failing reply bypasses this source mutation. No map borrow spans
            // the host callback, so reset/start may reenter before this assignment.
            self.state.try_borrow_mut().map_err(handler_borrow)?.aborted = true;
            return Ok(());
        }
        if delete {
            return Ok(());
        }
        if end + 1 < data.len() {
            self.stream_payload(&data[end + 1..])?;
        }
        Ok(())
    }

    fn stream_payload(&self, data: &[u32]) -> Result<(), StorageError> {
        let mut state = self.state.try_borrow_mut().map_err(handler_borrow)?;
        if state.aborted {
            return Ok(());
        }
        let id = if let Some(command) = &state.parsed {
            command.try_borrow().map_err(command_borrow)?.id
        } else {
            None
        };
        let key = id.or(state.last_pending).unwrap_or(0.0);
        let pending = state.pending.get(key).cloned();
        let previous = pending
            .as_ref()
            .map_or(0.0, |pending| pending.total_encoded_size.get());
        state.total_encoded += data.len() as f64;
        if previous + state.total_encoded > state.encoded_limit {
            let decoder = if let Some(decoder) = &state.active_decoder {
                Some(Rc::clone(decoder))
            } else if let Some(pending) = &pending {
                let decoder = pending.decoder.try_borrow().map_err(decoder_borrow)?;
                Some(Rc::clone(&decoder))
            } else {
                None
            };
            if let Some(decoder) = decoder {
                decoder.try_borrow_mut().map_err(decoder_borrow)?.release();
            }
            state.active_decoder = None;
            if pending.is_some() {
                state.remove_pending(key);
            }
            state.aborted = true;
            return Ok(());
        }
        if state.decode_error {
            return Ok(());
        }
        let decoder = if let Some(decoder) = &state.active_decoder {
            Rc::clone(decoder)
        } else {
            let decoder = if let Some(pending) = &pending {
                let decoder = pending.decoder.try_borrow().map_err(decoder_borrow)?;
                Rc::clone(&decoder)
            } else {
                self.decoder()?
            };
            state.active_decoder = Some(Rc::clone(&decoder));
            decoder
        };
        let status = decoder
            .try_borrow_mut()
            .map_err(decoder_borrow)?
            .put(data)
            .map_err(StorageError::host)?;
        if status != DecodeStatus::Ok {
            decoder.try_borrow_mut().map_err(decoder_borrow)?.release();
            state.active_decoder = None;
            state.decode_error = true;
            if pending.is_some() {
                state.remove_pending(key);
            }
        }
        Ok(())
    }
}
