// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{TerminalError, TerminalEvent, TerminalState, WindowReport};
use crate::{options::OptionsService, parser::params::Params};
use std::{cell::RefCell, rc::Rc};
const TITLE_STACK_LIMIT: usize = 10;

/// Permission gate for built-ins and custom CSI t handlers. Call it without a
/// terminal-state borrow because host property getters may synchronously reset.
pub fn window_option_allowed(
    options: &OptionsService,
    params: &Params,
) -> Result<bool, TerminalError> {
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
    Ok(options.get("windowOptions")?.get_property(key)?.is_truthy())
}
impl TerminalState {
    pub fn window_option_allowed(&self, params: &Params) -> Result<bool, TerminalError> {
        window_option_allowed(&self.options, params)
    }
}
pub(super) fn window_options(
    shared: &Rc<RefCell<TerminalState>>,
    params: &Params,
) -> Result<(), TerminalError> {
    let options = Rc::clone(
        &shared
            .try_borrow()
            .map_err(|_| TerminalError::Borrowed)?
            .options,
    );
    if !window_option_allowed(&options, params)? {
        return Ok(());
    }
    let first = params.values().first().copied().unwrap_or(0);
    let second = params.values().get(1).copied().unwrap_or(0);
    if first == 23 {
        if second == 0 || second == 2 {
            let title = shared
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?
                .title_stack
                .pop();
            if let Some(title) = title {
                shared
                    .try_borrow_mut()
                    .map_err(|_| TerminalError::Borrowed)?
                    .set_title(title)?;
                // Title observers see the old icon, even when both stacks pop.
                super::dispatch_events(shared)?;
            }
        }
        if second == 0 || second == 1 {
            let mut state = shared
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?;
            if let Some(icon) = state.icon_stack.pop() {
                state.set_icon_name(icon)?;
            }
        }
        return Ok(());
    }
    let mut state = shared
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?;
    match first {
        14 if second != 2 => state
            .events
            .borrow_mut()
            .push(TerminalEvent::WindowReport(WindowReport::TextAreaPixels)),
        16 => state
            .events
            .borrow_mut()
            .push(TerminalEvent::WindowReport(WindowReport::CellPixels)),
        18 => {
            let response = format!("\x1b[8;{};{}t", state.rows(), state.cols());
            state.reply(&response)?;
        }
        22 => {
            if second == 0 || second == 2 {
                let title = state.title.clone();
                state.title_stack.push(title);
                if state.title_stack.len() > TITLE_STACK_LIMIT {
                    state.title_stack.remove(0);
                }
            }
            if second == 0 || second == 1 {
                let icon = state.icon_name.clone();
                state.icon_stack.push(icon);
                if state.icon_stack.len() > TITLE_STACK_LIMIT {
                    state.icon_stack.remove(0);
                }
            }
        }
        _ => {}
    }
    Ok(())
}
