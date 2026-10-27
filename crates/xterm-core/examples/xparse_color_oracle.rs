//! Line-oriented differential-test adapter. Not an interactive terminal demo.
use std::io::{self, BufRead, Write};
use xterm_core::input::xparse_color::{parse_color, to_rgb_string};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let mut output = io::BufWriter::new(stdout.lock());
    for line in io::stdin().lock().lines() {
        let line = line?;
        if let Some(encoded) = line.strip_prefix("p ") {
            if encoded.len() % 2 != 0 || !encoded.is_ascii() {
                return Err("invalid encoded UTF-8".into());
            }
            let bytes = (0..encoded.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&encoded[i..i + 2], 16))
                .collect::<Result<Vec<_>, _>>()?;
            let spec = String::from_utf8(bytes)?;
            match parse_color(&spec) {
                Some([r, g, b]) => writeln!(output, "{r},{g},{b}")?,
                None => writeln!(output, "undefined")?,
            }
        } else if let Some(spec) = line.strip_prefix("f ") {
            let parts: Vec<_> = spec.split(',').collect();
            if parts.len() != 4 {
                return Err("format expects three channels and a depth".into());
            }
            let color = [parts[0].parse()?, parts[1].parse()?, parts[2].parse()?];
            let bits = if parts[3] == "default" {
                None
            } else {
                Some(parts[3].parse()?)
            };
            writeln!(output, "{}", to_rgb_string(color, bits))?;
        } else {
            return Err("unknown operation".into());
        }
    }
    Ok(())
}
