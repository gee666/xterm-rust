// Licensed under MIT. A stdin/stdout adapter for the independent JS oracle.
#[path = "../tests/support/kitty.rs"]
mod protocol;

use std::error::Error;
use std::io::{self, BufRead, Write};

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        writeln!(stdout, "{}", protocol::evaluate_row(&line?)?)?;
    }
    Ok(())
}
