// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Parser adapters for mode operations that can call back into the same terminal.
use super::super::{TerminalError, TerminalState};
use super::mode_callbacks::{apply, option_flag};
use crate::{
    log_service::LogLevel,
    parser::{
        escape::CommandHandler,
        handler::{DispatchError, HandlerResult},
        params::Params,
    },
};
use std::{cell::RefCell, rc::Rc};
type Shared = Rc<RefCell<TerminalState>>;

fn finish(shared: &Shared, result: Result<(), TerminalError>) -> HandlerResult {
    match result {
        Ok(()) => HandlerResult::Ready(true),
        Err(error) => match shared.try_borrow_mut() {
            Ok(mut state) => {
                state.error = Some(error);
                HandlerResult::Failed(DispatchError::CallbackFailed)
            }
            Err(_) => HandlerResult::Failed(DispatchError::Callback(
                crate::event::EmitError::callback(error),
            )),
        },
    }
}

pub(super) fn set_reset(
    prefix: Option<&'static str>,
    final_byte: &'static str,
    shared: Shared,
) -> CommandHandler {
    Rc::new(RefCell::new(move |params: &Params| {
        let result = set_parameters(&shared, prefix.is_some(), final_byte == "h", params);
        finish(&shared, result)
    }))
}

fn set_parameters(
    shared: &Shared,
    private: bool,
    enabled: bool,
    params: &Params,
) -> Result<(), TerminalError> {
    for &mode in params.values() {
        if shared
            .try_borrow()
            .map_err(|_| TerminalError::Borrowed)?
            .error
            .is_some()
        {
            return Err(TerminalError::Parse(
                crate::parser::escape::ParseError::HandlerAborted,
            ));
        }
        if private {
            log_mode(shared, mode, enabled)?;
        }
        if !apply(shared, private, mode, enabled)? {
            shared
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?
                .apply_state_mode(private, mode, enabled)?;
        }
        super::super::dispatch_events(shared)?;
    }
    Ok(())
}

fn log_mode(shared: &Shared, mode: i32, enabled: bool) -> Result<(), TerminalError> {
    let message = match mode {
        66 => Some(if enabled {
            "Serial port requested application keypad.".into()
        } else {
            "Switching back to normal keypad.".into()
        }),
        1005 | 1015 => Some(format!(
            "{} {mode} not supported (see #2507)",
            if enabled { "DECSET" } else { "DECRST" }
        )),
        _ => None,
    };
    if let Some(message) = message {
        let logger = Rc::clone(
            &shared
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .logger,
        );
        logger.log(
            LogLevel::Debug,
            &message.encode_utf16().collect::<Vec<_>>(),
            Vec::new(),
        )?;
    }
    Ok(())
}

pub(super) fn query(prefix: Option<&'static str>, shared: Shared) -> CommandHandler {
    Rc::new(RefCell::new(move |params: &Params| {
        let result = (|| {
            let mode = params.values().first().copied().unwrap_or(0);
            let (core, columns) = {
                let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
                (Rc::clone(&state.core), state.cols())
            };
            // Source retains dm object identity and the numeric cols before
            // reading nested options. Getters can reset modes or resize.
            let captured = (prefix.is_some() && mode == 9001).then(|| core.capture_private_modes());
            let gate = if prefix.is_some() && matches!(mode, 3 | 9001) {
                let options = Rc::clone(
                    &shared
                        .try_borrow()
                        .map_err(|_| TerminalError::Borrowed)?
                        .options,
                );
                Some(if mode == 3 {
                    option_flag(&options, "windowOptions", "setWinLines", false)?
                } else {
                    option_flag(&options, "vtExtensions", "win32InputMode", false)?
                })
            } else if matches!((prefix.is_some(), mode), (false, 20) | (true, 12)) {
                let options = Rc::clone(
                    &shared
                        .try_borrow()
                        .map_err(|_| TerminalError::Borrowed)?
                        .options,
                );
                Some(
                    options
                        .get(if prefix.is_some() {
                            "cursorBlink"
                        } else {
                            "convertEol"
                        })?
                        .is_truthy(),
                )
            } else {
                None
            };
            shared
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?
                .request_mode(
                    params,
                    prefix.is_none(),
                    gate,
                    columns,
                    captured.as_ref().map(|modes| modes.read()),
                )?;
            super::super::dispatch_events(&shared)
        })();
        finish(&shared, result)
    }))
}

pub(super) fn kitty(operation: &'static str, shared: Shared) -> CommandHandler {
    Rc::new(RefCell::new(move |params: &Params| {
        let result = (|| {
            let options = Rc::clone(
                &shared
                    .try_borrow()
                    .map_err(|_| TerminalError::Borrowed)?
                    .options,
            );
            let enabled = option_flag(&options, "vtExtensions", "kittyKeyboard", false)?;
            if enabled {
                shared
                    .try_borrow_mut()
                    .map_err(|_| TerminalError::Borrowed)?
                    .kitty_mode(operation, params)?;
            }
            super::super::dispatch_events(&shared)
        })();
        finish(&shared, result)
    }))
}
