// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-image/src/kitty/KittyGraphicsTypes.ts.

//! Lossless control fields. Invalid numeric text is present NaN, not absence;
//! string fields retain UTF-16, including lone surrogates and empty values.

use crate::image::numeric::parse_decimal_prefix;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct KittyCommand {
    pub action: Option<Vec<u16>>,
    pub format: Option<f64>,
    pub id: Option<f64>,
    pub image_number: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub source_width: Option<f64>,
    pub source_height: Option<f64>,
    pub x_offset: Option<f64>,
    pub y_offset: Option<f64>,
    pub columns: Option<f64>,
    pub rows: Option<f64>,
    pub more: Option<f64>,
    pub quiet: Option<f64>,
    pub cursor_movement: Option<f64>,
    pub z_index: Option<f64>,
    pub transmission: Option<Vec<u16>>,
    pub delete_selector: Option<Vec<u16>>,
    pub placement_id: Option<f64>,
    pub compression: Option<Vec<u16>>,
}

/// Parse comma-separated controls, splitting each segment at its first equals.
/// Unknown keys and segments without equals do not mutate the command. Duplicate
/// recognized keys replace their previous value, including empty strings/NaN.
pub fn parse_command(data: &[u16]) -> KittyCommand {
    let mut command = KittyCommand::default();
    for part in data.split(|unit| *unit == u16::from(b',')) {
        let Some(equals) = part.iter().position(|unit| *unit == u16::from(b'=')) else {
            continue;
        };
        if equals != 1 {
            continue;
        }
        let value = &part[equals + 1..];
        match u8::try_from(part[0]) {
            Ok(b'a') => command.action = Some(value.to_vec()),
            Ok(b'o') => command.compression = Some(value.to_vec()),
            Ok(b't') => command.transmission = Some(value.to_vec()),
            Ok(b'd') => command.delete_selector = Some(value.to_vec()),
            Ok(b'f') => command.format = Some(parse_decimal_prefix(value)),
            Ok(b'i') => command.id = Some(parse_decimal_prefix(value)),
            Ok(b'I') => command.image_number = Some(parse_decimal_prefix(value)),
            Ok(b's') => command.width = Some(parse_decimal_prefix(value)),
            Ok(b'v') => command.height = Some(parse_decimal_prefix(value)),
            Ok(b'x') => command.x = Some(parse_decimal_prefix(value)),
            Ok(b'y') => command.y = Some(parse_decimal_prefix(value)),
            Ok(b'w') => command.source_width = Some(parse_decimal_prefix(value)),
            Ok(b'h') => command.source_height = Some(parse_decimal_prefix(value)),
            Ok(b'X') => command.x_offset = Some(parse_decimal_prefix(value)),
            Ok(b'Y') => command.y_offset = Some(parse_decimal_prefix(value)),
            Ok(b'c') => command.columns = Some(parse_decimal_prefix(value)),
            Ok(b'r') => command.rows = Some(parse_decimal_prefix(value)),
            Ok(b'm') => command.more = Some(parse_decimal_prefix(value)),
            Ok(b'q') => command.quiet = Some(parse_decimal_prefix(value)),
            Ok(b'C') => command.cursor_movement = Some(parse_decimal_prefix(value)),
            Ok(b'z') => command.z_index = Some(parse_decimal_prefix(value)),
            Ok(b'p') => command.placement_id = Some(parse_decimal_prefix(value)),
            _ => {}
        }
    }
    command
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;
