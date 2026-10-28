// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{TerminalError, TerminalState};
use crate::{
    charset::builtin_charset,
    parser::{
        escape::{CommandHandler, EscapeSequenceParser, FunctionIdentifier, ParseError},
        handler::{DispatchError, HandlerResult},
        params::Params,
    },
};
use std::{cell::RefCell, rc::Rc};

pub(super) fn register(
    parser: &mut EscapeSequenceParser,
    state: &Rc<RefCell<TerminalState>>,
) -> Result<(), TerminalError> {
    let boundary = Rc::clone(state);
    parser.set_action_boundary_handler(move || {
        super::dispatch_events(&boundary).map_err(|error| {
            boundary.borrow_mut().error = Some(error);
            ParseError::HandlerAborted
        })
    });
    let shared = Rc::clone(state);
    let preceding = Rc::clone(&parser.preceding_join_state);
    parser.set_print_handler_checked(move |data| {
        let mut state = shared
            .try_borrow_mut()
            .map_err(|_| ParseError::HandlerAborted)?;
        if let Err(e) = state.print(data, &preceding) {
            state.error = Some(e);
            return Err(ParseError::HandlerAborted);
        }
        drop(state);
        super::dispatch_events(&shared).map_err(|e| {
            shared.borrow_mut().error = Some(e);
            ParseError::HandlerAborted
        })
    });
    for code in [
        0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x84, 0x85, 0x88,
    ] {
        let shared = Rc::clone(state);
        parser.set_execute_handler_checked(code, move |_| {
            let mut state = shared
                .try_borrow_mut()
                .map_err(|_| ParseError::HandlerAborted)?;
            if let Err(e) = state.execute(code) {
                state.error = Some(e);
                return Err(ParseError::HandlerAborted);
            }
            drop(state);
            super::dispatch_events(&shared).map(|()| true).map_err(|e| {
                shared.borrow_mut().error = Some(e);
                ParseError::HandlerAborted
            })
        });
    }
    for final_byte in [
        "7", "8", "D", "E", "H", "M", "=", ">", "c", "n", "o", "|", "}", "~",
    ] {
        let shared = Rc::clone(state);
        parser.register_esc_handler(
            FunctionIdentifier::new(final_byte),
            command(move |s, _| s.escape(final_byte), shared),
        )?;
    }
    for intermediate in ["(", ")", "*", "+", "-", ".", "/"] {
        for designator in [
            '0', 'A', 'B', '4', 'C', '5', 'R', 'Q', 'K', 'Y', 'E', '6', 'Z', 'H', '7', '=',
        ] {
            let final_byte = designator.to_string();
            let shared = Rc::clone(state);
            parser.register_esc_handler(
                FunctionIdentifier {
                    prefix: None,
                    intermediates: Some(intermediate),
                    final_byte: &final_byte,
                },
                command(
                    move |s, _| {
                        let level = match intermediate {
                            "(" => 0,
                            ")" | "-" => 1,
                            "*" | "." => 2,
                            "+" => 3,
                            _ => return Ok(()),
                        };
                        s.charset
                            .set_g_charset(level, builtin_charset(designator))
                            .map_err(|e| TerminalError::Buffer(e.to_string()))
                    },
                    shared,
                ),
            )?;
        }
    }
    for final_byte in ["@", "G"] {
        parser.register_esc_handler(
            FunctionIdentifier {
                prefix: None,
                intermediates: Some("%"),
                final_byte,
            },
            command(
                |s, _| {
                    s.charset.reset();
                    Ok(())
                },
                Rc::clone(state),
            ),
        )?;
    }
    parser.register_esc_handler(
        FunctionIdentifier {
            prefix: None,
            intermediates: Some("#"),
            final_byte: "8",
        },
        command(|s, _| s.alignment_pattern(), Rc::clone(state)),
    )?;
    super::commands::register(parser, state)?;
    super::osc::register(parser, state);
    Ok(())
}
pub(super) fn command(
    callback: impl Fn(&mut TerminalState, &Params) -> Result<(), TerminalError> + 'static,
    shared: Rc<RefCell<TerminalState>>,
) -> CommandHandler {
    Rc::new(RefCell::new(move |params: &Params| {
        let Ok(mut s) = shared.try_borrow_mut() else {
            return HandlerResult::Failed(DispatchError::CallbackFailed);
        };
        if s.error.is_some() {
            return HandlerResult::Failed(DispatchError::CallbackFailed);
        }
        if let Err(e) = callback(&mut s, params) {
            s.error = Some(e);
            return HandlerResult::Failed(DispatchError::CallbackFailed);
        }
        drop(s);
        match super::dispatch_events(&shared) {
            Ok(()) => HandlerResult::Ready(true),
            Err(e) => {
                shared.borrow_mut().error = Some(e);
                HandlerResult::Failed(DispatchError::CallbackFailed)
            }
        }
    }))
}
