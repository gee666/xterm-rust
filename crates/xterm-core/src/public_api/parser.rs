// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
//! Whole-sequence public callbacks over the real escape parser. Weak registrar
//! handles allow registration during callbacks without borrowing the engine.
//! Pending callbacks own their UTF-16 strings and parameter arrays.
use crate::{
    event::EmitError,
    lifecycle::DisposableHandle,
    parser::{
        apc::{ApcHandler, ApcRegistrar},
        dcs::{DcsHandler, DcsRegistrar},
        escape::{CommandRegistrar, EscapeSequenceParser, FunctionIdentifier, RegistrarError},
        handler::HandlerResult,
        osc::{OscHandler, OscRegistrar},
        params::{ParamValue, Params},
    },
};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

type WindowOptionGuard = dyn Fn(&Params) -> Result<bool, EmitError>;

/// Capture once while the engine is idle. Clones hold only weak references to
/// parser registries, so callbacks cannot retain their own parser through this
/// bundle. Explicit parser disposal leaves the registries reusable like JS.
#[derive(Clone)]
pub struct ParserRegistrations {
    csi: CommandRegistrar,
    esc: CommandRegistrar,
    dcs: DcsRegistrar,
    osc: OscRegistrar,
    apc: ApcRegistrar,
    window_option_guard: Option<Rc<WindowOptionGuard>>,
}
impl ParserRegistrations {
    pub fn new(parser: &EscapeSequenceParser) -> Self {
        Self {
            csi: parser.csi_registrar(),
            esc: parser.esc_registrar(),
            dcs: parser.dcs.registrar(),
            osc: parser.osc.registrar(),
            apc: parser.apc.registrar(),
            window_option_guard: None,
        }
    }
    pub fn osc_registrar(&self) -> OscRegistrar {
        self.osc.clone()
    }
    pub fn esc_registrar(&self) -> CommandRegistrar {
        self.esc.clone()
    }
    pub fn with_window_option_guard(
        mut self,
        guard: impl Fn(&Params) -> Result<bool, EmitError> + 'static,
    ) -> Self {
        self.window_option_guard = Some(Rc::new(guard));
        self
    }
}

/// A transient facade whose registrations outlive the facade. The raw-parser
/// constructor preserves its original exclusive lifetime; cached bundles use
/// `from_registrations` without borrowing an active parser or terminal state.
pub struct ParserApi<'a> {
    registrations: ParserRegistrations,
    lifetime: PhantomData<&'a mut EscapeSequenceParser>,
}
impl<'a> ParserApi<'a> {
    /// Terminal wrappers must attach a live windowOptions guard to their bundle.
    pub fn new(parser: &'a mut EscapeSequenceParser) -> Self {
        Self {
            registrations: ParserRegistrations::new(parser),
            lifetime: PhantomData,
        }
    }
    pub fn with_window_option_guard(
        parser: &'a mut EscapeSequenceParser,
        guard: impl Fn(&Params) -> Result<bool, EmitError> + 'static,
    ) -> Self {
        Self {
            registrations: ParserRegistrations::new(parser).with_window_option_guard(guard),
            lifetime: PhantomData,
        }
    }
    pub fn from_registrations(registrations: &'a ParserRegistrations) -> Self {
        Self {
            registrations: registrations.clone(),
            lifetime: PhantomData,
        }
    }
    pub fn register_csi_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        callback: impl FnMut(Vec<ParamValue>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, RegistrarError> {
        let guarded = id.final_byte == "t"
            && id.prefix.is_none_or(str::is_empty)
            && id.intermediates.is_none_or(str::is_empty);
        self.register_csi_packed(id.packed(false)?, guarded, callback)
    }
    /// JS accessors can change between the window-policy classification and
    /// identifier packing. The binding supplies the original classification;
    /// reconstructing it from the final packed byte would change source policy.
    pub fn register_csi_packed(
        &mut self,
        ident: u32,
        source_guarded: bool,
        mut callback: impl FnMut(Vec<ParamValue>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, RegistrarError> {
        let guard = source_guarded
            .then(|| self.registrations.window_option_guard.clone())
            .flatten();
        self.registrations.csi.register_packed(
            ident,
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
    ) -> Result<DisposableHandle, RegistrarError> {
        self.register_csi_handler(id, callback)
    }
    pub fn register_dcs_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        callback: impl FnMut(Vec<u16>, Vec<ParamValue>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, RegistrarError> {
        self.register_dcs_packed(id.packed(false)?, callback)
    }
    /// Register a source-packed identifier without rereading JS accessors.
    pub fn register_dcs_packed(
        &mut self,
        ident: u32,
        mut callback: impl FnMut(Vec<u16>, Vec<ParamValue>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, RegistrarError> {
        // JavaScript packs DCS identifiers into signed int32 bits.
        let ident = i32::from_ne_bytes(ident.to_ne_bytes());
        self.registrations
            .dcs
            .register_handler(
                ident,
                Rc::new(RefCell::new(DcsHandler::new(move |data, params| {
                    callback(data.to_vec(), params.to_array())
                }))),
            )
            .ok_or(RegistrarError::ParserDropped)
    }
    pub fn add_dcs_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        callback: impl FnMut(Vec<u16>, Vec<ParamValue>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, RegistrarError> {
        self.register_dcs_handler(id, callback)
    }
    pub fn register_esc_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        callback: impl FnMut() -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, RegistrarError> {
        self.register_esc_packed(id.packed(true)?, callback)
    }
    /// Register a source-packed identifier without rereading JS accessors.
    pub fn register_esc_packed(
        &mut self,
        ident: u32,
        mut callback: impl FnMut() -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, RegistrarError> {
        self.registrations
            .esc
            .register_packed(ident, Rc::new(RefCell::new(move |_: &Params| callback())))
    }
    pub fn add_esc_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        callback: impl FnMut() -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, RegistrarError> {
        self.register_esc_handler(id, callback)
    }
    pub fn register_osc_handler(
        &mut self,
        ident: f64,
        callback: impl FnMut(Vec<u16>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, RegistrarError> {
        self.registrations
            .osc
            .register_handler(ident, Rc::new(RefCell::new(OscHandler::new(callback))))
            .ok_or(RegistrarError::ParserDropped)
    }
    pub fn add_osc_handler(
        &mut self,
        ident: f64,
        callback: impl FnMut(Vec<u16>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, RegistrarError> {
        self.register_osc_handler(ident, callback)
    }
    pub fn register_apc_handler(
        &mut self,
        mut id: FunctionIdentifier<'_>,
        callback: impl FnMut(Vec<u16>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, RegistrarError> {
        id.prefix = None;
        self.register_apc_packed(id.packed(true)?, callback)
    }
    /// The JS binding performs the source APC prefix assignment before packing;
    /// a custom setter can leave a prefix present, so do not clear it again here.
    pub fn register_apc_packed(
        &mut self,
        ident: u32,
        callback: impl FnMut(Vec<u16>) -> HandlerResult + 'static,
    ) -> Result<DisposableHandle, RegistrarError> {
        self.registrations
            .apc
            .register_handler(ident, Rc::new(RefCell::new(ApcHandler::new(callback))))
            .ok_or(RegistrarError::ParserDropped)
    }
}
