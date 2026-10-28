// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! A small native viewport inspector, not the browser renderer.
use super::{
    commands::{Demo, DemoResult},
    escape,
};
use std::fmt::Write;
use xterm_core::{
    buffer::{AttributeData, CellData},
    headless::Terminal,
    serialize::{self, CellRange, Position},
};

const CSI: &str = "\x1b[";
const RESET_SGR: &str = "\x1b[0m";
const BOLD: u8 = 1;
const DIM: u8 = 2;
const ITALIC: u8 = 3;
const UNDERLINE: u8 = 4;
const BLINK: u8 = 5;
const INVERSE: u8 = 7;
const INVISIBLE: u8 = 8;
const STRIKETHROUGH: u8 = 9;
const OVERLINE: u8 = 53;
const FOREGROUND: u8 = 38;
const BACKGROUND: u8 = 48;
const PALETTE_COLOR: u8 = 5;
const RGB_COLOR: u8 = 2;

pub fn cell(terminal: &Terminal, col: usize, row: usize) -> DemoResult<CellData> {
    if col >= terminal.cols()? || row >= terminal.rows()? {
        return Err("cell is outside the visible viewport".into());
    }
    let active = terminal.buffer()?.active()?;
    let index = active
        .viewport_y()?
        .checked_add(row)
        .ok_or("line index overflow")?;
    let line = active.get_line(index)?.ok_or("visible line is missing")?;
    let line = line.try_borrow()?;
    Ok(line.get_cell(col).ok_or("visible cell is missing")?)
}

pub fn line(terminal: &Terminal, row: usize) -> DemoResult<Vec<u16>> {
    if row >= terminal.rows()? {
        return Err("line is outside the visible viewport".into());
    }
    let active = terminal.buffer()?.active()?;
    let index = active
        .viewport_y()?
        .checked_add(row)
        .ok_or("line index overflow")?;
    let line = active.get_line(index)?.ok_or("visible line is missing")?;
    let line = line.try_borrow()?;
    Ok(line.translate_to_string(true, None, None, None))
}

pub fn frame(demo: &Demo, plain: bool) -> DemoResult<String> {
    let terminal = &demo.terminal;
    let active = terminal.buffer()?.active()?;
    let mut output = format!(
        "{}x{} cursor=({},{}) base={} viewport={}\n",
        terminal.cols()?,
        terminal.rows()?,
        active.cursor_x()?,
        active.cursor_y()?,
        active.base_y()?,
        active.viewport_y()?
    );
    for row in 0..terminal.rows()? {
        output.push('|');
        if plain {
            // Native UTF-8 display uses replacement for lone surrogates; all
            // assertions and serialized/output observations remain lossless UTF-16.
            output.push_str(&String::from_utf16_lossy(&line(terminal, row)?));
        } else {
            for col in 0..terminal.cols()? {
                let cell = cell(terminal, col, row)?;
                if cell.get_width() == 0 {
                    continue;
                }
                style(&mut output, &cell)?;
                let chars = cell.get_chars();
                if chars.is_empty() {
                    output.push(' ');
                } else {
                    output.push_str(&String::from_utf16_lossy(&chars));
                }
            }
            output.push_str(RESET_SGR);
        }
        output.push_str("|\n");
    }
    writeln!(
        output,
        "outbound UTF-16: {}",
        escape::encode(&demo.outbound())
    )?;
    Ok(output)
}

fn style(output: &mut String, cell: &CellData) -> std::fmt::Result {
    output.push_str(RESET_SGR);
    for (flag, code) in [
        (cell.is_bold(), BOLD),
        (cell.is_dim(), DIM),
        (cell.is_italic(), ITALIC),
        (cell.is_underline(), UNDERLINE),
        (cell.is_blink(), BLINK),
        (cell.is_inverse(), INVERSE),
        (cell.is_invisible(), INVISIBLE),
        (cell.is_strikethrough(), STRIKETHROUGH),
        (cell.is_overline(), OVERLINE),
    ] {
        if flag != 0 {
            write!(output, "{CSI}{code}m")?;
        }
    }
    for (is_palette, is_rgb, color, selector) in [
        (
            cell.is_fg_palette(),
            cell.is_fg_rgb(),
            cell.get_fg_color(),
            FOREGROUND,
        ),
        (
            cell.is_bg_palette(),
            cell.is_bg_rgb(),
            cell.get_bg_color(),
            BACKGROUND,
        ),
    ] {
        if is_palette {
            write!(output, "{CSI}{selector};{PALETTE_COLOR};{color}m")?;
        } else if is_rgb {
            let [red, green, blue] =
                AttributeData::to_color_rgb(u32::try_from(color).map_err(|_| std::fmt::Error)?);
            write!(output, "{CSI}{selector};{RGB_COLOR};{red};{green};{blue}m")?;
        }
    }
    Ok(())
}

pub fn serialized(terminal: &Terminal) -> DemoResult<String> {
    let (buffer, attributes) =
        terminal.with_state(|state| (state.active_buffer(), state.attributes.clone()))?;
    let buffer = buffer.try_borrow()?;
    let cols = terminal.cols()?;
    let rows = terminal.rows()?;
    let start = buffer.ydisp;
    let end = start.checked_add(rows - 1).ok_or("line index overflow")?;
    let serialized = serialize::serialize_buffer(
        &*buffer,
        cols,
        rows,
        &attributes,
        CellRange {
            start: Position { x: 0, y: start },
            end: Position { x: cols, y: end },
        },
        false,
    )?;
    Ok(format!(
        "serialized UTF-16: {}",
        escape::encode(&serialized)
    ))
}
