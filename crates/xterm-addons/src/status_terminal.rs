//! Narrow adapter required by the program-status and progress addons.
//!
//! The terminal calls handlers synchronously, newest first, before parsing the
//! next sequence. Returning false delegates to older handlers and core behavior.
//! The adapter must not hold a mutable terminal borrow while invoking callbacks.

use std::rc::Rc;
use xterm_core::event::EmitError;
use xterm_core::lifecycle::DisposableHandle;
use xterm_core::parser::escape::RegistrarError;

pub type OscCallback = Rc<dyn Fn(&[u16]) -> Result<bool, EmitError>>;
pub type EscCallback = Rc<dyn Fn() -> Result<bool, EmitError>>;

pub trait StatusTerminal {
    fn register_osc_handler(&self, id: u16, callback: OscCallback) -> DisposableHandle;
    /// Register an ESC final byte without prefix or intermediates.
    fn register_esc_handler(
        &self,
        final_byte: u8,
        callback: EscCallback,
    ) -> Result<DisposableHandle, RegistrarError>;
    /// Respect disableStdin. `was_user_input=false` must not scroll to bottom.
    fn input(&self, data: &str, was_user_input: bool) -> Result<(), EmitError>;
}
