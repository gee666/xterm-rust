// Copyright (c) 2022-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{Link, Position, Range};
use xterm_core::buffer::BufferLine;

/// Find OSC 8 links for a one-based buffer row. Missing rows return `None`, empty
/// rows return `Some([])`, matching the source callback's undefined/array distinction.
///
/// `is_http_url` must use the host's WHATWG URL parser, accepting only http/https.
/// It is not called when `allow_non_http_protocols` is true. URI lookup returns
/// owned strings so buffer borrows never survive a host callback.
pub fn osc_links(
    lines: &[BufferLine],
    y: usize,
    mut uri: impl FnMut(u64) -> Option<Vec<u16>>,
    allow_non_http_protocols: bool,
    mut is_http_url: impl FnMut(&[u16]) -> bool,
) -> Option<Vec<Link>> {
    let line = lines.get(y.checked_sub(1)?)?;
    let length = line.get_trimmed_length();
    let mut result = Vec::new();
    let mut current: Option<(usize, u64)> = None;
    for x in 0..length {
        if current.is_none() && line.has_content(x).unwrap_or(0) == 0 {
            continue;
        }
        let mut id = url_id(line, x);
        if current.is_none() && id != 0 {
            // The upstream loop continues here, including on the last column.
            // Preserve that behavior rather than inventing a final flush.
            current = Some((x, id));
            continue;
        }
        if let Some((start, current_id)) = current {
            let finish = id != current_id;
            if finish || x == length - 1 {
                if let Some(text) = uri(current_id).filter(|text| !text.is_empty()) {
                    let end = x + usize::from(!finish && x == length - 1);
                    let range = wrapped_range(lines, y, start, end, current_id, &mut id);
                    if allow_non_http_protocols || is_http_url(&text) {
                        result.push(Link { text, range });
                    }
                }
                current = (id != 0).then_some((x, id));
            }
        }
    }
    Some(result)
}

fn url_id(line: &BufferLine, x: usize) -> u64 {
    line.get_extended(x).map(|ext| ext.url_id()).unwrap_or(0)
}

fn wrapped_range(
    lines: &[BufferLine],
    y: usize,
    start: usize,
    end: usize,
    id: u64,
    work_id: &mut u64,
) -> Range {
    // Upstream range expansion loads into the SAME work cell as the outer scan.
    // Preserve its last probe so the next run sees that mutation, including when
    // the probe failed. See the retained shared-work-cell counterexample.
    let mut has_id = |line: &BufferLine, x| {
        *work_id = url_id(line, x);
        *work_id == id && id != 0
    };
    let mut start_y = y;
    let mut start_x = start;
    let mut end_y = y;
    let mut end_x = end;
    while start_x == 0 {
        let Some(current) = lines.get(start_y - 1) else {
            break;
        };
        if !current.is_wrapped || start_y < 2 {
            break;
        }
        let previous = &lines[start_y - 2];
        let length = previous.get_trimmed_length();
        if length == 0 || !has_id(previous, length - 1) {
            break;
        }
        start_x = length - 1;
        while start_x > 0 && has_id(previous, start_x - 1) {
            start_x -= 1;
        }
        start_y -= 1;
    }
    loop {
        let Some(current) = lines.get(end_y - 1) else {
            break;
        };
        if end_x != current.get_trimmed_length() {
            break;
        }
        let Some(next) = lines.get(end_y).filter(|next| next.is_wrapped) else {
            break;
        };
        let length = next.get_trimmed_length();
        if length == 0 || !has_id(next, 0) {
            break;
        }
        end_x = 1;
        while end_x < length && has_id(next, end_x) {
            end_x += 1;
        }
        end_y += 1;
    }
    Range {
        start: Position {
            x: start_x + 1,
            y: start_y,
        },
        end: Position { x: end_x, y: end_y },
    }
}
