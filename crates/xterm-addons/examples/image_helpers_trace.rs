//! Replay source-generated image header and metadata operations.
#[path = "../tests/image_helper_support/mod.rs"]
mod support;
use std::error::Error;
use std::io::{self, BufRead};
use xterm_addons::image::header::HeaderParser;
fn main() -> Result<(), Box<dyn Error>> {
    let mut parser = HeaderParser::default();
    for line in io::stdin().lock().lines() {
        let operation = serde_json::from_str(&line?)?;
        println!("{}", support::apply(&mut parser, &operation)?);
    }
    Ok(())
}
