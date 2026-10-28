// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use xterm_core::buffer::Buffer;

fn units(text: &[u16]) -> String {
    if text.is_empty() {
        "-".into()
    } else {
        text.iter()
            .map(|unit| format!("{unit:x}"))
            .collect::<Vec<_>>()
            .join(",")
    }
}
pub fn buffer_snapshot(buffer: &Buffer) -> String {
    let mut rows = Vec::new();
    for row in 0..buffer.lines.len() {
        let line = buffer.lines.get(row).expect("trace contains dense lines");
        let line = line.borrow();
        let mut cells = Vec::new();
        for col in 0..line.len() {
            let cell = line
                .get_cell(col)
                .expect("trace has no unresolved extended cells");
            cells.push(format!(
                "{}:{}:{}:{}",
                cell.content,
                cell.fg,
                cell.bg,
                units(&cell.get_chars())
            ));
        }
        rows.push(format!("{};{}", u8::from(line.is_wrapped), cells.join("/")));
    }
    let tabs = buffer
        .tabs
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}",
        buffer.x,
        buffer.y,
        buffer.ybase,
        buffer.ydisp,
        buffer.scroll_top,
        buffer.scroll_bottom,
        buffer.lines.len(),
        buffer.lines.max_length(),
        u8::from(buffer.has_scrollback()),
        tabs,
        rows.join("|")
    )
}
