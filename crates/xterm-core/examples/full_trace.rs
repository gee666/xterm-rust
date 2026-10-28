//! Native half of the acceptance traces. The protocol carries raw UTF-16 units.
use std::{error::Error, fs, path::PathBuf};
use xterm_core::{options::OptionValue, terminal::CoreTerminal};
#[path = "../tests/support/terminal_snapshot.rs"]
mod base;
#[path = "../tests/support/full_trace_snapshot.rs"]
mod rich;

fn units(text: &str) -> Result<Vec<u16>, Box<dyn Error>> {
    if !text.is_empty() {
        for unit in text.split(',') {
            u16::from_str_radix(unit, 16)?;
        }
    }
    // Validation above establishes the existing fixture helper's parse invariant.
    Ok(base::parse_hex(text))
}
fn boolean(text: &str) -> Result<bool, Box<dyn Error>> {
    match text {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err("invalid boolean encoding".into()),
    }
}
fn value(kind: &str, text: &str) -> Result<OptionValue, Box<dyn Error>> {
    Ok(match kind {
        "b" => OptionValue::Bool(boolean(text)?),
        "n" => OptionValue::Number(text.parse()?),
        "s" => OptionValue::String(units(text)?),
        "m" => OptionValue::property_map(
            text.split(',')
                .filter(|item| !item.is_empty())
                .map(|item| {
                    let (key, flag) = item.split_once('=').ok_or("invalid option map")?;
                    Ok((key, OptionValue::Bool(boolean(flag)?)))
                })
                .collect::<Result<Vec<_>, Box<dyn Error>>>()?,
        ),
        _ => return Err("unknown option value kind".into()),
    })
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("usage: full_trace INPUT.tsv OUTPUT.tsv".into());
    }
    let input = fs::read_to_string(PathBuf::from(&args[0]))?;
    let mut terminal: Option<CoreTerminal> = None;
    let mut output = String::new();
    for (index, line) in input.lines().enumerate() {
        let f = line.split('\t').collect::<Vec<_>>();
        if f.first() == Some(&"N") {
            if let Some(mut previous) = terminal.take() {
                previous.dispose()?;
            }
            if f.len() != 4 {
                return Err(format!("invalid constructor line {}", index + 1).into());
            }
            let mut created = CoreTerminal::new(f[1].parse()?, f[2].parse()?, f[3].parse()?)?;
            // Reference listeners attach after construction; discard earlier events.
            base::events(&mut created);
            terminal = Some(created);
            continue;
        }
        let t = terminal.as_mut().ok_or("operation before construction")?;
        match (f[0], f.len()) {
            ("W", 2) => t.write_utf16(&units(f[1])?)?,
            ("B", 2) => {
                let bytes = units(f[1])?
                    .into_iter()
                    .map(u8::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                t.write_bytes(&bytes)?;
            }
            ("R", 3) => t.resize(f[1].parse()?, f[2].parse()?)?,
            ("S", 2) => t.scroll_lines(f[1].parse()?)?,
            ("C", 1) => t.clear()?,
            ("Q", 1) => t.reset()?,
            ("O", 4) => {
                let options = std::rc::Rc::clone(&t.state().options);
                options.set(f[1], value(f[2], f[3])?)?;
            }
            _ => return Err(format!("invalid trace line {}", index + 1).into()),
        }
        output.push_str(&rich::snapshot(t)?);
        output.push('\t');
        output.push_str(&rich::events(t)?);
        output.push('\n');
    }
    if let Some(mut terminal) = terminal {
        terminal.dispose()?;
    }
    fs::write(PathBuf::from(&args[1]), output)?;
    Ok(())
}
