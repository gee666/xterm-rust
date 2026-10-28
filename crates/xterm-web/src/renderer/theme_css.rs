// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Exact theme text from DomRenderer._injectCss, without owning DOM nodes.
use super::width_cache::Font;
use crate::theme::ColorSet;
use xterm_core::color::value::{multiply_opacity, opaque};

const INVERTED_DEFAULT_COLOR: usize = 257;

/// Generate source CSS for well-formed Unicode font strings and integral ids.
/// Number formatting is ECMAScript formatting, including non-finite raw values.
pub fn theme_css(id: u64, colors: &ColorSet, font: &Font, cursor_width: f64) -> String {
    let selector = format!(".xterm-dom-renderer-owner-{id}");
    let rows = format!("{selector} .xterm-rows");
    let cursor = format!("{rows} .xterm-cursor");
    let foreground = colors.foreground.borrow();
    let cursor_color = colors.cursor.borrow();
    let cursor_accent = colors.cursor_accent.borrow();
    let mut output = String::new();
    rule(
        &mut output,
        &rows,
        &format!(" pointer-events: none; color: {};", foreground.css),
    );
    rule(
        &mut output,
        &format!("{rows}, {rows} span"),
        &format!(
            " font-family: {}; font-size: {}px; font-kerning: none; white-space: pre",
            font.family,
            number(font.size)
        ),
    );
    rule(
        &mut output,
        &format!("{rows} .xterm-dim"),
        &format!(" color: {};", multiply_opacity(&foreground, 0.5).css),
    );
    rule(
        &mut output,
        &format!("{selector} span:not(.xterm-bold)"),
        &format!(" font-weight: {};", font.weight),
    );
    rule(
        &mut output,
        &format!("{selector} span.xterm-bold"),
        &format!(" font-weight: {};", font.weight_bold),
    );
    rule(
        &mut output,
        &format!("{selector} span.xterm-italic"),
        " font-style: italic;",
    );
    rule(
        &mut output,
        &format!("{selector} span.xterm-blink-hidden"),
        " visibility: hidden;",
    );
    let underline_animation = format!("blink_underline_{id}");
    let bar_animation = format!("blink_bar_{id}");
    let block_animation = format!("blink_block_{id}");
    output.push_str(&format!(
        "@keyframes {underline_animation} {{ 50% {{  border-bottom-style: hidden; }}}}"
    ));
    output.push_str(&format!(
        "@keyframes {bar_animation} {{ 50% {{  box-shadow: none; }}}}"
    ));
    output.push_str(&format!("@keyframes {block_animation} {{ 0% {{  background-color: {};  color: {}; }} 50% {{  background-color: inherit;  color: {}; }}}}",
        cursor_color.css, cursor_accent.css, cursor_color.css));
    let focused = format!("{rows}.xterm-focus .xterm-cursor.xterm-cursor-blink");
    for (style, animation) in [
        ("underline", underline_animation),
        ("bar", bar_animation),
        ("block", block_animation),
    ] {
        rule(
            &mut output,
            &format!("{focused}.xterm-cursor-{style}"),
            &format!(" animation: {animation} 1s step-end infinite;"),
        );
    }
    rule(
        &mut output,
        &format!("{rows}.xterm-cursor-blink-idle .xterm-cursor.xterm-cursor-blink"),
        " animation: none !important;",
    );
    rule(
        &mut output,
        &format!("{cursor}.xterm-cursor-block"),
        &format!(
            " background-color: {}; color: {};",
            cursor_color.css, cursor_accent.css
        ),
    );
    rule(
        &mut output,
        &format!("{cursor}.xterm-cursor-block:not(.xterm-cursor-blink)"),
        &format!(
            " background-color: {} !important; color: {} !important;",
            cursor_color.css, cursor_accent.css
        ),
    );
    rule(
        &mut output,
        &format!("{cursor}.xterm-cursor-outline"),
        &format!(
            " outline: 1px solid {}; outline-offset: -1px;",
            cursor_color.css
        ),
    );
    rule(
        &mut output,
        &format!("{cursor}.xterm-cursor-bar"),
        &format!(
            " box-shadow: {}px 0 0 {} inset;",
            number(cursor_width),
            cursor_color.css
        ),
    );
    rule(
        &mut output,
        &format!("{cursor}.xterm-cursor-underline"),
        &format!(
            " border-bottom: 1px {}; border-bottom-style: solid; height: calc(100% - 1px);",
            cursor_color.css
        ),
    );
    rule(
        &mut output,
        &format!("{selector} .xterm-selection"),
        " position: absolute; top: 0; left: 0; z-index: 1; pointer-events: none;",
    );
    rule(
        &mut output,
        &format!("{selector}.focus .xterm-selection div"),
        &format!(
            " position: absolute; background-color: {};",
            colors.selection_background_opaque.borrow().css
        ),
    );
    rule(
        &mut output,
        &format!("{selector} .xterm-selection div"),
        &format!(
            " position: absolute; background-color: {};",
            colors.selection_inactive_background_opaque.borrow().css
        ),
    );
    for (index, entry) in colors.ansi.iter().enumerate() {
        let entry = entry.borrow();
        output.push_str(&format!(
            "{selector} .xterm-fg-{index} {{ color: {}; }}",
            entry.css
        ));
        output.push_str(&format!(
            "{selector} .xterm-fg-{index}.xterm-dim {{ color: {}; }}",
            multiply_opacity(&entry, 0.5).css
        ));
        output.push_str(&format!(
            "{selector} .xterm-bg-{index} {{ background-color: {}; }}",
            entry.css
        ));
    }
    let inverted = opaque(&colors.background.borrow());
    output.push_str(&format!(
        "{selector} .xterm-fg-{INVERTED_DEFAULT_COLOR} {{ color: {}; }}",
        inverted.css
    ));
    output.push_str(&format!(
        "{selector} .xterm-fg-{INVERTED_DEFAULT_COLOR}.xterm-dim {{ color: {}; }}",
        multiply_opacity(&inverted, 0.5).css
    ));
    output.push_str(&format!(
        "{selector} .xterm-bg-{INVERTED_DEFAULT_COLOR} {{ background-color: {}; }}",
        foreground.css
    ));
    output
}

fn rule(output: &mut String, selector: &str, declarations: &str) {
    output.push_str(selector);
    output.push_str(" {");
    output.push_str(declarations);
    output.push('}');
}

fn number(value: f64) -> String {
    ryu_js::Buffer::new().format(value).to_owned()
}
