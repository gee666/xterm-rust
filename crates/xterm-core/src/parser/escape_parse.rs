// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Escape parser synchronous action loop and fast paths.

use super::*;

impl EscapeSequenceParser {
    /// Parse a decoded chunk. A pending handler must be awaited and resumed with
    /// this same slice and its result before any different chunk is submitted.
    pub fn parse(
        &mut self,
        data: &[u32],
        previous_result: Option<bool>,
    ) -> Result<Option<HandlerResult>, ParseError> {
        let (mut index, pending) = self.resume(data, previous_result)?;
        if pending.is_some() {
            return Ok(pending);
        }
        while index < data.len() {
            let code = data[index];
            // These fast paths are upstream behavior, even for custom tables.
            if code < 0x18 && self.current_state as u8 <= ParserState::CsiIgnore as u8 {
                self.execute_code(code, true)?;
                self.action_boundary()?;
                index += 1;
                continue;
            }
            if code == 0x1b
                && (self.current_state as u8) < ParserState::OscString as u8
                && data.get(index + 1) == Some(&0x5b)
                && index + 2 < data.len()
            {
                let (next, pending) = self.fast_csi(data, index)?;
                if pending.is_some() {
                    return Ok(pending);
                }
                self.action_boundary()?;
                index = next;
                continue;
            }
            // A short custom Uint16Array returns undefined outside its bounds.
            // Upstream bitwise access coerces that value to IGNORE/GROUND.
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
                        }) {
                            return Ok(None);
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
                        return Ok(Some(pending));
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
                        return Ok(Some(pending));
                    }
                    self.finish_string(code, &mut transition)?;
                }
            }
            self.current_state = transition.next;
            self.action_boundary()?;
            index += 1;
        }
        Ok(None)
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
            // The source read-ahead includes DEL after an initial DCS_PUT even
            // though DEL by itself maps to IGNORE. Preserve that distinction.
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
