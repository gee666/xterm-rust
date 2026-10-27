// Licensed under MIT. Native runner for the independent browser input oracle.
#[path = "../tests/support/input_trace.rs"]
mod protocol;
use std::error::Error;
use std::io::{self, BufRead, Write};

fn main() -> Result<(), Box<dyn Error>> {
    let mut trace = protocol::InputTrace::default();
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        writeln!(stdout, "{}", trace.apply(&line?)?)?;
    }
    Ok(())
}
