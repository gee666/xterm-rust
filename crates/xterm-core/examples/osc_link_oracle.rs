//! Differential-test adapter for the real buffer-backed OSC link service.
use std::io::{self, BufRead, Write};
#[path = "../tests/support/osc_link_trace.rs"]
mod trace;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runner = trace::Runner::new()?;
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    for operation in io::stdin().lock().lines() {
        writeln!(out, "{}", runner.step(&operation?)?)?;
    }
    Ok(())
}
