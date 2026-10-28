// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{TerminalError, TerminalEvent, TerminalState, WindowReport};
use crate::parser::params::Params;
const TITLE_STACK_LIMIT: usize = 10;

impl TerminalState {
    /// Security gate also used before public custom CSI t callbacks.
    pub fn window_option_allowed(&self, params: &Params) -> Result<bool, TerminalError> {
        let operation = params.values().first().copied().unwrap_or(0);
        let key = match operation {
            1 => "restoreWin",
            2 => "minimizeWin",
            3 => "setWinPosition",
            4 => "setWinSizePixels",
            5 => "raiseWin",
            6 => "lowerWin",
            7 => "refreshWin",
            8 => "setWinSizeChars",
            9 => "maximizeWin",
            10 => "fullscreenWin",
            11 => "getWinState",
            13 => "getWinPosition",
            14 => "getWinSizePixels",
            15 => "getScreenSizePixels",
            16 => "getCellSizePixels",
            18 => "getWinSizeChars",
            19 => "getScreenSizeChars",
            20 => "getIconTitle",
            21 => "getWinTitle",
            22 => "pushTitle",
            23 => "popTitle",
            24.. => "setWinLines",
            _ => return Ok(false),
        };
        Ok(self
            .options
            .get("windowOptions")?
            .get_property(key)?
            .is_truthy())
    }
    pub(crate) fn window_options(&mut self, params: &Params) -> Result<(), TerminalError> {
        if !self.window_option_allowed(params)? {
            return Ok(());
        }
        let first = params.values().first().copied().unwrap_or(0);
        let second = params.values().get(1).copied().unwrap_or(0);
        match first {
            14 if second != 2 => self
                .events
                .borrow_mut()
                .push(TerminalEvent::WindowReport(WindowReport::TextAreaPixels)),
            16 => self
                .events
                .borrow_mut()
                .push(TerminalEvent::WindowReport(WindowReport::CellPixels)),
            18 => self.reply(&format!("\x1b[8;{};{}t", self.rows(), self.cols()))?,
            22 => {
                if second == 0 || second == 2 {
                    self.title_stack.push(self.title.clone());
                    if self.title_stack.len() > TITLE_STACK_LIMIT {
                        self.title_stack.remove(0);
                    }
                }
                if second == 0 || second == 1 {
                    self.icon_stack.push(self.icon_name.clone());
                    if self.icon_stack.len() > TITLE_STACK_LIMIT {
                        self.icon_stack.remove(0);
                    }
                }
            }
            23 => {
                if second == 0 || second == 2 {
                    if let Some(title) = self.title_stack.pop() {
                        self.set_title(title)?;
                    }
                }
                if second == 0 || second == 1 {
                    if let Some(icon) = self.icon_stack.pop() {
                        self.set_icon_name(icon)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
