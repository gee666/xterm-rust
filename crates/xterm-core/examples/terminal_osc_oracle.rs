//! Actual terminal OSC differential adapter, not an interactive demo.
use std::io::{self, BufRead, Write};
#[path = "../tests/support/terminal_osc_trace.rs"]
mod trace;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runner = trace::Runner::new()?;
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    for line in io::stdin().lock().lines() {
        writeln!(out, "{}", runner.step(&line?)?)?;
    }
    Ok(())
}
