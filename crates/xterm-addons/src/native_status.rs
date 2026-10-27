//! Live engine adapter for OSC status/progress addons.
//!
//! Retain this adapter while addons are active. Cloneable registrars allow addon
//! activation from a callback without borrowing the parsing engine. Input replies
//! use a separately owned CoreService for the same reason.

use crate::status_terminal::{EscCallback, OscCallback, StatusTerminal};
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    core_service::{CoreError, CoreService},
    event::EmitError,
    lifecycle::DisposableHandle,
    parser::{
        escape::{CommandRegistrar, FunctionIdentifier, RegistrarError},
        osc::{OscHandler, OscRegistrar},
    },
    terminal::CoreTerminal,
};

pub struct NativeStatusTerminal {
    engine: Rc<RefCell<CoreTerminal>>,
    core: Rc<CoreService>,
    osc: OscRegistrar,
    esc: CommandRegistrar,
    input_error: RefCell<Option<CoreError>>,
}

impl NativeStatusTerminal {
    /// Construct outside parser dispatch, without an outstanding engine borrow.
    pub fn new(engine: Rc<RefCell<CoreTerminal>>) -> Self {
        let core = Rc::clone(&engine.borrow().state().core);
        let (osc, esc) = {
            let mut engine = engine.borrow_mut();
            let parser = engine.parser_mut();
            (parser.osc.registrar(), parser.esc_registrar())
        };
        Self {
            engine,
            core,
            osc,
            esc,
            input_error: RefCell::new(None),
        }
    }
    pub fn engine(&self) -> &Rc<RefCell<CoreTerminal>> {
        &self.engine
    }
    /// The parser reports CallbackFailed. This retains its original input cause.
    pub fn take_input_error(&self) -> Option<CoreError> {
        self.input_error.borrow_mut().take()
    }
}

impl StatusTerminal for NativeStatusTerminal {
    fn register_osc_handler(&self, id: u16, callback: OscCallback) -> DisposableHandle {
        self.osc
            .register_handler(
                f64::from(id),
                Rc::new(RefCell::new(OscHandler::new(move |data| {
                    callback(&data).into()
                }))),
            )
            // The adapter owns the engine, which owns this parser and registry.
            // Disposal clears handlers but does not drop that registry.
            .expect("owned engine keeps OSC parser registry alive")
    }
    fn register_esc_handler(
        &self,
        final_byte: u8,
        callback: EscCallback,
    ) -> Result<DisposableHandle, RegistrarError> {
        self.esc.register(
            FunctionIdentifier::new(&char::from(final_byte).to_string()),
            Rc::new(RefCell::new(
                move |_: &xterm_core::parser::params::Params| callback().into(),
            )),
        )
    }
    fn input(&self, data: &str, was_user_input: bool) -> Result<(), EmitError> {
        match self
            .core
            .trigger_data_event(&data.encode_utf16().collect::<Vec<_>>(), was_user_input)
        {
            Ok(()) => Ok(()),
            Err(error) => {
                *self.input_error.borrow_mut() = Some(error);
                Err(EmitError)
            }
        }
    }
}
