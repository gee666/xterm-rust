// Licensed under MIT. Ordered browser keyboard dispatch trace runner.
#[path = "../tests/support/keyboard_dispatch.rs"]
mod support;

use std::error::Error;
use std::io::{self, BufRead, Write};
use xterm_web::input::keyboard_dispatch::KeyboardDispatch;

fn main() -> Result<(), Box<dyn Error>> {
    let dispatch = KeyboardDispatch::default();
    let host = support::Host::default();
    let mut out = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        writeln!(out, "{}", support::row(&dispatch, &host, &line?)?)?;
    }
    Ok(())
}
