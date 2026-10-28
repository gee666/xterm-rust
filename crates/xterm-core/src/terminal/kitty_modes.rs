// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Kitty keyboard negotiation is terminal state, separate from key encoding.
use super::{TerminalError, TerminalState};
use crate::parser::params::Params;
use std::rc::Rc;

const KITTY_STACK_LIMIT: usize = 16;

impl TerminalState {
    pub(super) fn kitty_mode(
        &mut self,
        operation: &str,
        params: &Params,
    ) -> Result<(), TerminalError> {
        let first = params.values().first().copied().unwrap_or(0);
        match operation {
            "=" => {
                let mode = params
                    .values()
                    .get(1)
                    .copied()
                    .filter(|value| *value != 0)
                    .unwrap_or(1);
                // CSI parameters are nonnegative int32 values. The stored u32
                // retains the exact flag bits used by the keyboard encoder.
                let flags = first as u32;
                let mut state = self.core.kitty_keyboard.borrow_mut();
                match mode {
                    1 => state.flags = flags,
                    2 => state.flags |= flags,
                    3 => state.flags &= !flags,
                    _ => {}
                }
            }
            "?" => {
                let flags = self.core.kitty_keyboard.borrow().flags;
                self.reply(&format!("\x1b[?{flags}u"))?;
            }
            ">" => {
                let is_alt = Rc::ptr_eq(&self.buffers.active(), &self.buffers.alt());
                let mut state = self.core.kitty_keyboard.borrow_mut();
                let state = &mut *state;
                let stack = if is_alt {
                    &mut state.alt_stack
                } else {
                    &mut state.main_stack
                };
                if stack.len() >= KITTY_STACK_LIMIT {
                    stack.remove(0);
                }
                stack.push(state.flags);
                state.flags = first as u32;
            }
            "<" => {
                let is_alt = Rc::ptr_eq(&self.buffers.active(), &self.buffers.alt());
                let mut state = self.core.kitty_keyboard.borrow_mut();
                let state = &mut *state;
                let stack = if is_alt {
                    &mut state.alt_stack
                } else {
                    &mut state.main_stack
                };
                // At most sixteen iterations, regardless of hostile CSI counts.
                let count = first.max(1) as usize;
                for _ in 0..count.min(stack.len()) {
                    // min bounds each iteration to an existing stack entry.
                    state.flags = stack.pop().expect("bounded Kitty stack pop");
                }
                if stack.is_empty() {
                    state.flags = 0;
                }
            }
            _ => unreachable!("dispatch_mode only forwards Kitty operations"),
        }
        Ok(())
    }
}
