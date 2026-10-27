// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// HTML assertions ported from addon-serialize/src/SerializeAddon.test.ts:227-438.
mod serialize_html_support;
use serialize_html_support::*;
use xterm_core::{
    buffer::constants::{attributes as color, bg_flags as bg, fg_flags as fg, underline_style},
    serialize::{HtmlOptions, HtmlRange},
};

fn marked(
    buffer: &mut Buffer,
    start: usize,
    end: usize,
    fg: u32,
    bg: u32,
    style: u32,
    underline_color: u32,
) {
    for cell in &mut buffer.lines[0][start..end] {
        cell.fg = fg;
        cell.bg = bg;
        cell.extended.set_underline_style(style);
        cell.extended.set_underline_color(underline_color);
        cell.update_extended();
    }
}
fn contains_span(output: &str, style: &str, text: &str) {
    assert_eq!(
        output
            .matches(&format!("<span style='{style}'>{text}</span>"))
            .count(),
        1,
        "{output}"
    );
}

#[test]
fn upstream_empty_selection_unsafe_chars_and_explicit_range() {
    let empty = buffer(&["", ""], 10);
    let output = html(&empty, 10, &HtmlOptions::default(), None, &config());
    assert!(!output.is_empty());
    assert_eq!(
        output.matches("<div><span>          </span></div>").count(),
        2
    );
    let selection_options = HtmlOptions {
        only_selection: true,
        ..HtmlOptions::default()
    };
    assert_eq!(html(&empty, 10, &selection_options, None, &config()), "");
    let text = buffer(&[" terminal ", ""], 10);
    let output = html(
        &text,
        10,
        &selection_options,
        Some(range(1, 0, 9, 0)),
        &config(),
    );
    assert_eq!(
        output.matches("<div><span>terminal</span></div>").count(),
        1
    );
    let unsafe_chars = buffer(&[" <a>&pi; ", ""], 10);
    let output = html(
        &unsafe_chars,
        10,
        &selection_options,
        Some(range(1, 0, 8, 0)),
        &config(),
    );
    assert_eq!(
        output
            .matches("<div><span>&lt;a>&amp;pi;</span></div>")
            .count(),
        1
    );
    let text = buffer(&["bye hello", "world"], 10);
    let output = html(
        &text,
        10,
        &HtmlOptions {
            range: Some(HtmlRange {
                start_line: 0,
                end_line: 0,
                start_col: 4,
            }),
            ..HtmlOptions::default()
        },
        None,
        &config(),
    );
    assert_eq!(output.matches("<div><span>").count(), 1);
    assert!(output.contains("hello"));
    assert!(!output.contains("bye"));
    assert!(!output.contains("world"));
}

#[test]
fn upstream_single_flag_style_assertions() {
    for (fg, bg, expected) in [
        (fg::BOLD, 0, "font-weight: bold;"),
        (0, bg::ITALIC, "font-style: italic;"),
        (fg::INVERSE, 0, "color: #000000; background-color: #BFBFBF;"),
        (fg::INVISIBLE, 0, "visibility: hidden;"),
        (0, bg::DIM, "opacity: 0.5;"),
        (fg::STRIKETHROUGH, 0, "text-decoration: line-through;"),
    ] {
        let mut text = buffer(&[" terminal ", ""], 10);
        marked(&mut text, 1, 9, fg, bg, 0, 0);
        contains_span(
            &html(&text, 10, &HtmlOptions::default(), None, &config()),
            expected,
            "terminal",
        );
    }
}

#[test]
fn upstream_underline_style_and_color_assertions() {
    for (style, expected) in [
        (underline_style::SINGLE, "underline"),
        (underline_style::DOUBLE, "underline double"),
        (underline_style::CURLY, "underline wavy"),
        (underline_style::DOTTED, "underline dotted"),
        (underline_style::DASHED, "underline dashed"),
    ] {
        let mut text = buffer(&[" terminal ", ""], 10);
        marked(&mut text, 1, 9, fg::UNDERLINE, 0, style, 0);
        contains_span(
            &html(&text, 10, &HtmlOptions::default(), None, &config()),
            &format!("text-decoration: {expected};"),
            "terminal",
        );
    }
    for (color, expected) in [
        (color::CM_P256 | 46, "#00ff00"),
        (color::CM_RGB | 0xff8040, "#ff8040"),
    ] {
        let mut text = buffer(&[" terminal ", ""], 10);
        marked(
            &mut text,
            1,
            9,
            fg::UNDERLINE,
            0,
            underline_style::SINGLE,
            color,
        );
        let output = html(&text, 10, &HtmlOptions::default(), None, &config());
        assert!(output.contains("text-decoration: underline;"));
        assert!(output.contains(&format!("text-decoration-color: {expected};")));
    }
}

#[test]
fn upstream_combined_style_assertions() {
    let mut text = buffer(&[" terminal ", ""], 10);
    marked(&mut text, 0, 1, fg::BOLD, 0, 0, 0);
    marked(&mut text, 1, 6, fg::BOLD | fg::STRIKETHROUGH, 0, 0, 0);
    marked(&mut text, 6, 9, fg::STRIKETHROUGH, 0, 0, 0);
    let output = html(&text, 10, &HtmlOptions::default(), None, &config());
    contains_span(&output, "font-weight: bold;", " ");
    contains_span(
        &output,
        "font-weight: bold; text-decoration: line-through;",
        "termi",
    );
    contains_span(&output, "text-decoration: line-through;", "nal");
}

#[test]
fn upstream_foreground_background_theme_and_headless_assertions() {
    for (fg, bg, property) in [
        (color::CM_P256 | 46, 0, "color"),
        (0, color::CM_P256 | 46, "background-color"),
    ] {
        let mut text = buffer(&[" terminal ", ""], 10);
        marked(&mut text, 1, 9, fg, bg, 0, 0);
        contains_span(
            &html(&text, 10, &HtmlOptions::default(), None, &config()),
            &format!("{property}: #00ff00;"),
            "terminal",
        );
    }
    let mut text = buffer(&[" terminal ", ""], 10);
    marked(&mut text, 1, 9, color::CM_P256, 0, 0, 0);
    let mut config = config();
    let mut palette = xterm_core::color::palette::default_ansi_colors().map(|color| color.css);
    palette[0] = "#ffa500".into();
    config.ansi_colors = Some(palette);
    contains_span(
        &html(&text, 10, &HtmlOptions::default(), None, &config),
        "color: #ffa500;",
        "terminal",
    );
}

#[test]
fn upstream_global_default_custom_and_included_background_assertions() {
    let empty = buffer(&["", ""], 10);
    let mut config = config();
    let output = html(&empty, 10, &HtmlOptions::default(), None, &config);
    assert_eq!(output.matches("color: #000000; background-color: #ffffff; font-family: monospace; font-size: 15px;").count(), 1);
    let include = HtmlOptions {
        include_global_background: true,
        ..HtmlOptions::default()
    };
    let output = html(&empty, 10, &include, None, &config);
    assert_eq!(output.matches("color: #ffffff; background-color: #000000; font-family: monospace; font-size: 15px;").count(), 1);
    config.font_family = "verdana".into();
    config.font_size = 20.0;
    config.foreground = Some("#ff00ff".into());
    config.background = Some("#00ff00".into());
    let output = html(&empty, 10, &include, None, &config);
    assert_eq!(
        output
            .matches(
                "color: #ff00ff; background-color: #00ff00; font-family: verdana; font-size: 20px;"
            )
            .count(),
        1
    );
}
