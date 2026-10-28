// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;

const INVALID_BASE64: &str = "EINVAL:invalid base64 data";
const UNSUPPORTED_MEDIUM: &str = "EINVAL:unsupported transmission medium";
const RENDER_FAILED: &str = "EINVAL:image rendering failed";
const FORMAT_PNG: f64 = 100.0;
const FORMAT_RGBA: f64 = 32.0;

impl KittyGraphicsHandler {
    pub(super) fn no_payload(&self, command: CommandRef) -> Result<HandlerResult, StorageError> {
        let info = command_snapshot(&command)?;
        if let (Some(id), Some(_)) = (info.id, info.image_number) {
            self.response(
                id,
                "EINVAL:cannot specify both i and I keys",
                info.quiet.unwrap_or(0.0),
                None,
            )?;
        } else if action_is(&info, b'd') {
            self.delete_command(&command)?;
        } else if action_is(&info, b'q') {
            // Source no-semicolon capability queries bypass transport validation.
            self.response(
                info.id.unwrap_or(0.0),
                "OK",
                info.quiet.unwrap_or(0.0),
                None,
            )?;
        } else if action_is(&info, b'p') {
            return self.placement(command);
        } else if let Some(id) = info.id {
            self.response(
                id,
                "EINVAL:unsupported action",
                info.quiet.unwrap_or(0.0),
                None,
            )?;
        }
        Ok(HandlerResult::Ready(true))
    }

    pub(super) fn payload_command(
        &self,
        command: CommandRef,
        owned: Option<Vec<u8>>,
        length: usize,
        decode_error: bool,
    ) -> Result<HandlerResult, StorageError> {
        let info = command_snapshot(&command)?;
        if action_is(&info, b't') {
            self.transmit(&command, owned, length, decode_error)?;
            let info = command_snapshot(&command)?;
            if direct(&info) {
                if let Some(id) = info.id {
                    if decode_error {
                        self.response(id, INVALID_BASE64, info.quiet.unwrap_or(0.0), None)?;
                    } else if length != 0 {
                        self.response(id, "OK", info.quiet.unwrap_or(0.0), None)?;
                    }
                }
            }
        } else if action_is(&info, b'T') {
            return self.transmit_display(command, owned, length, decode_error);
        } else if action_is(&info, b'q') {
            self.query(&command, length, decode_error)?;
        } else if action_is(&info, b'p') {
            return self.placement(command);
        } else if let Some(id) = info.id {
            self.response(
                id,
                "EINVAL:unsupported action",
                info.quiet.unwrap_or(0.0),
                None,
            )?;
        }
        Ok(HandlerResult::Ready(true))
    }

    fn transmit(
        &self,
        command: &CommandRef,
        owned: Option<Vec<u8>>,
        length: usize,
        decode_error: bool,
    ) -> Result<(), StorageError> {
        let info = command_snapshot(command)?;
        if !direct(&info) {
            if let Some(id) = info.id {
                self.response(id, UNSUPPORTED_MEDIUM, info.quiet.unwrap_or(0.0), None)?;
            }
            return Ok(());
        }
        if decode_error || length == 0 {
            return Ok(());
        }
        let id = info.id;
        let owned = owned.ok_or_else(|| StorageError::host(KittyProtocolError::MissingPayload))?;
        // Blob construction precedes the remaining metadata reads. A host may
        // reenter/mutate a retained command, so read those fields afterward.
        let data = self.factory.capture_owned(owned, "")?;
        let info = command_snapshot(command)?;
        self.storage.store_image(
            id,
            KittyImageBytes {
                data,
                width: info.width.unwrap_or(0.0),
                height: info.height.unwrap_or(0.0),
                format: info.format.unwrap_or(FORMAT_RGBA),
                compression: info.compression.unwrap_or_default(),
            },
        )?;
        Ok(())
    }

    fn transmit_display(
        &self,
        command: CommandRef,
        owned: Option<Vec<u8>>,
        length: usize,
        decode_error: bool,
    ) -> Result<HandlerResult, StorageError> {
        if decode_error {
            let info = command_snapshot(&command)?;
            if let Some(id) = info.id {
                self.response(id, INVALID_BASE64, info.quiet.unwrap_or(0.0), None)?;
            }
            return Ok(HandlerResult::Ready(true));
        }
        self.transmit(&command, owned, length, decode_error)?;
        let info = command_snapshot(&command)?;
        let id = if let Some(id) = info.id {
            id
        } else {
            self.storage.last_image_id()?
        };
        let Some(image) = self.storage.get_image(id)? else {
            return Ok(HandlerResult::Ready(true));
        };
        // Unsupported transport may still display previously stored data. Do not
        // silently improve that source behavior into an early return.
        let future = self.host.begin_display(KittyDisplayRequest {
            image,
            command: Rc::clone(&command),
            storage: Rc::clone(&self.storage),
            pixel_limit: self.options.pixel_limit,
        });
        let respond = command_snapshot(&command)?.id.is_some();
        Ok(display_result(
            future,
            Rc::clone(&self.host),
            command,
            id,
            respond,
            false,
        ))
    }

    fn placement(&self, command: CommandRef) -> Result<HandlerResult, StorageError> {
        let info = command_snapshot(&command)?;
        let Some(id) = info.id else {
            return Ok(HandlerResult::Ready(true));
        };
        let Some(image) = self.storage.get_image(id)? else {
            self.response(
                id,
                "ENOENT:image not found",
                info.quiet.unwrap_or(0.0),
                info.placement_id,
            )?;
            return Ok(HandlerResult::Ready(true));
        };
        let future = self.host.begin_display(KittyDisplayRequest {
            image,
            command: Rc::clone(&command),
            storage: Rc::clone(&self.storage),
            pixel_limit: self.options.pixel_limit,
        });
        Ok(display_result(
            future,
            Rc::clone(&self.host),
            command,
            id,
            true,
            true,
        ))
    }

    fn query(
        &self,
        command: &CommandRef,
        length: usize,
        decode_error: bool,
    ) -> Result<(), StorageError> {
        let info = command_snapshot(command)?;
        let id = info.id.unwrap_or(0.0);
        let quiet = info.quiet.unwrap_or(0.0);
        let response = if !direct(&info) {
            UNSUPPORTED_MEDIUM
        } else if decode_error {
            INVALID_BASE64
        } else if length == 0 || info.format.unwrap_or(FORMAT_RGBA) == FORMAT_PNG {
            "OK"
        } else {
            let width = info.width.unwrap_or(0.0);
            let height = info.height.unwrap_or(0.0);
            if !truthy(width) || !truthy(height) {
                "EINVAL:width and height required for raw pixel data"
            } else {
                // Negative dimensions and unknown formats retain the source
                // numeric comparison; raster validation is a later phase.
                let channels = if info.format.unwrap_or(FORMAT_RGBA) == FORMAT_RGBA {
                    4.0
                } else {
                    3.0
                };
                if (length as f64) < width * height * channels {
                    "EINVAL:insufficient pixel data"
                } else {
                    "OK"
                }
            }
        };
        self.response(id, response, quiet, None)
    }

    pub(super) fn delete_command(&self, command: &CommandRef) -> Result<(), StorageError> {
        let info = command_snapshot(command)?;
        let all = info.delete_selector.is_none()
            || is_ascii(info.delete_selector.as_deref(), b"a")
            || is_ascii(info.delete_selector.as_deref(), b"A");
        if all {
            self.state
                .try_borrow_mut()
                .map_err(handler_borrow)?
                .cleanup_pending()?;
            self.storage.delete_all()?;
        } else if is_ascii(info.delete_selector.as_deref(), b"i")
            || is_ascii(info.delete_selector.as_deref(), b"I")
        {
            if let Some(id) = info.id {
                {
                    let mut state = self.state.try_borrow_mut().map_err(handler_borrow)?;
                    if let Some(pending) = state.pending.get(id).cloned() {
                        let decoder = {
                            let decoder = pending.decoder.try_borrow().map_err(decoder_borrow)?;
                            Rc::clone(&decoder)
                        };
                        decoder.try_borrow_mut().map_err(decoder_borrow)?.release();
                    }
                    state.remove_pending(id);
                }
                self.storage.delete_by_id(id)?;
            }
        }
        // Other selectors are intentionally unsupported no-ops in upstream.
        Ok(())
    }
}

fn display_result(
    future: DisplayFuture,
    host: Rc<dyn KittyGraphicsHost>,
    command: CommandRef,
    id: f64,
    respond: bool,
    placement: bool,
) -> HandlerResult {
    HandlerResult::PendingFallible(Box::pin(async move {
        // Upstream _displayImage catches decode/render failures into false. The
        // later response listener failure remains a rejected promise, not false.
        let rendered = future.await.is_ok();
        if respond {
            let info = command_snapshot(&command).map_err(dispatch_error)?;
            response_to(
                host.as_ref(),
                id,
                if rendered { "OK" } else { RENDER_FAILED },
                info.quiet.unwrap_or(0.0),
                if placement { info.placement_id } else { None },
            )
            .map_err(dispatch_error)?;
        }
        Ok(true)
    }))
}
