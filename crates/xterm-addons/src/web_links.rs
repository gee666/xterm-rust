// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-web-links/src/WebLinkProvider.ts.

//! Link windowing and UTF-16-to-cell mapping. The host evaluates RegExp and
//! WHATWG URL with its platform's standard implementations. No terminal logic
//! is delegated to JavaScript. A browser host must use JS UTF-16 match indices.

use std::fmt;

pub const STRICT_URL_REGEX: &str =
    r#"(https?|HTTPS?):[/]{2}[^\s"'!*(){}|\\\^<>`]*[^\s"':,.!?{}|\\\^~\[\]`()<>]"#;
const MAX_EXPANSION: usize = 2048;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegexSpec {
    pub source: String,
    pub flags: String,
}
impl Default for RegexSpec {
    fn default() -> Self {
        Self {
            source: STRICT_URL_REGEX.into(),
            flags: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkCell {
    pub chars: Vec<u16>,
    pub width: u8,
}

pub trait LinkLine {
    fn is_wrapped(&self) -> bool;
    fn translate_trimmed(&self) -> Vec<u16>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn cell(&self, column: usize) -> Option<LinkCell>;
}

pub trait LinkBuffer {
    fn line(&self, index: usize) -> Option<&dyn LinkLine>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkMatch {
    pub index: usize,
    pub text: Vec<u16>,
}

#[derive(Clone, Debug)]
pub enum LinkError {
    Message(String),
    Host(std::rc::Rc<dyn std::error::Error>),
}
impl LinkError {
    pub fn host(error: impl std::error::Error + 'static) -> Self {
        Self::Host(std::rc::Rc::new(error))
    }
}
impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => f.write_str(message),
            Self::Host(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for LinkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Host(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

/// Matches include the URL validation used by upstream isUrl, not regex alone.
/// Empty matches must return an error. Upstream loops forever for such custom
/// regexes; Rust deliberately rejects them instead of hanging the browser.
pub trait LinkMatcher {
    fn matches(&self, text: &[u16], regex: &RegexSpec) -> Result<Vec<LinkMatch>, LinkError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkRange {
    pub start: Position,
    pub end: Position,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub text: Vec<u16>,
    pub range: LinkRange,
}

pub fn compute_links(
    y: usize,
    regex: &RegexSpec,
    buffer: &dyn LinkBuffer,
    matcher: &dyn LinkMatcher,
) -> Result<Vec<Link>, LinkError> {
    let Some(index) = y.checked_sub(1) else {
        return Ok(Vec::new());
    };
    let (lines, top) = windowed_lines(index, buffer);
    let text: Vec<u16> = lines.into_iter().flatten().collect();
    let mut result = Vec::new();
    for candidate in matcher.matches(&text, regex)? {
        if candidate.text.is_empty() {
            return Err(LinkError::Message(
                "custom URL regex matched an empty string".into(),
            ));
        }
        if candidate
            .index
            .checked_add(candidate.text.len())
            .is_none_or(|end| end > text.len())
        {
            return Err(LinkError::Message(
                "URL matcher returned an out-of-bounds match".into(),
            ));
        }
        let Some((start_y, start_x)) = map_string_index(buffer, top, 0, candidate.index) else {
            continue;
        };
        let Some((end_y, end_x)) =
            map_string_index(buffer, start_y as isize, start_x, candidate.text.len())
        else {
            continue;
        };
        result.push(Link {
            text: candidate.text,
            range: LinkRange {
                start: Position {
                    x: start_x + 1,
                    y: start_y + 1,
                },
                end: Position {
                    x: end_x,
                    y: end_y + 1,
                },
            },
        });
    }
    Ok(result)
}

pub fn windowed_lines(index: usize, buffer: &dyn LinkBuffer) -> (Vec<Vec<u16>>, isize) {
    let mut top = index as isize;
    let mut bottom = index;
    let mut lines = Vec::new();
    if let Some(line) = buffer.line(index) {
        let content = line.translate_trimmed();
        if line.is_wrapped() && content.first() != Some(&32) {
            let mut length = 0;
            loop {
                // Preserve pre-decrement even on the failed lookup. The source
                // returns that top index, including -1 at a missing first line.
                top -= 1;
                let Some(line) = usize::try_from(top).ok().and_then(|i| buffer.line(i)) else {
                    break;
                };
                if length >= MAX_EXPANSION {
                    break;
                }
                let content = line.translate_trimmed();
                length += content.len();
                let stop = !line.is_wrapped() || content.contains(&32);
                lines.push(content);
                if stop {
                    break;
                }
            }
            lines.reverse();
        }
        lines.push(content);
        let mut length = 0;
        loop {
            bottom += 1;
            let Some(line) = buffer.line(bottom) else {
                break;
            };
            if !line.is_wrapped() || length >= MAX_EXPANSION {
                break;
            }
            let content = line.translate_trimmed();
            length += content.len();
            let stop = content.contains(&32);
            lines.push(content);
            if stop {
                break;
            }
        }
    }
    (lines, top)
}

pub fn map_string_index(
    buffer: &dyn LinkBuffer,
    mut line_index: isize,
    column: usize,
    string_index: usize,
) -> Option<(usize, usize)> {
    let mut remaining = i128::try_from(string_index).ok()?;
    let mut start = column;
    while remaining != 0 {
        let line = buffer.line(usize::try_from(line_index).ok()?)?;
        for index in start..line.len() {
            let cell = line.cell(index)?;
            if cell.width != 0 {
                remaining -= cell.chars.len().max(1) as i128;
                if index == line.len() - 1 && cell.chars.is_empty() {
                    if let Some(next) = buffer.line(usize::try_from(line_index + 1).ok()?) {
                        if next.is_wrapped() && next.cell(0).is_some_and(|cell| cell.width == 2) {
                            remaining += 1;
                        }
                    }
                }
            }
            if remaining < 0 {
                return Some((usize::try_from(line_index).ok()?, index));
            }
        }
        line_index += 1;
        start = 0;
    }
    Some((usize::try_from(line_index).ok()?, start))
}
