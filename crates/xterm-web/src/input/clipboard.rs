// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from browser/Clipboard.ts.

/// Normalize LF and CRLF, leaving existing standalone CR untouched.
pub fn prepare_text_for_terminal(text: &[u16]) -> Vec<u16> {
    let mut result = Vec::with_capacity(text.len());
    let mut index = 0;
    while index < text.len() {
        let unit = text[index];
        if unit == 13 && text.get(index + 1) == Some(&10) {
            index += 1;
        }
        result.push(if unit == 10 { 13 } else { unit });
        index += 1;
    }
    result
}

/// Sanitizing ESC is intentionally conditional on bracketed mode.
pub fn bracket_text_for_paste(text: &[u16], bracketed: bool) -> Vec<u16> {
    if !bracketed {
        return text.to_vec();
    }
    let mut result: Vec<u16> = "\x1b[200~".encode_utf16().collect();
    result.extend(
        text.iter()
            .map(|unit| if *unit == 27 { 0x241b } else { *unit }),
    );
    result.extend("\x1b[201~".encode_utf16());
    result
}

pub fn paste(text: &[u16], bracketed_mode: bool, ignore_bracketed_mode: bool) -> Vec<u16> {
    bracket_text_for_paste(
        &prepare_text_for_terminal(text),
        bracketed_mode && !ignore_bracketed_mode,
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContextMenuPosition {
    pub left: f64,
    pub top: f64,
    pub width: u8,
    pub height: u8,
    pub z_index: u16,
}

pub fn context_menu_position(
    client_x: f64,
    client_y: f64,
    screen_left: f64,
    screen_top: f64,
) -> ContextMenuPosition {
    ContextMenuPosition {
        left: client_x - screen_left - 10.0,
        top: client_y - screen_top - 10.0,
        width: 20,
        height: 20,
        z_index: 1000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn units(value: &str) -> Vec<u16> {
        value.encode_utf16().collect()
    }

    #[test]
    fn upstream_line_endings() {
        for value in ["foo\nbar\n", "foo\r\nbar\r\n"] {
            assert_eq!(
                prepare_text_for_terminal(&units(value)),
                units("foo\rbar\r")
            );
        }
    }

    #[test]
    fn upstream_brackets() {
        assert_eq!(
            bracket_text_for_paste(&units("foo bar"), false),
            units("foo bar")
        );
        assert_eq!(
            bracket_text_for_paste(&units("foo bar"), true),
            units("\x1b[200~foo bar\x1b[201~")
        );
    }

    #[test]
    fn upstream_escape_sanitization() {
        let text = units("\x1b[201~foo\x1b[200~bar");
        assert_eq!(bracket_text_for_paste(&text, false), text);
        assert_eq!(
            bracket_text_for_paste(&text, true),
            units("\x1b[200~␛[201~foo␛[200~bar\x1b[201~")
        );
    }

    #[test]
    fn utf16_and_option_matrix() {
        let value = [0xd800, 13, 13, 10, 10, 27, 0xdc00];
        let prepared = [0xd800, 13, 13, 13, 27, 0xdc00];
        assert_eq!(prepare_text_for_terminal(&value), prepared);
        for (mode, ignore) in [(false, false), (false, true), (true, true)] {
            assert_eq!(paste(&value, mode, ignore), prepared);
        }
        assert_eq!(paste(&[], true, false), units("\x1b[200~\x1b[201~"));
        assert_eq!(context_menu_position(20.5, 5.0, 30.0, -2.0).left, -19.5);
    }
}
