// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use super::{numbers, Demo, DemoResult};
use crate::{escape, view};

impl Demo {
    pub(super) fn check(&self, command: &str, argument: &str) -> DemoResult<()> {
        match command {
            "assert-size" | "assert-cursor" => {
                let expected = numbers(argument, 2)?;
                let actual = if command == "assert-size" {
                    (self.terminal.cols()?, self.terminal.rows()?)
                } else {
                    let active = self.terminal.buffer()?.active()?;
                    (active.cursor_x()?, active.cursor_y()?)
                };
                if actual != (expected[0], expected[1]) {
                    return Err(format!("{command}: expected {expected:?}, got {actual:?}").into());
                }
            }
            "assert-line" => {
                let (row, text) = argument
                    .split_once(' ')
                    .ok_or("assert-line needs ROW and escaped text")?;
                let row = row.parse()?;
                let actual = view::line(&self.terminal, row)?;
                let expected = escape::decode(text)?;
                if actual != expected {
                    return Err(format!(
                        "line {row}: expected UTF-16 {expected:?}, got {actual:?}"
                    )
                    .into());
                }
            }
            "assert-cell" => {
                let mut fields = argument.splitn(3, ' ');
                let col = fields
                    .next()
                    .ok_or("assert-cell needs COL ROW TEXT")?
                    .parse()?;
                let row = fields
                    .next()
                    .ok_or("assert-cell needs COL ROW TEXT")?
                    .parse()?;
                let expected =
                    escape::decode(fields.next().ok_or("assert-cell needs COL ROW TEXT")?)?;
                let actual = view::cell(&self.terminal, col, row)?.get_chars();
                if actual != expected {
                    return Err(format!(
                        "cell ({col},{row}): expected {expected:?}, got {actual:?}"
                    )
                    .into());
                }
            }
            "assert-fg" | "assert-width" => {
                let values = numbers(argument, 3)?;
                let cell = view::cell(&self.terminal, values[0], values[1])?;
                let expected = i32::try_from(values[2])?;
                let actual = if command == "assert-fg" {
                    if !cell.is_fg_palette() {
                        return Err("cell foreground is not palette-based".into());
                    }
                    cell.get_fg_color()
                } else {
                    cell.get_width()
                };
                if actual != expected {
                    return Err(format!("{command}: expected {expected}, got {actual}").into());
                }
            }
            "assert-output" => {
                let expected = escape::decode(argument)?;
                let actual = self.outbound();
                if actual != expected {
                    return Err(
                        format!("outbound UTF-16: expected {expected:?}, got {actual:?}").into(),
                    );
                }
            }
            _ => return Err(format!("unknown assertion {command:?}").into()),
        }
        Ok(())
    }
}
