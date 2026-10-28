// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use crate::options::{OptionError, OptionValue};
/// Cache policy changes on option assignment, as CoreTerminal's disposable hooks do.
pub(super) fn enabled(pty: &OptionValue) -> Result<bool, OptionError> {
    if !pty.is_truthy() {
        return Ok(false);
    }
    let backend = pty.get_property("backend")?;
    let build = pty.get_property("buildNumber")?;
    if matches!(backend, OptionValue::Undefined) || matches!(build, OptionValue::Undefined) {
        return Ok(false);
    }
    Ok(backend == OptionValue::from("conpty") && build.number()? < 21376.0)
}
