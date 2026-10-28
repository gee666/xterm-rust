// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
// Validation recorder, not the interactive demonstration required by the port.
#[path = "../tests/support/terminal_snapshot.rs"]
mod terminal_snapshot;
#[path = "../tests/support/input_handler_core_trace.rs"]
mod trace;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for (_, actual) in trace::replay()? {
        println!("{}", actual.join("\t"));
    }
    Ok(())
}
