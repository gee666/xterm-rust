//! Native public loadAddon integration. Adapters retain services, not their
//! parent terminal, so a terminal that owns its addons has no ownership cycle.
use crate::{
    native_status::StatusTerminalAdapter, program_status::ProgramStatusAddon,
    progress::ProgressAddon,
};
use std::rc::Rc;
use xterm_core::{
    event::EmitError,
    headless::{HeadlessError, Terminal},
    lifecycle::Disposable,
    public_api::TerminalAddon,
};

impl TerminalAddon<Terminal> for ProgramStatusAddon {
    type Error = HeadlessError;
    fn activate(&self, terminal: &Terminal) -> Result<(), Self::Error> {
        let adapter = Rc::new(StatusTerminalAdapter::for_activation(terminal)?);
        ProgramStatusAddon::activate(self, adapter)
            .map_err(|error| HeadlessError::Dispatch(EmitError::callback(error)))
    }
    fn dispose(&self) -> Result<(), Self::Error> {
        Disposable::dispose(self);
        Ok(())
    }
}
impl TerminalAddon<Terminal> for ProgressAddon {
    type Error = HeadlessError;
    fn activate(&self, terminal: &Terminal) -> Result<(), Self::Error> {
        let adapter = Rc::new(StatusTerminalAdapter::for_activation(terminal)?);
        ProgressAddon::activate(self, adapter);
        Ok(())
    }
    fn dispose(&self) -> Result<(), Self::Error> {
        Disposable::dispose(self);
        Ok(())
    }
}
