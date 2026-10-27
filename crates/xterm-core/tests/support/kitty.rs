// Licensed under MIT. Shared TSV protocol for tests and the differential runner.
use std::error::Error;
use xterm_core::input::keyboard::KeyboardEvent;
use xterm_core::input::kitty_keyboard::{evaluate, EventType};

fn units(text: &str) -> Result<Vec<u16>, std::num::ParseIntError> {
    text.split(',')
        .filter(|unit| !unit.is_empty())
        .map(|unit| u16::from_str_radix(unit, 16))
        .collect()
}

pub fn evaluate_row(line: &str) -> Result<String, Box<dyn Error>> {
    let fields: Vec<_> = line.split('\t').collect();
    if fields.len() < 7 {
        return Err("expected seven tab-separated input fields".into());
    }
    let key = units(fields[1])?;
    let code = units(fields[2])?;
    let modifiers: u8 = fields[3].parse()?;
    let event_type = match fields[5] {
        "1" => EventType::Press,
        "2" => EventType::Repeat,
        "3" => EventType::Release,
        _ => return Err("event type must be 1, 2 or 3".into()),
    };
    let event = KeyboardEvent {
        key_code: fields[0].parse()?,
        key: &key,
        code: &code,
        shift_key: modifiers & 1 != 0,
        alt_key: modifiers & 2 != 0,
        ctrl_key: modifiers & 4 != 0,
        meta_key: modifiers & 8 != 0,
    };
    let result = evaluate(&event, fields[4].parse()?, event_type, fields[6] == "1");
    let key = result
        .key
        .map(|units| {
            units
                .iter()
                .map(|u| format!("{u:x}"))
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_else(|| "-".into());
    Ok(format!(
        "{}\t{}\t{key}",
        result.result_type as u8,
        u8::from(result.cancel)
    ))
}
