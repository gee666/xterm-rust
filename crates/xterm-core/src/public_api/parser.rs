// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
//! Whole-sequence public callbacks over the real escape parser. Payloads are
//! owned UTF-16 units, preserving JavaScript strings including lone surrogates.
//! The parser, not this borrowed adapter, owns registrations and pending work.

use crate::{
    event::EmitError,
    lifecycle::DisposableHandle,
    parser::{
        apc::ApcHandler,
        dcs::DcsHandler,
        escape::{EscapeSequenceParser, FunctionIdentifier, IdentifierError},
        handler::HandlerResult,
        osc::OscHandler,
        params::{ParamValue, Params},
    },
};
use std::{cell::RefCell, rc::Rc};

type WindowOptionGuard = dyn Fn(&Params) -> Result<bool, EmitError>;

pub struct ParserApi<'a> {
    parser: &'a mut EscapeSequenceParser,
    window_option_guard: Option<Rc<WindowOptionGuard>>,
}
impl<'a> ParserApi<'a> {
    /// Raw parser adapter without terminal policy. Terminal wrappers must use
    /// `with_window_option_guard` to apply live windowOptions before CSI t hooks.
    pub fn new(parser: &'a mut EscapeSequenceParser) -> Self {
        Self {
            parser,
            window_option_guard: None,
        }
    }
    pub fn with_window_option_guard(
        parser: &'a mut EscapeSequenceParser,
        guard: impl Fn(&Params) -> Result<bool, EmitError> + 'static,
    ) -> Self {
        Self {
            parser,
            window_option_guard: Some(Rc::new(guard)),
        }
    }

    pub fn register_csi_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        mut callback: impl FnMut(Vec<ParamValue>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, IdentifierError> {
        let guard = (id.final_byte == "t"
            && id.prefix.is_none_or(str::is_empty)
            && id.intermediates.is_none_or(str::is_empty))
        .then(|| self.window_option_guard.clone())
        .flatten();
        self.parser.register_csi_handler(
            id,
            Rc::new(RefCell::new(move |params: &Params| {
                if let Some(guard) = &guard {
                    match guard(params) {
                        Ok(true) => {}
                        Ok(false) => return HandlerResult::Ready(true),
                        Err(error) => return HandlerResult::from(Err(error)),
                    }
                }
                callback(params.to_array())
            })),
        )
    }
    pub fn add_csi_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        callback: impl FnMut(Vec<ParamValue>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, IdentifierError> {
        self.register_csi_handler(id, callback)
    }
    pub fn register_dcs_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        mut callback: impl FnMut(Vec<u16>, Vec<ParamValue>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, IdentifierError> {
        self.parser.register_dcs_handler(
            id,
            Rc::new(RefCell::new(DcsHandler::new(move |data, params| {
                callback(data.to_vec(), params.to_array())
            }))),
        )
    }
    pub fn add_dcs_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        callback: impl FnMut(Vec<u16>, Vec<ParamValue>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, IdentifierError> {
        self.register_dcs_handler(id, callback)
    }
    pub fn register_esc_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        mut callback: impl FnMut() -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, IdentifierError> {
        self.parser.register_esc_handler(
            id,
            Rc::new(RefCell::new(move |_: &crate::parser::params::Params| {
                callback()
            })),
        )
    }
    pub fn add_esc_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        callback: impl FnMut() -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, IdentifierError> {
        self.register_esc_handler(id, callback)
    }
    pub fn register_osc_handler(
        &mut self,
        ident: f64,
        callback: impl FnMut(Vec<u16>) -> HandlerResult + 'static,
    ) -> DisposableHandle {
        self.parser
            .osc
            .register_handler(ident, Rc::new(RefCell::new(OscHandler::new(callback))))
    }
    pub fn add_osc_handler(
        &mut self,
        ident: f64,
        callback: impl FnMut(Vec<u16>) -> HandlerResult + 'static,
    ) -> DisposableHandle {
        self.register_osc_handler(ident, callback)
    }
    pub fn register_apc_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        callback: impl FnMut(Vec<u16>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, IdentifierError> {
        self.parser
            .register_apc_handler(id, Rc::new(RefCell::new(ApcHandler::new(callback))))
    }
}
