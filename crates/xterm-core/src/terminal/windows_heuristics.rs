// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use crate::options::{OptionError, OptionValue};
/// Cache policy changes on option assignment, as CoreTerminal's disposable hooks do.
pub(super) fn enabled(pty: &OptionValue) -> Result<bool, OptionError> {
    if !pty.is_truthy() {
        return Ok(false);
    }
    if matches!(pty.get_property("backend")?, OptionValue::Undefined)
        || matches!(pty.get_property("buildNumber")?, OptionValue::Undefined)
    {
        return Ok(false);
    }
    // Keep the source's second reads: host getters can change values in between.
    Ok(pty.get_property("backend")? == OptionValue::from("conpty")
        && pty.get_property("buildNumber")?.number()? < 21376.0)
}
