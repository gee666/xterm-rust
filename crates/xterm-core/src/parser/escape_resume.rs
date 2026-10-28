// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Escape parser asynchronous dispatch and continuation.

use super::*;

impl EscapeSequenceParser {
    /// The boolean indicates exhausted handlers, for synchronous fallbacks only.
    fn dispatch(
        &mut self,
        mut pause: Pause,
        previous: Option<bool>,
    ) -> Result<(bool, Option<HandlerResult>), ParseError> {
        if previous == Some(true) {
            return Ok((false, None));
        }
        while pause.next > 0 {
            pause.next -= 1;
            // Clone before calling. Tokens can mutate this very list in callbacks.
            let handler = pause
                .handlers
                .borrow()
                .get(pause.next)
                .cloned()
                .ok_or(DispatchError::ActiveHandlerRemoved)?;
            let result = handler.borrow_mut()(&self.params);
            match result {
                HandlerResult::Ready(true) => return Ok((false, None)),
                HandlerResult::Ready(false) => {}
                HandlerResult::Failed(error) => return Err(error.into()),
                pending @ (HandlerResult::Pending(_) | HandlerResult::PendingFallible(_)) => {
                    self.stack = Stack::Paused(pause);
                    return Ok((false, Some(pending)));
                }
            }
        }
        Ok((true, None))
    }

    pub(super) fn command(
        &mut self,
        kind: PendingKind,
        code: u32,
        position: usize,
        transition: Transition,
    ) -> Result<Option<HandlerResult>, ParseError> {
        let id = self.collect.wrapping_shl(8) | code;
        let handlers = match kind {
            PendingKind::Csi => self.csi.get(&id),
            _ => self.esc.get(&id),
        };
        let next = handlers.borrow().len();
        let (exhausted, pending) = self.dispatch(
            Pause {
                kind,
                handlers,
                next,
                transition,
                position,
            },
            None,
        )?;
        if pending.is_some() {
            return Ok(pending);
        }
        if exhausted {
            match kind {
                PendingKind::Csi => {
                    if let Some(fallback) = &mut self.csi_fallback {
                        fallback(id, &self.params);
                    }
                }
                _ => {
                    if let Some(fallback) = &mut self.esc_fallback {
                        fallback(id);
                    }
                }
            }
        }
        self.preceding_join_state.set(0);
        Ok(None)
    }

    pub(super) fn finish_string(
        &mut self,
        code: u32,
        transition: &mut Transition,
    ) -> Result<(), ParseError> {
        if code == 0x1b {
            transition.next =
                ParserState::from_code((transition.next as u8) | ParserState::Escape as u8)
                    .ok_or(ParseError::MissingTransition)?;
        }
        self.clear_sequence()?;
        self.preceding_join_state.set(0);
        Ok(())
    }

    pub(super) fn resume(
        &mut self,
        data: &[u32],
        previous: Option<bool>,
    ) -> Result<(usize, Option<HandlerResult>), ParseError> {
        let pause = match std::mem::replace(&mut self.stack, Stack::None) {
            Stack::None => return Ok((0, None)),
            Stack::Reset(position) => return Ok((position + 1, None)),
            Stack::Failed(pause) => {
                self.stack = Stack::Failed(pause);
                return Err(ParseError::ImproperContinuation);
            }
            Stack::Paused(pause) => pause,
        };
        let Some(previous) = previous else {
            self.stack = Stack::Failed(pause);
            return Err(ParseError::ImproperContinuation);
        };
        let position = pause.position;
        let mut transition = pause.transition;
        let saved = pause.clone();
        let resumed = (|| -> Result<Option<HandlerResult>, ParseError> {
            Ok(match pause.kind {
                // Upstream does not invoke fallback on asynchronous exhaustion.
                PendingKind::Csi | PendingKind::Esc => self.dispatch(pause, Some(previous))?.1,
                kind => {
                    let code = *data.get(position).ok_or(ParseError::ImproperContinuation)?;
                    let success = code != 0x18 && code != 0x1a;
                    let result = match kind {
                        PendingKind::Dcs => self.dcs.unhook(success, previous)?,
                        PendingKind::Osc => self.osc.end(success, previous)?,
                        PendingKind::Apc => self.apc.end(success, previous)?,
                        _ => unreachable!("command variants handled above"),
                    };
                    if result.is_some() {
                        self.stack = Stack::Paused(pause);
                    } else {
                        self.finish_string(code, &mut transition)?;
                    }
                    result
                }
            })
        })();
        let pending = match resumed {
            Ok(pending) => pending,
            Err(error) => {
                // A thrown callback does not clear the upstream saved frame.
                // Keep it for diagnosis or explicit reset, not silent replay.
                self.stack = Stack::Paused(saved);
                return Err(error);
            }
        };
        if pending.is_some() {
            return Ok((position, pending));
        }
        self.preceding_join_state.set(0);
        self.current_state = transition.next;
        self.action_boundary()?;
        Ok((position + 1, None))
    }
}
