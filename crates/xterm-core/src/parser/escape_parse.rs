// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Escape parser synchronous action loop and fast paths.

use super::*;

pub(crate) enum ParseStep {
    Continue { next: usize, executed: bool },
    Pending(HandlerResult),
    Complete,
}

impl EscapeSequenceParser {
    /// Parse a decoded chunk. A pending handler must be awaited and resumed with
    /// this same slice and its result before any different chunk is submitted.
    pub fn parse(
        &mut self,
        data: &[u32],
        previous_result: Option<bool>,
    ) -> Result<Option<HandlerResult>, ParseError> {
        let (mut index, pending) = self.begin_steps(data, previous_result)?;
        if pending.is_some() {
            return Ok(pending);
        }
        loop {
            match self.parse_step(data, index)? {
                ParseStep::Continue { next, executed } => {
                    self.finish_step(executed);
                    self.action_boundary()?;
                    index = next;
                }
                ParseStep::Pending(pending) => return Ok(Some(pending)),
                ParseStep::Complete => return Ok(None),
            }
        }
    }

    pub(crate) fn begin_steps(
        &mut self,
        data: &[u32],
        previous: Option<bool>,
    ) -> Result<(usize, Option<HandlerResult>), ParseError> {
        self.resume(data, previous)
    }

    // Native shared ownership releases the engine before delivering an EXECUTE
    // notification. Source resets join state only after that callback returns.
    pub(crate) fn finish_step(&self, executed: bool) {
        if executed {
            self.preceding_join_state.set(0);
        }
    }

    pub(crate) fn parse_step(
        &mut self,
        data: &[u32],
        mut index: usize,
    ) -> Result<ParseStep, ParseError> {
        let Some(&code) = data.get(index) else {
            return Ok(ParseStep::Complete);
        };
        if code < 0x18 && self.current_state as u8 <= ParserState::CsiIgnore as u8 {
            self.execute_code(code, true)?;
            return Ok(ParseStep::Continue {
                next: index + 1,
                executed: true,
            });
        }
        if code == 0x1b
            && (self.current_state as u8) < ParserState::OscString as u8
            && data.get(index + 1) == Some(&0x5b)
            && index + 2 < data.len()
        {
            let (next, pending) = self.fast_csi(data, index)?;
            return Ok(match pending {
                Some(pending) => ParseStep::Pending(pending),
                None => ParseStep::Continue {
                    next,
                    executed: false,
                },
            });
        }
        // JS coerces an out-of-bounds custom table entry to IGNORE/GROUND.
        let mut transition = self
            .transitions
            .for_codepoint(code, self.current_state)
            .unwrap_or(Transition::new(ParserAction::Ignore, ParserState::Ground));
        use ParserAction::*;
        match transition.action {
            Print | OscPut | DcsPut | ApcPut => {
                let end = payload_end(data, index, transition.action);
                match transition.action {
                    Print => {
                        if let Some(print) = &mut self.print {
                            print(&data[index..end])?;
                        }
                    }
                    OscPut => self.osc.put(&data[index..end])?,
                    DcsPut => self.dcs.put(&data[index..end])?,
                    ApcPut => self.apc.put(&data[index..end])?,
                    _ => unreachable!("payload actions selected above"),
                }
                index = end - 1;
            }
            Execute => self.execute_code(code, false)?,
            Ignore => {}
            Error => {
                if let Some(error) = &mut self.error {
                    if error(ParsingState {
                        position: index,
                        code,
                        current_state: self.current_state,
                        collect: self.collect,
                        params: &self.params,
                    })? {
                        return Ok(ParseStep::Complete);
                    }
                }
            }
            CsiDispatch | EscDispatch => {
                let kind = if transition.action == CsiDispatch {
                    PendingKind::Csi
                } else {
                    PendingKind::Esc
                };
                if let Some(pending) = self.command(kind, code, index, transition)? {
                    return Ok(ParseStep::Pending(pending));
                }
            }
            Param => self.parameter(code)?,
            Collect => self.collect = self.collect.wrapping_shl(8) | code,
            Clear => self.clear_sequence()?,
            DcsHook => self
                .dcs
                .hook((self.collect.wrapping_shl(8) | code) as i32, &self.params)?,
            OscStart => self.osc.start()?,
            ApcStart => self.apc.start(self.collect.wrapping_shl(8) | code)?,
            DcsUnhook | OscEnd | ApcEnd => {
                let success = code != 0x18 && code != 0x1a;
                let (kind, pending) = match transition.action {
                    DcsUnhook => (PendingKind::Dcs, self.dcs.unhook(success, true)?),
                    OscEnd => (PendingKind::Osc, self.osc.end(success, true)?),
                    _ => (PendingKind::Apc, self.apc.end(success, true)?),
                };
                if let Some(pending) = pending {
                    self.stack = Stack::Paused(Pause {
                        kind,
                        handlers: Rc::default(),
                        next: 0,
                        transition,
                        position: index,
                    });
                    return Ok(ParseStep::Pending(pending));
                }
                self.finish_string(code, &mut transition)?;
            }
        }
        self.current_state = transition.next;
        Ok(ParseStep::Continue {
            next: index + 1,
            executed: transition.action == Execute,
        })
    }

    fn fast_csi(
        &mut self,
        data: &[u32],
        start: usize,
    ) -> Result<(usize, Option<HandlerResult>), ParseError> {
        self.clear_sequence()?;
        let mut index = start + 2;
        if (0x3c..=0x3f).contains(&data[index]) {
            self.collect = data[index];
            index += 1;
        }
        while index < data.len() {
            let code = data[index];
            if (0x30..=0x3b).contains(&code) {
                self.parameter(code)?;
            } else if (0x40..=0x7e).contains(&code) {
                let transition = Transition::new(ParserAction::CsiDispatch, ParserState::Ground);
                let pending = self.command(PendingKind::Csi, code, index, transition)?;
                if pending.is_none() {
                    self.current_state = ParserState::Ground;
                }
                return Ok((index + 1, pending));
            } else {
                break;
            }
            index += 1;
        }
        self.current_state = ParserState::CsiParam;
        Ok((index, None))
    }
}

fn payload_end(data: &[u32], start: usize, action: ParserAction) -> usize {
    let mut end = start + 1;
    while end < data.len() {
        let code = data[end];
        let allowed = match action {
            ParserAction::Print => (0x20..=0x7e).contains(&code) || code >= 0xa0,
            ParserAction::OscPut => code >= 0x20 && !(0x80..0xa0).contains(&code),
            // Source read-ahead includes DEL after an initial DCS_PUT.
            ParserAction::DcsPut => {
                ![0x18, 0x1a, 0x1b].contains(&code) && !(0x80..0xa0).contains(&code)
            }
            ParserAction::ApcPut => {
                (0x20..0x7f).contains(&code) || (0x08..0x0e).contains(&code) || code >= 0xa0
            }
            _ => unreachable!("only called for payload actions"),
        };
        if !allowed {
            break;
        }
        end += 1;
    }
    end
}
