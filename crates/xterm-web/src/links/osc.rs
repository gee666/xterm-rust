// Copyright (c) 2022-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{Link, Position, Range};
use std::convert::Infallible;
use xterm_core::buffer::BufferLine;

/// Reads stay inside the callback. A live ring buffer can borrow its Rc row
/// briefly without copying scrollback or keeping a borrow across host callbacks.
pub trait OscBuffer {
    type Error;
    fn with_line<R>(
        &self,
        index: usize,
        read: impl FnOnce(&BufferLine) -> R,
    ) -> Result<Option<R>, Self::Error>;
}
impl<T: AsRef<[BufferLine]> + ?Sized> OscBuffer for T {
    type Error = Infallible;
    fn with_line<R>(
        &self,
        index: usize,
        read: impl FnOnce(&BufferLine) -> R,
    ) -> Result<Option<R>, Infallible> {
        Ok(self.as_ref().get(index).map(read))
    }
}

/// Slice convenience API. Missing rows return None; empty rows return Some([]).
/// `is_http_url` uses the host WHATWG parser and is bypassed when allowed.
pub fn osc_links(
    lines: &[BufferLine],
    y: usize,
    mut uri: impl FnMut(u64) -> Option<Vec<u16>>,
    allow_non_http_protocols: bool,
    is_http_url: impl FnMut(&[u16]) -> bool,
) -> Option<Vec<Link>> {
    match try_osc_links(
        lines,
        y,
        |id| Ok(uri(id)),
        allow_non_http_protocols,
        is_http_url,
    ) {
        Ok(links) => links,
        Err(error) => match error {},
    }
}

pub fn try_osc_links<B: OscBuffer + ?Sized>(
    lines: &B,
    y: usize,
    mut uri: impl FnMut(u64) -> Result<Option<Vec<u16>>, B::Error>,
    allow_non_http_protocols: bool,
    mut is_http_url: impl FnMut(&[u16]) -> bool,
) -> Result<Option<Vec<Link>>, B::Error> {
    let Some(row) = y.checked_sub(1) else {
        return Ok(None);
    };
    let Some(length) = lines.with_line(row, BufferLine::get_trimmed_length)? else {
        return Ok(None);
    };
    let mut result = Vec::new();
    let mut current: Option<(usize, u64)> = None;
    for x in 0..length {
        if current.is_none()
            && lines
                .with_line(row, |line| line.has_content(x).unwrap_or(0))?
                .unwrap_or(0)
                == 0
        {
            continue;
        }
        let mut id = url_id(lines, row, x)?;
        if current.is_none() && id != 0 {
            // Source continues even when this new run begins at trimmed EOL.
            current = Some((x, id));
            continue;
        }
        if let Some((start, current_id)) = current {
            let finish = id != current_id;
            if finish || x == length - 1 {
                if let Some(text) = uri(current_id)?.filter(|text| !text.is_empty()) {
                    let end = x + usize::from(!finish && x == length - 1);
                    let range = wrapped_range(lines, y, start, end, current_id, &mut id)?;
                    if allow_non_http_protocols || is_http_url(&text) {
                        result.push(Link { text, range });
                    }
                }
                current = (id != 0).then_some((x, id));
            }
        }
    }
    Ok(Some(result))
}
fn url_id<B: OscBuffer + ?Sized>(lines: &B, y: usize, x: usize) -> Result<u64, B::Error> {
    Ok(lines
        .with_line(y, |line| {
            line.get_extended(x).map(|ext| ext.url_id()).unwrap_or(0)
        })?
        .unwrap_or(0))
}
fn has_id<B: OscBuffer + ?Sized>(
    lines: &B,
    y: usize,
    x: usize,
    id: u64,
    work_id: &mut u64,
) -> Result<bool, B::Error> {
    *work_id = url_id(lines, y, x)?;
    Ok(*work_id == id && id != 0)
}
fn wrapped_range<B: OscBuffer + ?Sized>(
    lines: &B,
    y: usize,
    start: usize,
    end: usize,
    id: u64,
    work_id: &mut u64,
) -> Result<Range, B::Error> {
    // Range probes mutate the SAME work cell as the outer scan, including failed
    // probes. Preserve that mutation; see the shared-work-cell counterexample.
    let (mut start_y, mut start_x, mut end_y, mut end_x) = (y, start, y, end);
    while start_x == 0 {
        if !lines
            .with_line(start_y - 1, |line| line.is_wrapped)?
            .unwrap_or(false)
            || start_y < 2
        {
            break;
        }
        let Some(length) = lines.with_line(start_y - 2, BufferLine::get_trimmed_length)? else {
            break;
        };
        if length == 0 || !has_id(lines, start_y - 2, length - 1, id, work_id)? {
            break;
        }
        start_x = length - 1;
        while start_x > 0 && has_id(lines, start_y - 2, start_x - 1, id, work_id)? {
            start_x -= 1;
        }
        start_y -= 1;
    }
    loop {
        let Some(length) = lines.with_line(end_y - 1, BufferLine::get_trimmed_length)? else {
            break;
        };
        if end_x != length
            || !lines
                .with_line(end_y, |line| line.is_wrapped)?
                .unwrap_or(false)
        {
            break;
        }
        let Some(length) = lines.with_line(end_y, BufferLine::get_trimmed_length)? else {
            break;
        };
        if length == 0 || !has_id(lines, end_y, 0, id, work_id)? {
            break;
        }
        end_x = 1;
        while end_x < length && has_id(lines, end_y, end_x, id, work_id)? {
            end_x += 1;
        }
        end_y += 1;
    }
    Ok(Range {
        start: Position {
            x: start_x + 1,
            y: start_y,
        },
        end: Position { x: end_x, y: end_y },
    })
}
