//! Live engine adapter for OSC status/progress addons.
//!
//! Retain this adapter while addons are active. Cloneable registrars allow addon
//! activation from a callback without borrowing the parsing engine. Input replies
//! use a separately owned CoreService for the same reason.

use crate::status_terminal::{EscCallback, OscCallback, StatusTerminal};
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    core_service::CoreService,
    event::EmitError,
    lifecycle::DisposableHandle,
    parser::{
        escape::{CommandRegistrar, FunctionIdentifier, RegistrarError},
        osc::{OscHandler, OscRegistrar},
    },
    terminal::CoreTerminal,
};

/// One registration implementation shared by native and browser public hosts.
/// The owner keeps weak parser registrars valid without retaining callbacks.
pub struct StatusTerminalAdapter<T> {
    owner: Rc<T>,
    core: Rc<CoreService>,
    osc: OscRegistrar,
    esc: CommandRegistrar,
    input_error: RefCell<Option<EmitError>>,
}

pub type NativeStatusTerminal = StatusTerminalAdapter<RefCell<CoreTerminal>>;
pub type HeadlessStatusTerminal = StatusTerminalAdapter<xterm_core::headless::Terminal>;

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
            owner: engine,
            core,
            osc,
            esc,
            input_error: RefCell::new(None),
        }
    }
}

impl HeadlessStatusTerminal {
    pub fn from_terminal(
        owner: Rc<xterm_core::headless::Terminal>,
    ) -> Result<Self, xterm_core::headless::HeadlessError> {
        let core = owner.with_state(|state| Rc::clone(&state.core))?;
        let (osc, esc) = owner.addon_registrars()?;
        Ok(Self {
            owner,
            core,
            osc,
            esc,
            input_error: RefCell::new(None),
        })
    }
}

impl StatusTerminalAdapter<CoreService> {
    /// Private managed-add-on construction. Registration happens synchronously
    /// while the host Terminal is borrowed by TerminalAddon::activate. Retaining
    /// CoreService rather than Terminal avoids terminal -> addon -> terminal cycles.
    pub(crate) fn for_activation(
        terminal: &xterm_core::headless::Terminal,
    ) -> Result<Self, xterm_core::headless::HeadlessError> {
        let core = terminal.with_state(|state| Rc::clone(&state.core))?;
        let (osc, esc) = terminal.addon_registrars()?;
        Ok(Self {
            owner: core.clone(),
            core,
            osc,
            esc,
            input_error: RefCell::new(None),
        })
    }
}

impl<T> StatusTerminalAdapter<T> {
    pub fn owner(&self) -> &Rc<T> {
        &self.owner
    }
    /// Retains the same typed input cause returned through parser dispatch.
    pub fn take_input_error(&self) -> Option<EmitError> {
        self.input_error.borrow_mut().take()
    }
}

impl<T> StatusTerminal for StatusTerminalAdapter<T> {
    fn register_osc_handler(&self, id: u16, callback: OscCallback) -> DisposableHandle {
        self.osc
            .register_handler(
                f64::from(id),
                Rc::new(RefCell::new(OscHandler::new(move |data| {
                    callback(&data).into()
                }))),
            )
            // Public adapters retain the parser owner. The private managed path
            // registers only during activation with a borrowed live Terminal.
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
                let error = EmitError::callback(error);
                *self.input_error.borrow_mut() = Some(error.clone());
                Err(error)
            }
        }
    }
}
