// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{
    append, scrollback_start, style, BufferAccess, CellRange, HtmlConfig, HtmlOptions, Position,
    SerializeError,
};
use crate::buffer::{constants::underline_style, CellData};

/// Serialize the active buffer with the upstream HTML fragment format.
///
/// `selection` is used only when `only_selection` is set and no explicit range
/// exists. An absent selection then yields an empty string. UTF-16 cells are
/// retained verbatim except for upstream's whole-cell `&` and `<` escaping.
/// Theme strings and font family are not HTML-sanitized by the upstream contract.
pub fn serialize_html(
    buffer: &dyn BufferAccess,
    cols: usize,
    rows: usize,
    selection: Option<CellRange>,
    config: &HtmlConfig,
    options: &HtmlOptions,
) -> Result<Vec<u16>, SerializeError> {
    let range = if let Some(range) = options.range {
        Some(CellRange {
            start: Position {
                x: range.start_col,
                y: range.start_line,
            },
            end: Position {
                x: cols,
                y: range.end_line,
            },
        })
    } else if options.only_selection {
        let Some(selection) = selection else {
            return Ok(Vec::new());
        };
        Some(selection)
    } else {
        let start = scrollback_start(buffer.len(), rows, options.scrollback);
        (start < buffer.len()).then(|| CellRange {
            start: Position { x: 0, y: start },
            end: Position {
                x: cols,
                y: buffer.len() - 1,
            },
        })
    };
    let defaults;
    let palette = match &config.ansi_colors {
        Some(colors) => colors,
        None => {
            defaults = crate::color::palette::default_ansi_colors().map(|color| color.css);
            &defaults
        }
    };
    let mut output = header(config, options.include_global_background);
    if let Some(range) = range {
        let mut previous = CellData::default();
        for row in range.start.y..=range.end.y {
            append(&mut output, "<div><span>");
            if let Some(length) = buffer.line_len(row)? {
                let start = if row == range.start.y {
                    range.start.x
                } else {
                    0
                };
                let end = if row == range.end.y {
                    range.end.x
                } else {
                    length
                };
                for col in start..end {
                    if let Some(cell) = buffer.cell(row, col)? {
                        emit_cell(&mut output, &cell, &previous, palette);
                        // BaseSerializeHandler advances oldCell even for placeholders.
                        previous = cell;
                    }
                }
            }
            append(&mut output, "</span></div>");
        }
    }
    append(&mut output, "</div></pre><!--EndFragment--></body></html>");
    Ok(output)
}

fn header(config: &HtmlConfig, include_background: bool) -> Vec<u16> {
    let (fg, bg) = if include_background {
        (
            config.foreground.as_deref().unwrap_or("#ffffff"),
            config.background.as_deref().unwrap_or("#000000"),
        )
    } else {
        ("#000000", "#ffffff")
    };
    let mut output = Vec::new();
    append(&mut output, &format!(
        "<html><body><!--StartFragment--><pre><div style='color: {fg}; background-color: {bg}; font-family: {}; font-size: {}px;'>",
        config.font_family, ryu_js::Buffer::new().format(config.font_size)
    ));
    output
}

fn emit_cell(output: &mut Vec<u16>, cell: &CellData, previous: &CellData, palette: &[String; 256]) {
    if cell.get_width() == 0 {
        return;
    }
    if let Some(styles) = diff_style(cell, previous, palette) {
        append(output, "</span><span");
        if !styles.is_empty() {
            append(output, &format!(" style='{}'", styles.join(" ")));
        }
        append(output, ">");
    }
    let chars = cell.get_chars();
    match chars.as_slice() {
        [] => output.push(u16::from(b' ')),
        [38] => append(output, "&amp;"),
        [60] => append(output, "&lt;"),
        _ => output.extend(chars),
    }
}

fn diff_style(
    cell: &CellData,
    previous: &CellData,
    palette: &[String; 256],
) -> Option<Vec<String>> {
    if cell.attributes_equals(previous) {
        return None;
    }
    if style::equal_fg(cell, previous)
        && style::equal_bg(cell, previous)
        && style::equal_flags(cell, previous)
    {
        return None;
    }
    let colors = palette;
    let mut styles = Vec::new();
    for (name, color, rgb, palette) in [
        (
            "color",
            cell.get_fg_color(),
            cell.is_fg_rgb(),
            cell.is_fg_palette(),
        ),
        (
            "background-color",
            cell.get_bg_color(),
            cell.is_bg_rgb(),
            cell.is_bg_palette(),
        ),
    ] {
        if let Some(color) = hex_color(color, rgb, palette, colors) {
            styles.push(format!("{name}: {color};"));
        }
    }
    if cell.is_inverse() != 0 {
        styles.push("color: #000000; background-color: #BFBFBF;".into());
    }
    if cell.is_bold() != 0 {
        styles.push("font-weight: bold;".into());
    }
    let mut decorations = Vec::new();
    if cell.is_underline() != 0 {
        decorations.push(match cell.get_underline_style() {
            underline_style::DOUBLE => "underline double",
            underline_style::CURLY => "underline wavy",
            underline_style::DOTTED => "underline dotted",
            underline_style::DASHED => "underline dashed",
            _ => "underline",
        });
    }
    if cell.is_overline() != 0 {
        decorations.push("overline");
    }
    if cell.is_strikethrough() != 0 {
        decorations.push("line-through");
    }
    if cell.is_blink() != 0 {
        decorations.push("blink");
    }
    if !decorations.is_empty() {
        styles.push(format!("text-decoration: {};", decorations.join(" ")));
    }
    if cell.is_underline() != 0 && !cell.is_underline_color_default() {
        if let Some(color) = hex_color(
            cell.get_underline_color(),
            cell.is_underline_color_rgb(),
            true,
            colors,
        ) {
            styles.push(format!("text-decoration-color: {color};"));
        }
    }
    if cell.is_invisible() != 0 {
        styles.push("visibility: hidden;".into());
    }
    if cell.is_italic() != 0 {
        styles.push("font-style: italic;".into());
    }
    if cell.is_dim() != 0 {
        styles.push("opacity: 0.5;".into());
    }
    Some(styles)
}

fn hex_color(color: i32, rgb: bool, palette: bool, colors: &[String; 256]) -> Option<String> {
    if rgb {
        return Some(format!("#{:06x}", color & 0xffffff));
    }
    if palette {
        // Packed palette getters guarantee the index is in 0..=255.
        return Some(colors[color as usize].clone());
    }
    None
}
