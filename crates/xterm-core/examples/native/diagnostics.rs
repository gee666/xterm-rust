// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! The example owns console output; the portable engine owns logging policy.
use super::{commands::DemoResult, escape};
use std::{
    io::{self, Write},
    rc::Rc,
};
use xterm_core::{
    headless::Terminal,
    log_service::{LogError, LogRecord, LogSink},
};

struct Console;
impl LogSink for Console {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        // Typed parameters stay debug observations. Do not invoke object coercion
        // merely to display a console argument, unlike a terminal option setter.
        writeln!(
            io::stderr().lock(),
            "[core:{:?}] {} {:?}",
            record.level,
            escape::encode(&record.message),
            record.parameters
        )
        .map_err(|error| LogError::Host(Rc::new(error)))
    }
}

pub fn install(terminal: &Terminal) -> DemoResult<()> {
    let console: Rc<dyn LogSink> = Rc::new(Console);
    let logger = terminal.with_state(|state| Rc::clone(&state.logger))?;
    logger.set_console_sink(Rc::clone(&console));
    // Construction may have diagnosed a problem before a host could attach.
    // Forward those records once instead of retaining an inaccessible buffer.
    for record in terminal.drain_logs()? {
        console.log(record)?;
    }
    Ok(())
}
