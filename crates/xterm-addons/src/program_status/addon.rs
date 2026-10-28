// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of addon-program-status/ProgramStatusAddon.ts.

use super::{parse_report_utf16, ProgramStatusStore, StatusRecord, StatusReport};
use crate::status_terminal::StatusTerminal;
use std::cell::RefCell;
use std::rc::Rc;
use xterm_core::event::{EmitError, Emitter, Event};
use xterm_core::lifecycle::{Disposable, DisposableStore};
use xterm_core::parser::escape::RegistrarError;

const OSC_IDENT: u16 = 7501;
const REPLY: &str = "\x1b]7501;?\x1b\\";
type Snapshot = Rc<[StatusRecord]>;

#[derive(Default)]
struct Inner {
    store: RefCell<ProgramStatusStore>,
    change: Emitter<Snapshot>,
}

impl Inner {
    fn fire(&self) -> Result<(), EmitError> {
        let snapshot = self.store.borrow_mut().records();
        self.change.fire(&snapshot)
    }

    fn clear(&self, id: &str) -> Result<(), EmitError> {
        let changed = self.store.borrow_mut().clear(id);
        if changed {
            self.fire()?;
        }
        Ok(())
    }
}

/// Owns parser registrations. Dropping the addon also unregisters them.
#[derive(Default)]
pub struct ProgramStatusAddon {
    inner: Rc<Inner>,
    registrations: DisposableStore,
    terminals: RefCell<Vec<Rc<dyn StatusTerminal>>>,
}

impl ProgramStatusAddon {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn records(&self) -> Snapshot {
        self.inner.store.borrow_mut().records()
    }
    pub fn on_change(&self) -> Event<Snapshot> {
        self.inner.change.event()
    }

    pub fn activate(&self, terminal: Rc<dyn StatusTerminal>) -> Result<(), RegistrarError> {
        let weak = Rc::downgrade(&self.inner);
        let input = Rc::downgrade(&terminal);
        self.registrations.add(terminal.register_osc_handler(
            OSC_IDENT,
            Rc::new(move |data| {
                let Some(inner) = weak.upgrade() else {
                    return Ok(false);
                };
                if data == [u16::from(b'?')] {
                    if let Some(terminal) = input.upgrade() {
                        terminal.input(REPLY, false)?;
                    }
                    return Ok(true);
                }
                match parse_report_utf16(data) {
                    Some(StatusReport::Clear(id)) => inner.clear(&id)?,
                    Some(StatusReport::Record(record)) => {
                        inner.store.borrow_mut().replace(record);
                        inner.fire()?;
                    }
                    None => {}
                }
                Ok(true)
            }),
        ));
        let weak = Rc::downgrade(&self.inner);
        self.registrations.add(terminal.register_esc_handler(
            b'c',
            Rc::new(move || {
                if let Some(inner) = weak.upgrade() {
                    inner.clear("")?;
                }
                Ok(false)
            }),
        )?);
        if !self.registrations.is_disposed() {
            self.terminals.borrow_mut().push(terminal);
        }
        Ok(())
    }

    pub fn clear(&self, id: &str) -> Result<(), EmitError> {
        self.inner.clear(id)
    }

    pub fn process_exited(&self) -> Result<(), EmitError> {
        let changed = self.inner.store.borrow_mut().remove_active_records();
        if changed {
            self.inner.fire()?;
        }
        Ok(())
    }

    pub fn prompt_started(&self) -> Result<(), EmitError> {
        self.process_exited()
    }
}

impl Disposable for ProgramStatusAddon {
    fn dispose(&self) {
        self.inner.store.borrow_mut().clear("");
        self.registrations.dispose();
        self.terminals.borrow_mut().clear();
        self.inner.change.dispose();
    }
}

impl Drop for ProgramStatusAddon {
    fn drop(&mut self) {
        self.dispose();
    }
}
