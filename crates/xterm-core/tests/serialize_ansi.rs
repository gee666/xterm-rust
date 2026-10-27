// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use xterm_core::{
    buffer::{
        constants::{attributes as color, bg_flags, fg_flags},
        AttributeData, BufferLine, CellData,
    },
    serialize::*,
};

struct Grid {
    lines: Vec<BufferLine>,
    x: usize,
    y: usize,
    base: usize,
}
impl Grid {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            lines: (0..rows)
                .map(|_| BufferLine::new(cols, None, false))
                .collect(),
            x: 0,
            y: 0,
            base: 0,
        }
    }
    fn text(&mut self, row: usize, col: usize, text: &str, attrs: &AttributeData) {
        for (i, c) in text.chars().enumerate() {
            self.lines[row].set_cell_from_codepoint(col + i, u32::from(c), 1, attrs);
        }
    }
    fn ansi(&self, attrs: &AttributeData) -> Vec<u16> {
        serialize_buffer(
            self,
            self.lines[0].len(),
            2,
            attrs,
            CellRange {
                start: Position { x: 0, y: 0 },
                end: Position {
                    x: self.lines[0].len(),
                    y: self.lines.len() - 1,
                },
            },
            false,
        )
        .unwrap()
    }
}
impl BufferAccess for Grid {
    fn len(&self) -> usize {
        self.lines.len()
    }
    fn line_len(&self, row: usize) -> Result<Option<usize>, SerializeError> {
        Ok(self.lines.get(row).map(BufferLine::len))
    }
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        Ok(self.lines.get(row).and_then(|line| line.get_cell(col)))
    }
    fn is_wrapped(&self, row: usize) -> Result<bool, SerializeError> {
        Ok(self.lines.get(row).is_some_and(|l| l.is_wrapped))
    }
    fn cursor_x(&self) -> usize {
        self.x
    }
    fn cursor_y(&self) -> usize {
        self.y
    }
    fn base_y(&self) -> usize {
        self.base
    }
}
fn units(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn fg(n: u32) -> AttributeData {
    AttributeData {
        fg: color::CM_P16 | n,
        ..AttributeData::default()
    }
}

#[test]
fn restoring_cursor_styles_upstream() {
    let mut g = Grid::new(10, 2);
    g.text(0, 0, "> ", &fg(2));
    g.x = 2;
    assert_eq!(
        g.ansi(&AttributeData::default()),
        units("\x1b[32m> \x1b[0m")
    );
}
#[test]
fn explicit_ranges_upstream() {
    let mut g = Grid::new(10, 2);
    let a = AttributeData::default();
    g.text(0, 0, "hello", &a);
    g.text(1, 0, "world", &a);
    for (start, end, want) in [(0, 0, "hello"), (0, 1, "hello\r\nworld"), (1, 1, "world")] {
        assert_eq!(
            serialize_buffer(
                &g,
                10,
                2,
                &a,
                CellRange {
                    start: Position { x: 0, y: start },
                    end: Position { x: 10, y: end }
                },
                true
            )
            .unwrap(),
            units(want)
        );
    }
}
#[test]
fn underline_variants_upstream() {
    for (n, want) in [(1, "4"), (2, "4:2"), (3, "4:3"), (4, "4:4"), (5, "4:5")] {
        let mut g = Grid::new(10, 2);
        let mut a = AttributeData {
            fg: fg_flags::UNDERLINE,
            ..AttributeData::default()
        };
        a.extended.set_underline_style(n);
        a.update_extended();
        g.text(0, 0, "test", &a);
        g.x = 4;
        assert_eq!(
            g.ansi(&AttributeData::default()),
            units(&format!("\x1b[{want}mtest\x1b[0m"))
        );
    }
}
#[test]
fn underline_colors_upstream() {
    for (c, want) in [
        (color::CM_RGB | 0xff8040, "4:1;58:2::255:128:64"),
        (color::CM_P256 | 46, "4:1;58:5:46"),
    ] {
        let mut g = Grid::new(10, 2);
        let mut a = AttributeData {
            fg: fg_flags::UNDERLINE,
            ..AttributeData::default()
        };
        a.extended.set_underline_style(1);
        a.extended.set_underline_color(c);
        a.update_extended();
        g.text(0, 0, "test", &a);
        g.x = 4;
        assert_eq!(
            g.ansi(&AttributeData::default()),
            units(&format!("\x1b[{want}mtest\x1b[0m"))
        );
    }
}
#[test]
fn empty_and_internal_gaps_are_distinct_from_spaces() {
    let mut g = Grid::new(10, 2);
    let a = AttributeData::default();
    assert!(g.ansi(&a).is_empty());
    g.text(0, 3, "a ", &a);
    g.x = 5;
    assert_eq!(g.ansi(&a), units("\x1b[3Ca "));
}
#[test]
fn colored_nulls_erase_and_do_not_emit_spaces() {
    let mut g = Grid::new(10, 2);
    let a = AttributeData {
        bg: color::CM_P16 | 2,
        ..AttributeData::default()
    };
    for i in 0..3 {
        g.lines[0].set_cell_from_codepoint(i, 0, 1, &a);
    }
    g.text(0, 3, "x", &a);
    g.x = 4;
    assert_eq!(
        g.ansi(&AttributeData::default()),
        units("\x1b[42m\x1b[3X\x1b[3Cx\x1b[0m")
    );
}
#[test]
fn natural_wrap_has_no_separator() {
    let mut g = Grid::new(4, 2);
    let a = AttributeData::default();
    g.text(0, 0, "abcd", &a);
    g.text(1, 0, "ef", &a);
    g.lines[1].is_wrapped = true;
    g.x = 2;
    g.y = 1;
    assert_eq!(g.ansi(&a), units("abcdef"));
}
#[test]
fn impossible_wrap_uses_temporary_text_and_erases_it() {
    let mut g = Grid::new(4, 2);
    let a = AttributeData::default();
    g.text(0, 0, "a", &a);
    g.text(1, 0, "b", &a);
    g.lines[1].is_wrapped = true;
    g.x = 1;
    g.y = 1;
    assert_eq!(
        g.ansi(&a),
        units("a----\x1b[1D\x1b[1X\x1b[A\x1b[1C\x1b[3X\x1b[1D\x1b[Bb")
    );
}
#[test]
fn wide_placeholders_and_lone_surrogates_are_lossless() {
    let mut g = Grid::new(4, 2);
    let a = AttributeData::default();
    g.lines[0].set_cell_from_codepoint(0, 0x4e00, 2, &a);
    g.lines[0].set_cell_from_codepoint(1, 0, 0, &a);
    g.lines[0].set_cell_from_codepoint(2, 0xd800, 1, &a);
    g.x = 3;
    let mut want = units("一");
    want.push(0xd800);
    assert_eq!(g.ansi(&a), want);
}
#[test]
fn colors_cover_palette_bright_and_rgb() {
    for (bits, want) in [
        (color::CM_P16 | 15, "97"),
        (color::CM_P256 | 255, "38;5;255"),
        (color::CM_RGB | 0x00ff01, "38;2;0;255;1"),
    ] {
        let mut g = Grid::new(4, 2);
        let a = AttributeData {
            fg: bits,
            ..AttributeData::default()
        };
        g.text(0, 0, "x", &a);
        g.x = 1;
        assert_eq!(g.ansi(&a), units(&format!("\x1b[{want}mx")));
    }
}
#[test]
fn all_flags_preserve_source_order() {
    let mut g = Grid::new(4, 2);
    let a = AttributeData {
        fg: fg_flags::INVERSE
            | fg_flags::BOLD
            | fg_flags::UNDERLINE
            | fg_flags::BLINK
            | fg_flags::INVISIBLE
            | fg_flags::STRIKETHROUGH,
        bg: bg_flags::OVERLINE | bg_flags::ITALIC | bg_flags::DIM,
        ..AttributeData::default()
    };
    g.text(0, 0, "x", &a);
    g.x = 1;
    assert_eq!(g.ansi(&a), units("\x1b[7;1;4;53;5;8;3;2;9mx"));
}
#[test]
fn cursor_moves_vertical_before_horizontal() {
    let mut g = Grid::new(10, 2);
    let a = AttributeData::default();
    g.text(0, 0, "abc", &a);
    g.x = 1;
    g.y = 1;
    assert_eq!(g.ansi(&a), units("abc\x1b[1B\x1b[2D"));
}
#[test]
fn normal_alternate_modes_and_margins_have_source_order() {
    let mut normal = Grid::new(10, 2);
    let mut alt = Grid::new(10, 2);
    let a = AttributeData::default();
    normal.text(0, 0, "n", &a);
    normal.x = 1;
    alt.text(0, 0, "a", &a);
    alt.x = 1;
    let modes = SerializeModes {
        application_cursor_keys: true,
        application_keypad: true,
        bracketed_paste: true,
        insert: true,
        origin: true,
        reverse_wraparound: true,
        send_focus: true,
        wraparound: false,
        mouse_tracking: MouseTracking::Drag,
        show_cursor: false,
    };
    let state = SerializeState {
        normal: &normal,
        alternate: &alt,
        active_alternate: true,
        cols: 10,
        rows: 2,
        current_attrs: &a,
        modes: &modes,
        scroll_top: 1,
        scroll_bottom: 1,
    };
    assert_eq!(serialize_terminal(&state,&SerializeOptions::default()).unwrap(),units("n\x1b[?1049h\x1b[Ha\x1b[?1h\x1b[?66h\x1b[?2004h\x1b[4h\x1b[?6h\x1b[?45h\x1b[?1004h\x1b[?7l\x1b[?1002h\x1b[?25l\x1b[2;2r"));
    assert_eq!(
        serialize_terminal(
            &state,
            &SerializeOptions {
                exclude_modes: true,
                exclude_alt_buffer: true,
                ..SerializeOptions::default()
            }
        )
        .unwrap(),
        units("n")
    );
}
#[test]
fn zero_selected_rows_and_reversed_ranges_report_source_range_error() {
    let g = Grid::new(10, 2);
    let a = AttributeData::default();
    let modes = SerializeModes::default();
    let state = SerializeState {
        normal: &g,
        alternate: &g,
        active_alternate: false,
        cols: 10,
        rows: 2,
        current_attrs: &a,
        modes: &modes,
        scroll_top: 0,
        scroll_bottom: 1,
    };
    assert_eq!(
        serialize_terminal(
            &state,
            &SerializeOptions {
                scrollback: Some(-2),
                ..SerializeOptions::default()
            }
        ),
        Err(SerializeError::InvalidRange)
    );
    assert_eq!(
        serialize_terminal(
            &state,
            &SerializeOptions {
                range: Some(LineRange { start: 1, end: 0 }),
                ..SerializeOptions::default()
            }
        ),
        Err(SerializeError::InvalidRange)
    );
}
