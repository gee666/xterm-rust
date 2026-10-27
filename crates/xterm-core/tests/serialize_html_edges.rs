mod serialize_html_support;
use serialize_html_support::*;
use xterm_core::{
    buffer::{
        constants::{
            attributes as color, bg_flags as bg, content, fg_flags as fg, underline_style,
        },
        CellData,
    },
    serialize::{serialize_html, BufferAccess, HtmlOptions, HtmlRange, SerializeError},
};

#[test]
fn exact_wrappers_empty_range_missing_line_and_scrollback_clamp() {
    let text = buffer(&["a", "b", "c", "d"], 1);
    let header = "<html><body><!--StartFragment--><pre><div style='color: #000000; background-color: #ffffff; font-family: monospace; font-size: 15px;'>";
    let footer = "</div></pre><!--EndFragment--></body></html>";
    for (scrollback, expected) in [
        (None, "abcd"),
        (Some(0), "cd"),
        (Some(1), "bcd"),
        (Some(100), "abcd"),
        (Some(-1), "d"),
        (Some(-2), ""),
        (Some(isize::MIN), ""),
    ] {
        let rows: String = expected
            .chars()
            .map(|c| format!("<div><span>{c}</span></div>"))
            .collect();
        assert_eq!(
            html(
                &text,
                1,
                &HtmlOptions {
                    scrollback,
                    ..HtmlOptions::default()
                },
                None,
                &config()
            ),
            format!("{header}{rows}{footer}")
        );
    }
    let empty = buffer(&[], 1);
    assert_eq!(
        html(&empty, 1, &HtmlOptions::default(), None, &config()),
        format!("{header}{footer}")
    );
    let missing = HtmlOptions {
        range: Some(HtmlRange {
            start_line: 5,
            end_line: 5,
            start_col: 0,
        }),
        ..HtmlOptions::default()
    };
    assert_eq!(
        html(&text, 1, &missing, None, &config()),
        format!("{header}<div><span></span></div>{footer}")
    );
    let reversed = HtmlOptions {
        range: Some(HtmlRange {
            start_line: 5,
            end_line: 4,
            start_col: 0,
        }),
        ..HtmlOptions::default()
    };
    assert_eq!(
        html(&text, 1, &reversed, None, &config()),
        format!("{header}{footer}")
    );
}

#[test]
fn selection_multiline_end_exclusive_empty_and_range_precedence() {
    let text = buffer(&["abcd", "efgh", "ijkl"], 4);
    let options = HtmlOptions {
        only_selection: true,
        scrollback: Some(-100),
        ..HtmlOptions::default()
    };
    let output = html(&text, 4, &options, Some(range(2, 0, 2, 2)), &config());
    assert!(output.contains(
        "<div><span>cd</span></div><div><span>efgh</span></div><div><span>ij</span></div>"
    ));
    let empty = html(&text, 4, &options, Some(range(2, 0, 2, 0)), &config());
    assert!(empty.contains("<div><span></span></div>"));
    let zero_end = html(&text, 4, &options, Some(range(2, 0, 0, 1)), &config());
    assert!(zero_end.contains("<div><span>cd</span></div><div><span></span></div>"));
    let explicit = HtmlOptions {
        range: Some(HtmlRange {
            start_line: 1,
            end_line: 1,
            start_col: 1,
        }),
        ..options
    };
    let output = html(&text, 4, &explicit, None, &config());
    assert!(output.contains("<div><span>fgh</span></div>"));
    assert!(!output.contains("abcd"));
}

#[test]
fn wide_placeholders_advance_old_style_and_rows_do_not_reset_it() {
    let mut text = buffer(&["界x", "yz"], 3);
    text.lines[0][0].content = (2 << content::WIDTH_SHIFT) | u32::from('界');
    let mut placeholder = CellData::default();
    placeholder.fg = fg::BOLD;
    text.lines[0].insert(1, placeholder);
    text.lines[0].truncate(3);
    text.lines[0][2].fg = fg::BOLD;
    text.lines[1][0].fg = fg::BOLD;
    let output = html(&text, 3, &HtmlOptions::default(), None, &config());
    // The invisible placeholder still became oldCell. Therefore no bold span
    // starts for x, and no bold span starts again for y on the next row.
    assert!(
        output.contains("<div><span>界x</span></div><div><span>y</span><span>z </span></div>"),
        "{output}"
    );
}

#[test]
fn whole_cell_escaping_and_lone_utf16_units_match_source_policy() {
    let mut text = buffer(&["abcde"], 5);
    text.lines[0] = vec![
        cell("&"),
        cell("<"),
        cell(">\"'"),
        cell("&\u{301}"),
        cell(""),
    ];
    text.lines[0][4].content = (1 << content::WIDTH_SHIFT) | 0xd800;
    let output = serialize_html(&text, 5, 2, None, &config(), &HtmlOptions::default()).unwrap();
    let expected: Vec<u16> = "&amp;&lt;>\"'&\u{301}"
        .encode_utf16()
        .chain([0xd800])
        .collect();
    assert!(output
        .windows(expected.len())
        .any(|window| window == expected));
}

#[test]
fn all_decorations_follow_order_and_colors_use_leading_zeroes() {
    let mut text = buffer(&["x"], 1);
    let cell = &mut text.lines[0][0];
    cell.fg = fg::INVERSE
        | fg::BOLD
        | fg::UNDERLINE
        | fg::STRIKETHROUGH
        | fg::BLINK
        | fg::INVISIBLE
        | color::CM_RGB
        | 0x010203;
    cell.bg = bg::OVERLINE | bg::ITALIC | bg::DIM | color::CM_RGB | 0x00000f;
    cell.extended.set_underline_style(underline_style::DOUBLE);
    cell.extended.set_underline_color(color::CM_RGB | 0x000102);
    cell.update_extended();
    let output = html(&text, 1, &HtmlOptions::default(), None, &config());
    let expected = "color: #010203; background-color: #00000f; color: #000000; background-color: #BFBFBF; font-weight: bold; text-decoration: underline double overline line-through blink; text-decoration-color: #000102; visibility: hidden; font-style: italic; opacity: 0.5;";
    assert!(
        output.contains(&format!("<span style='{expected}'>x</span>")),
        "{output}"
    );
    for style in [underline_style::NONE, 6, 7] {
        text.lines[0][0].extended.set_underline_style(style);
        assert!(html(&text, 1, &HtmlOptions::default(), None, &config())
            .contains("text-decoration: underline overline line-through blink;"));
    }
}

#[test]
fn default_palette_endpoints_theme_overrides_and_font_js_numbers() {
    let mut text = buffer(&["x"], 1);
    for (index, css) in [
        (0, "#2e3436"),
        (15, "#eeeeec"),
        (16, "#000000"),
        (231, "#ffffff"),
        (232, "#080808"),
        (255, "#eeeeee"),
    ] {
        text.lines[0][0].fg = color::CM_P256 | index;
        assert!(html(&text, 1, &HtmlOptions::default(), None, &config())
            .contains(&format!("color: {css};")));
    }
    let mut config = config();
    config.ansi_colors = Some(std::array::from_fn(|i| format!("theme-{i}")));
    assert!(html(&text, 1, &HtmlOptions::default(), None, &config).contains("color: theme-255;"));
    for (number, expected) in [
        (1e21, "1e+21"),
        (1e-7, "1e-7"),
        (-0.0, "0"),
        (f64::NAN, "NaN"),
        (f64::INFINITY, "Infinity"),
    ] {
        config.font_size = number;
        assert!(html(&text, 1, &HtmlOptions::default(), None, &config)
            .contains(&format!("font-size: {expected}px;")));
    }
}

struct Borrowed;
impl BufferAccess for Borrowed {
    fn len(&self) -> usize {
        1
    }
    fn line_len(&self, _: usize) -> Result<Option<usize>, SerializeError> {
        Err(SerializeError::BorrowedLine)
    }
    fn cell(&self, _: usize, _: usize) -> Result<Option<CellData>, SerializeError> {
        Err(SerializeError::BorrowedLine)
    }
    fn is_wrapped(&self, _: usize) -> Result<bool, SerializeError> {
        Err(SerializeError::BorrowedLine)
    }
    fn cursor_x(&self) -> usize {
        0
    }
    fn cursor_y(&self) -> usize {
        0
    }
    fn base_y(&self) -> usize {
        0
    }
}
#[test]
fn borrowed_buffer_errors_propagate_instead_of_emitting_successful_empty_html() {
    assert_eq!(
        serialize_html(&Borrowed, 1, 2, None, &config(), &HtmlOptions::default()),
        Err(SerializeError::BorrowedLine)
    );
}
