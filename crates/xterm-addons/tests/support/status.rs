use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_addons::status_terminal::{EscCallback, OscCallback, StatusTerminal};
use xterm_core::event::EmitError;
use xterm_core::lifecycle::{to_disposable, DisposableHandle};

type Registration<T> = (u16, Rc<Cell<bool>>, T);

/// Fixture registrations, not a terminal engine or byte-stream parser.
#[derive(Default)]
pub struct FixtureTerminal {
    osc: RefCell<Vec<Registration<OscCallback>>>,
    esc: RefCell<Vec<Registration<EscCallback>>>,
    pub replies: RefCell<Vec<String>>,
    pub disable_stdin: Cell<bool>,
}

impl FixtureTerminal {
    pub fn osc_units(&self, id: u16, data: &[u16]) -> Result<bool, EmitError> {
        let callbacks: Vec<_> = self
            .osc
            .borrow()
            .iter()
            .filter(|(key, live, _)| *key == id && live.get())
            .map(|(_, _, callback)| Rc::clone(callback))
            .collect();
        for callback in callbacks.iter().rev() {
            if callback(data)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub fn osc(&self, id: u16, data: &str) -> bool {
        self.osc_units(id, &data.encode_utf16().collect::<Vec<_>>())
            .unwrap()
    }
    pub fn ris(&self) -> bool {
        let callbacks: Vec<_> = self
            .esc
            .borrow()
            .iter()
            .filter(|(key, live, _)| *key == u16::from(b'c') && live.get())
            .map(|(_, _, callback)| Rc::clone(callback))
            .collect();
        for callback in callbacks.iter().rev() {
            if callback().unwrap() {
                return true;
            }
        }
        false
    }
}

impl StatusTerminal for FixtureTerminal {
    fn register_osc_handler(&self, id: u16, callback: OscCallback) -> DisposableHandle {
        let live = Rc::new(Cell::new(true));
        self.osc.borrow_mut().push((id, Rc::clone(&live), callback));
        to_disposable(move || live.set(false))
    }
    fn register_esc_handler(
        &self,
        final_byte: u8,
        callback: EscCallback,
    ) -> Result<DisposableHandle, xterm_core::parser::escape::RegistrarError> {
        xterm_core::parser::escape::FunctionIdentifier::new(&char::from(final_byte).to_string())
            .packed(true)?;
        let live = Rc::new(Cell::new(true));
        self.esc
            .borrow_mut()
            .push((u16::from(final_byte), Rc::clone(&live), callback));
        Ok(to_disposable(move || live.set(false)))
    }
    fn input(&self, data: &str, was_user_input: bool) -> Result<(), EmitError> {
        assert!(!was_user_input);
        if !self.disable_stdin.get() {
            self.replies.borrow_mut().push(data.to_owned());
        }
        Ok(())
    }
}
