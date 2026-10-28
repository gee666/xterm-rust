// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! CSI commands and the related DCS status-query registration.
use super::{register::command, TerminalError, TerminalState};
use crate::parser::{
    escape::{EscapeSequenceParser, FunctionIdentifier},
    handler::{DispatchError, HandlerResult},
};
use std::{cell::RefCell, rc::Rc};
pub(super) fn register(
    parser: &mut EscapeSequenceParser,
    state: &Rc<RefCell<TerminalState>>,
) -> Result<(), TerminalError> {
    for final_byte in [
        "@", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "P", "S", "T", "X",
        "Z", "^", "`", "a", "b", "d", "e", "f", "g", "r", "s", "u",
    ] {
        parser.register_csi_handler(
            FunctionIdentifier::new(final_byte),
            command(
                move |s, p| {
                    if final_byte == "H" {
                        s.update_windows_wrapping()?;
                    }
                    s.dispatch_csi(final_byte, p, false).map(|_| ())
                },
                Rc::clone(state),
            ),
        )?;
    }
    for (intermediates, final_byte, dispatch) in [
        (" ", "@", " @"),
        (" ", "A", " A"),
        ("'", "}", "'}"),
        ("'", "~", "'~"),
        ("\"", "q", "\"q"),
    ] {
        parser.register_csi_handler(
            FunctionIdentifier {
                prefix: None,
                intermediates: Some(intermediates),
                final_byte,
            },
            command(
                move |s, p| s.dispatch_csi(dispatch, p, false).map(|_| ()),
                Rc::clone(state),
            ),
        )?;
    }
    for final_byte in ["J", "K"] {
        parser.register_csi_handler(
            FunctionIdentifier {
                prefix: Some("?"),
                intermediates: None,
                final_byte,
            },
            command(
                move |s, p| s.dispatch_csi(final_byte, p, true).map(|_| ()),
                Rc::clone(state),
            ),
        )?;
    }
    parser.register_csi_handler(
        FunctionIdentifier::new("t"),
        command(|s, p| s.window_options(p), Rc::clone(state)),
    )?;
    for prefix in [None, Some(">")] {
        parser.register_csi_handler(
            FunctionIdentifier {
                prefix,
                intermediates: None,
                final_byte: "c",
            },
            command(
                move |s, p| s.device_attributes(p, prefix.is_some()),
                Rc::clone(state),
            ),
        )?;
    }
    for prefix in [None, Some("?")] {
        parser.register_csi_handler(
            FunctionIdentifier {
                prefix,
                intermediates: None,
                final_byte: "n",
            },
            command(
                move |s, p| s.device_status(p, prefix.is_some()),
                Rc::clone(state),
            ),
        )?;
    }
    parser.register_csi_handler(
        FunctionIdentifier {
            prefix: Some(">"),
            intermediates: None,
            final_byte: "q",
        },
        command(|s, p| s.xt_version(p), Rc::clone(state)),
    )?;
    parser.register_csi_handler(
        FunctionIdentifier {
            prefix: None,
            intermediates: Some(" "),
            final_byte: "q",
        },
        command(|s, p| s.cursor_style(p), Rc::clone(state)),
    )?;
    parser.register_csi_handler(
        FunctionIdentifier {
            prefix: None,
            intermediates: Some("!"),
            final_byte: "p",
        },
        command(|s, _| s.soft_reset(), Rc::clone(state)),
    )?;
    let shared = Rc::clone(state);
    parser.register_dcs_handler(
        FunctionIdentifier {
            prefix: None,
            intermediates: Some("$"),
            final_byte: "q",
        },
        Rc::new(RefCell::new(crate::parser::dcs::DcsHandler::new(
            move |data, _| {
                let result = shared.borrow_mut().status_string(data);
                if let Err(error) = result {
                    shared.borrow_mut().error = Some(error);
                    return HandlerResult::Failed(DispatchError::CallbackFailed);
                }
                match super::dispatch_events(&shared) {
                    Ok(()) => HandlerResult::Ready(true),
                    Err(error) => {
                        shared.borrow_mut().error = Some(error);
                        HandlerResult::Failed(DispatchError::CallbackFailed)
                    }
                }
            },
        ))),
    )?;
    parser.register_csi_handler(
        FunctionIdentifier::new("m"),
        command(
            |s, p| {
                let enabled = s.mode_option("vtExtensions", "kittySgrBoldFaintControl", true)?;
                for code in super::sgr::apply_with_options(&mut s.attributes, p, enabled) {
                    s.mode_debug(&format!("Unknown SGR attribute: {code}."))?;
                }
                Ok(())
            },
            Rc::clone(state),
        ),
    )?;
    for prefix in [None, Some("?")] {
        for final_byte in ["h", "l"] {
            parser.register_csi_handler(
                FunctionIdentifier {
                    prefix,
                    intermediates: None,
                    final_byte,
                },
                mode_command(prefix, final_byte, Rc::clone(state)),
            )?;
        }
        parser.register_csi_handler(
            FunctionIdentifier {
                prefix,
                intermediates: Some("$"),
                final_byte: "p",
            },
            command(
                move |s, p| s.dispatch_mode(prefix, "$p", p).map(|_| ()),
                Rc::clone(state),
            ),
        )?;
    }
    for prefix in ["=", "?", ">", "<"] {
        parser.register_csi_handler(
            FunctionIdentifier {
                prefix: Some(prefix),
                intermediates: None,
                final_byte: "u",
            },
            command(
                move |s, p| s.dispatch_mode(Some(prefix), "u", p).map(|_| ()),
                Rc::clone(state),
            ),
        )?;
    }
    Ok(())
}

// Set/reset has multiple independently observable parameters. Flush callbacks
// between them, and run logger callbacks before acquiring mutable state.
fn mode_command(
    prefix: Option<&'static str>,
    final_byte: &'static str,
    shared: Rc<RefCell<TerminalState>>,
) -> crate::parser::escape::CommandHandler {
    Rc::new(RefCell::new(
        move |params: &crate::parser::params::Params| {
            let result = apply_modes(&shared, prefix, final_byte, params);
            match result {
                Ok(()) => HandlerResult::Ready(true),
                Err(error) => {
                    if let Ok(mut state) = shared.try_borrow_mut() {
                        state.error = Some(error);
                    }
                    HandlerResult::Failed(DispatchError::CallbackFailed)
                }
            }
        },
    ))
}

fn apply_modes(
    shared: &Rc<RefCell<TerminalState>>,
    prefix: Option<&str>,
    final_byte: &str,
    params: &crate::parser::params::Params,
) -> Result<(), TerminalError> {
    use crate::{
        log_service::LogLevel,
        parser::params::{ParamValue, Params},
    };
    let enabled = final_byte == "h";
    for &mode in params.values() {
        if prefix == Some("?") {
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
        }
        // Existing Params values have already passed add_param validation.
        let single = Params::from_array(&[ParamValue::Param(mode)])
            .expect("validated source parameter fits one-parameter storage");
        {
            let mut state = shared
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?;
            if state.error.is_some() {
                return Err(TerminalError::Parse(
                    crate::parser::escape::ParseError::HandlerAborted,
                ));
            }
            state.dispatch_mode(prefix, final_byte, &single)?;
        }
        super::dispatch_events(shared)?;
    }
    Ok(())
}
