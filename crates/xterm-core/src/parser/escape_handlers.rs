// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of EscapeSequenceParser handler registration.

use super::super::handler::registry::{HandlerRegistry, Registrar};
use super::{CommandHandler, EscapeSequenceParser, IdentifierError};
use crate::lifecycle::DisposableHandle;

/// Byte strings describing an ESC, CSI, DCS or APC function identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FunctionIdentifier<'a> {
    pub prefix: Option<&'a str>,
    pub intermediates: Option<&'a str>,
    pub final_byte: &'a str,
}

impl<'a> FunctionIdentifier<'a> {
    pub const fn new(final_byte: &'a str) -> Self {
        Self {
            prefix: None,
            intermediates: None,
            final_byte,
        }
    }

    /// CSI/DCS finals use 0x40..=0x7e, ESC/APC use 0x30..=0x7e.
    pub fn packed(self, esc_or_apc: bool) -> Result<u32, IdentifierError> {
        let mut value = 0u32;
        if let Some(prefix) = self.prefix.filter(|s| !s.is_empty()) {
            if prefix.encode_utf16().count() > 1 {
                return Err(IdentifierError::PrefixLength);
            }
            if prefix.len() != 1 || !(0x3c..=0x3f).contains(&prefix.as_bytes()[0]) {
                return Err(IdentifierError::PrefixRange);
            }
            value = u32::from(prefix.as_bytes()[0]);
        }
        if let Some(intermediates) = self.intermediates {
            if intermediates.encode_utf16().count() > 2 {
                return Err(IdentifierError::IntermediatesLength);
            }
            if !intermediates.bytes().all(|b| (0x20..=0x2f).contains(&b)) {
                return Err(IdentifierError::IntermediatesRange);
            }
            for byte in intermediates.bytes() {
                value = (value << 8) | u32::from(byte);
            }
        }
        let lower = if esc_or_apc { 0x30 } else { 0x40 };
        if self.final_byte.encode_utf16().count() != 1 {
            return Err(IdentifierError::FinalLength);
        }
        if self.final_byte.len() != 1 || !(lower..=0x7e).contains(&self.final_byte.as_bytes()[0]) {
            return Err(IdentifierError::FinalRange(lower));
        }
        Ok((value << 8) | u32::from(self.final_byte.as_bytes()[0]))
    }
}

type Callback = dyn FnMut(&super::Params) -> super::HandlerResult;
pub(super) type HandlerList = super::super::handler::registry::HandlerList<Callback>;
pub(super) type Registry = HandlerRegistry<u32, Callback>;

/// Registration-only handle that can be used from an executing callback.
/// It never borrows terminal state or the parser action loop.
#[derive(Clone)]
pub struct CommandRegistrar {
    registry: Registrar<u32, Callback>,
    esc: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistrarError {
    Identifier(IdentifierError),
    ParserDropped,
}
impl std::fmt::Display for RegistrarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(error) => error.fmt(f),
            Self::ParserDropped => f.write_str("parser no longer exists"),
        }
    }
}
impl std::error::Error for RegistrarError {}
impl From<IdentifierError> for RegistrarError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl CommandRegistrar {
    pub fn register(
        &self,
        id: FunctionIdentifier<'_>,
        handler: CommandHandler,
    ) -> Result<DisposableHandle, RegistrarError> {
        self.register_packed(id.packed(self.esc)?, handler)
    }
    /// Use a key already computed by a binding's identifier reader. Re-reading
    /// the original object here would change JavaScript getter/Proxy effects.
    pub fn register_packed(
        &self,
        id: u32,
        handler: CommandHandler,
    ) -> Result<DisposableHandle, RegistrarError> {
        self.registry
            .register(id, handler)
            .ok_or(RegistrarError::ParserDropped)
    }
    pub fn clear(&self, id: FunctionIdentifier<'_>) -> Result<(), RegistrarError> {
        self.clear_packed(id.packed(self.esc)?)
    }
    pub fn clear_packed(&self, id: u32) -> Result<(), RegistrarError> {
        self.registry
            .clear(&id)
            .ok_or(RegistrarError::ParserDropped)
    }
}

impl EscapeSequenceParser {
    pub fn csi_registrar(&self) -> CommandRegistrar {
        CommandRegistrar {
            registry: self.csi.registrar(),
            esc: false,
        }
    }
    pub fn esc_registrar(&self) -> CommandRegistrar {
        CommandRegistrar {
            registry: self.esc.registrar(),
            esc: true,
        }
    }
    pub fn register_csi_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        handler: CommandHandler,
    ) -> Result<DisposableHandle, IdentifierError> {
        Ok(self.csi.register(id.packed(false)?, handler))
    }
    pub fn register_esc_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        handler: CommandHandler,
    ) -> Result<DisposableHandle, IdentifierError> {
        Ok(self.esc.register(id.packed(true)?, handler))
    }
    pub fn clear_csi_handler(&mut self, id: FunctionIdentifier<'_>) -> Result<(), IdentifierError> {
        self.csi.clear(&id.packed(false)?);
        Ok(())
    }
    pub fn clear_esc_handler(&mut self, id: FunctionIdentifier<'_>) -> Result<(), IdentifierError> {
        self.esc.clear(&id.packed(true)?);
        Ok(())
    }
    pub fn set_print_handler(&mut self, mut handler: impl FnMut(&[u32]) + 'static) {
        self.set_print_handler_checked(move |data| {
            handler(data);
            Ok(())
        });
    }
    /// A backend may stop parsing on a recoverable failure instead of panicking.
    pub fn set_print_handler_checked(
        &mut self,
        handler: impl FnMut(&[u32]) -> Result<(), super::ParseError> + 'static,
    ) {
        self.print = Some(Box::new(handler));
    }
    pub fn clear_print_handler(&mut self) {
        self.print = None;
    }
    /// The fast executable path supplies the code; the table path supplies None.
    pub fn set_execute_handler(
        &mut self,
        code: u32,
        mut handler: impl FnMut(Option<u32>) -> bool + 'static,
    ) {
        self.set_execute_handler_checked(code, move |value| Ok(handler(value)));
    }
    pub fn set_execute_handler_checked(
        &mut self,
        code: u32,
        handler: impl FnMut(Option<u32>) -> Result<bool, super::ParseError> + 'static,
    ) {
        let handler: super::ExecuteHandler = Box::new(handler);
        let handler: super::ExecuteHandle = std::rc::Rc::new(std::cell::RefCell::new(handler));
        let removed = self.execute.borrow_mut().insert(code, handler);
        drop(removed);
    }
    pub fn clear_execute_handler(&mut self, code: u32) {
        let removed = self.execute.borrow_mut().remove(&code);
        drop(removed);
    }
    pub fn set_execute_handler_fallback(&mut self, mut handler: impl FnMut(u32) + 'static) {
        self.set_execute_handler_fallback_checked(move |code| {
            handler(code);
            Ok(())
        });
    }
    pub fn set_execute_handler_fallback_checked(
        &mut self,
        handler: impl FnMut(u32) -> Result<(), super::ParseError> + 'static,
    ) {
        self.execute_fallback = Some(Box::new(handler));
    }
    pub fn set_csi_handler_fallback(
        &mut self,
        mut handler: impl FnMut(u32, &super::Params) + 'static,
    ) {
        self.set_csi_handler_fallback_checked(move |id, params| {
            handler(id, params);
            Ok(())
        });
    }
    pub fn set_csi_handler_fallback_checked(
        &mut self,
        handler: impl FnMut(u32, &super::Params) -> Result<(), super::ParseError> + 'static,
    ) {
        self.csi_fallback = Some(Box::new(handler));
    }
    pub fn set_esc_handler_fallback(&mut self, mut handler: impl FnMut(u32) + 'static) {
        self.set_esc_handler_fallback_checked(move |id| {
            handler(id);
            Ok(())
        });
    }
    pub fn set_esc_handler_fallback_checked(
        &mut self,
        handler: impl FnMut(u32) -> Result<(), super::ParseError> + 'static,
    ) {
        self.esc_fallback = Some(Box::new(handler));
    }
    /// Return true to abort this chunk. Other state changes are unsupported upstream.
    pub fn set_error_handler(
        &mut self,
        mut handler: impl FnMut(super::ParsingState<'_>) -> bool + 'static,
    ) {
        self.set_error_handler_checked(move |state| Ok(handler(state)));
    }
    pub fn set_error_handler_checked(
        &mut self,
        handler: impl FnMut(super::ParsingState<'_>) -> Result<bool, super::ParseError> + 'static,
    ) {
        self.error = Some(Box::new(handler));
    }
    pub fn clear_error_handler(&mut self) {
        self.error = None;
    }
    pub fn register_dcs_handler(
        &mut self,
        id: FunctionIdentifier<'_>,
        handler: super::super::dcs::HandlerHandle,
    ) -> Result<DisposableHandle, IdentifierError> {
        Ok(self.dcs.register_handler(id.packed(false)? as i32, handler))
    }
    pub fn clear_dcs_handler(&mut self, id: FunctionIdentifier<'_>) -> Result<(), IdentifierError> {
        self.dcs.clear_handler(id.packed(false)? as i32);
        Ok(())
    }
    pub fn register_apc_handler(
        &mut self,
        mut id: FunctionIdentifier<'_>,
        handler: super::super::apc::HandlerHandle,
    ) -> Result<DisposableHandle, IdentifierError> {
        id.prefix = None;
        Ok(self.apc.register_handler(id.packed(true)?, handler))
    }
    pub fn clear_apc_handler(
        &mut self,
        mut id: FunctionIdentifier<'_>,
    ) -> Result<(), IdentifierError> {
        id.prefix = None;
        self.apc.clear_handler(id.packed(true)?);
        Ok(())
    }
    pub fn ident_to_string(mut id: u32) -> String {
        let mut bytes = Vec::new();
        while id != 0 {
            bytes.push((id & 0xff) as u8);
            id >>= 8;
        }
        bytes.into_iter().rev().map(char::from).collect()
    }
}
