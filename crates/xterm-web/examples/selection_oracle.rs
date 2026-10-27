// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! Line-oriented native driver for scripts/selection-model-oracle.mjs.
use std::{
    error::Error,
    io::{self, BufRead, Write},
    num::NonZeroU32,
};
use xterm_web::selection::model::{Dimensions, Position, SelectionModel};

fn position(column: &str, row: &str) -> Result<Option<Position>, Box<dyn Error>> {
    if column == "-" && row == "-" {
        Ok(None)
    } else {
        Ok(Some(Position::new(column.parse()?, row.parse()?)))
    }
}

fn encoded(point: Option<Position>) -> String {
    point.map_or_else(
        || "-,-".into(),
        |point| format!("{},{}", point.column, point.row),
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for line in stdin.lock().lines() {
        let line = line?;
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 11 {
            return Err(format!("expected 11 fields, received {}", fields.len()).into());
        }
        let dimensions = Dimensions {
            columns: NonZeroU32::new(fields[0].parse()?).ok_or("zero columns")?,
            rows: NonZeroU32::new(fields[1].parse()?).ok_or("zero rows")?,
            scrollback: fields[2].parse()?,
        };
        let mut model = SelectionModel {
            selection_start: position(fields[3], fields[4])?,
            selection_end: position(fields[5], fields[6])?,
            selection_start_length: fields[7].parse()?,
            is_select_all_active: fields[8] == "1",
        };
        if fields[8] != "0" && fields[8] != "1" {
            return Err("invalid select-all flag".into());
        }
        let refresh = if fields[9] == "-" {
            None
        } else {
            Some(model.handle_trim(fields[9].parse()?)?)
        };
        match fields[10] {
            "0" => {}
            "1" => model.clear_selection(),
            _ => return Err("invalid clear flag".into()),
        }
        writeln!(
            stdout,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            encoded(model.final_selection_start()),
            encoded(model.final_selection_end(dimensions)?),
            u8::from(model.are_selection_values_reversed()),
            refresh.map_or_else(|| "-".into(), |value| u8::from(value).to_string()),
            encoded(model.selection_start),
            encoded(model.selection_end),
            model.selection_start_length,
            u8::from(model.is_select_all_active)
        )?;
    }
    stdout.flush()?;
    Ok(())
}
