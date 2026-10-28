// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! SGR callback boundaries cannot use the ordinary whole-command state borrow.
use super::{
    sgr::{SgrAction, SgrCursor},
    TerminalError, TerminalState,
};
use crate::{
    buffer::AttributeData,
    event::EmitError,
    log_service::{LogLevel, LogParameter, LogValue},
    options::OptionValue,
    parser::{
        escape::CommandHandler,
        handler::{DispatchError, HandlerResult},
        params::Params,
    },
};
use std::{cell::RefCell, rc::Rc};

type AttributeIdentity = Rc<RefCell<Option<AttributeData>>>;

#[cfg(test)]
#[path = "sgr_dispatch_tests.rs"]
mod tests;

pub(super) fn handler(shared: Rc<RefCell<TerminalState>>) -> CommandHandler {
    Rc::new(RefCell::new(move |params: &Params| {
        match shared.try_borrow() {
            Ok(state) if state.error.is_some() => {
                return HandlerResult::Failed(DispatchError::CallbackFailed)
            }
            Err(_) => {
                return HandlerResult::Failed(DispatchError::Callback(EmitError::callback(
                    TerminalError::Borrowed,
                )))
            }
            Ok(_) => {}
        }
        let result = apply(&shared, params).and_then(|()| super::dispatch_events(&shared));
        match result {
            Ok(()) => HandlerResult::Ready(true),
            Err(error) => {
                if let Ok(mut state) = shared.try_borrow_mut() {
                    state.error = Some(error);
                    HandlerResult::Failed(DispatchError::CallbackFailed)
                } else {
                    HandlerResult::Failed(DispatchError::Callback(EmitError::callback(error)))
                }
            }
        }
    }))
}

fn apply(shared: &Rc<RefCell<TerminalState>>, params: &Params) -> Result<(), TerminalError> {
    let (identity, options, logger) = {
        let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        (
            Rc::clone(&state.attributes_identity),
            Rc::clone(&state.options),
            Rc::clone(&state.logger),
        )
    };
    let mut cursor = SgrCursor::new(params);
    loop {
        let action = with_target(shared, &identity, |attr| cursor.next(attr))?;
        let code = match action {
            SgrAction::Done => return Ok(()),
            SgrAction::Unknown(code) => Some(code),
            SgrAction::Kitty(control) => {
                // Both property access and logging can reenter/reset the terminal.
                let bag = options.get("vtExtensions")?;
                let value = match bag {
                    OptionValue::Null | OptionValue::Undefined => OptionValue::Undefined,
                    value => value.get_property("kittySgrBoldFaintControl")?,
                };
                let enabled = match value {
                    OptionValue::Null | OptionValue::Undefined => true,
                    value => value.is_truthy(),
                };
                with_target(shared, &identity, |attr| control.apply(attr, enabled))?
            }
        };
        if let Some(code) = code {
            logger.log(
                LogLevel::Debug,
                &"Unknown SGR attribute: %d."
                    .encode_utf16()
                    .collect::<Vec<_>>(),
                vec![LogParameter::Value(LogValue::Value(OptionValue::Number(
                    f64::from(code),
                )))],
            )?;
        }
    }
}

// Print runs retain the same attribute-object target across callback resets.
pub(super) fn with_target<T>(
    shared: &Rc<RefCell<TerminalState>>,
    identity: &AttributeIdentity,
    change: impl FnOnce(&mut AttributeData) -> T,
) -> Result<T, TerminalError> {
    let mut state = shared
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?;
    if Rc::ptr_eq(identity, &state.attributes_identity) {
        return Ok(change(&mut state.attributes));
    }
    drop(state);
    let mut detached = identity
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?;
    let attr = detached.as_mut().ok_or(TerminalError::InvalidParameter(
        "replaced SGR attribute object was not retained",
    ))?;
    Ok(change(attr))
}
