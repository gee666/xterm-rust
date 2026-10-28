// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! CSI commands and the related DCS status-query registration.
#[path = "mode_callbacks.rs"]
pub(super) mod mode_callbacks;
#[path = "mode_handlers.rs"]
mod mode_handlers;
use super::{
    register::{command, shared_command},
    TerminalError, TerminalState,
};
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
        "@", "A", "B", "C", "D", "E", "F", "G", "H", "I", "K", "P", "X", "Z", "`", "a", "d", "e",
        "f", "g", "r", "s", "u",
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
    for final_byte in ["L", "M", "S", "T", "^"] {
        parser.register_csi_handler(
            FunctionIdentifier::new(final_byte),
            shared_command(
                move |s, p| super::csi_edits::rows(s, p, final_byte),
                Rc::clone(state),
            ),
        )?;
    }
    parser.register_csi_handler(
        FunctionIdentifier::new("J"),
        shared_command(
            |s, p| super::csi_erase::erase_display(s, p, false),
            Rc::clone(state),
        ),
    )?;
    parser.register_csi_handler(
        FunctionIdentifier::new("b"),
        repeat_command(Rc::clone(state)),
    )?;
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
            if final_byte == "J" {
                shared_command(
                    |s, p| super::csi_erase::erase_display(s, p, true),
                    Rc::clone(state),
                )
            } else {
                command(
                    move |s, p| s.dispatch_csi(final_byte, p, true).map(|_| ()),
                    Rc::clone(state),
                )
            },
        )?;
    }
    parser.register_csi_handler(
        FunctionIdentifier::new("t"),
        shared_command(super::window::window_options, Rc::clone(state)),
    )?;
    for prefix in [None, Some(">")] {
        parser.register_csi_handler(
            FunctionIdentifier {
                prefix,
                intermediates: None,
                final_byte: "c",
            },
            shared_command(
                move |s, p| super::reply_callbacks::device_attributes(s, p, prefix.is_some()),
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
        shared_command(
            |s, _| super::reply_callbacks::soft_reset(s),
            Rc::clone(state),
        ),
    )?;
    let shared = Rc::clone(state);
    parser.register_dcs_handler(
        FunctionIdentifier {
            prefix: None,
            intermediates: Some("$"),
            final_byte: "q",
        },
        Rc::new(RefCell::new(crate::parser::dcs::DcsHandler::new(
            move |data, _| match super::reply_callbacks::status_string(&shared, data)
                .and_then(|()| super::dispatch_events(&shared))
            {
                Ok(()) => HandlerResult::Ready(true),
                Err(error) => HandlerResult::Failed(DispatchError::Callback(
                    crate::event::EmitError::callback(error),
                )),
            },
        ))),
    )?;
    parser.register_csi_handler(
        FunctionIdentifier::new("m"),
        super::sgr_dispatch::handler(Rc::clone(state)),
    )?;
    for prefix in [None, Some("?")] {
        for final_byte in ["h", "l"] {
            parser.register_csi_handler(
                FunctionIdentifier {
                    prefix,
                    intermediates: None,
                    final_byte,
                },
                mode_handlers::set_reset(prefix, final_byte, Rc::clone(state)),
            )?;
        }
        parser.register_csi_handler(
            FunctionIdentifier {
                prefix,
                intermediates: Some("$"),
                final_byte: "p",
            },
            mode_handlers::query(prefix, Rc::clone(state)),
        )?;
    }
    for prefix in ["=", "?", ">", "<"] {
        parser.register_csi_handler(
            FunctionIdentifier {
                prefix: Some(prefix),
                intermediates: None,
                final_byte: "u",
            },
            mode_handlers::kitty(prefix, Rc::clone(state)),
        )?;
    }
    Ok(())
}

// REP can invoke provider/a11y callbacks. Prepare its data under a short borrow,
// then run the same unborrowed print path used for ordinary parser text.
fn repeat_command(shared: Rc<RefCell<TerminalState>>) -> crate::parser::escape::CommandHandler {
    Rc::new(RefCell::new(
        move |params: &crate::parser::params::Params| {
            let result = (|| {
                let (data, preceding) = {
                    let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
                    (
                        state.repeat_data(params)?,
                        Rc::clone(&state.preceding_join_state),
                    )
                };
                if let Some(data) = data {
                    super::print::print(&shared, &data, &preceding)?;
                }
                super::dispatch_events(&shared)
            })();
            match result {
                Ok(()) => HandlerResult::Ready(true),
                Err(error) => {
                    if let Ok(mut state) = shared.try_borrow_mut() {
                        state.error = Some(error);
                        HandlerResult::Failed(DispatchError::CallbackFailed)
                    } else {
                        HandlerResult::Failed(DispatchError::Callback(
                            crate::event::EmitError::callback(error),
                        ))
                    }
                }
            }
        },
    ))
}
