// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use super::{TerminalError, TerminalState};
use std::{cell::RefCell, rc::Rc};

/// Never hold terminal state, queue or callback-slot borrows across user code.
pub(super) fn dispatch_events(shared: &Rc<RefCell<TerminalState>>) -> Result<(), TerminalError> {
    let (queue, handler) = {
        let s = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        (Rc::clone(&s.events), Rc::clone(&s.event_handler))
    };
    loop {
        let action = {
            let mut s = shared
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?;
            s.pending_replies
                .pop_front()
                .map(|data| (data, Rc::clone(&s.core)))
        };
        if let Some((data, core)) = action {
            core.trigger_data_event(&data, false)?;
            continue;
        }
        let callback = handler.borrow().clone();
        let Some(callback) = callback else {
            return Ok(());
        };
        let event = {
            let mut queue = queue.borrow_mut();
            if queue.is_empty() {
                return Ok(());
            }
            queue.remove(0)
        };
        callback(event).map_err(TerminalError::Event)?;
    }
}
