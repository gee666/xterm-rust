// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{EventHandler, TerminalEvent};
use crate::event::EmitError;
use std::{cell::RefCell, rc::Rc};

/// Service emitters already released their own state. Do not reacquire terminal
/// state merely to forward their callback, as observers may synchronously reset.
pub(super) fn forward_event(
    queue: &Rc<RefCell<Vec<TerminalEvent>>>,
    slot: &Rc<RefCell<Option<EventHandler>>>,
    event: TerminalEvent,
) -> Result<(), EmitError> {
    queue.borrow_mut().push(event);
    loop {
        let handler = slot.borrow().clone();
        let Some(handler) = handler else {
            return Ok(());
        };
        let event = {
            let mut events = queue.borrow_mut();
            if events.is_empty() {
                return Ok(());
            }
            events.remove(0)
        };
        handler(event)?;
    }
}
