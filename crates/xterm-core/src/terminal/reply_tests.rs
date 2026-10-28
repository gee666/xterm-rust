// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
//! Faithful InputHandler window, version, cursor-style and DECSTR assertions.
use super::{CoreTerminal, TerminalEvent};
use crate::{
    core_service::CursorStyle,
    options::OptionValue as V,
    parser::params::{ParamValue, Params},
};
fn terminal(cols: usize, rows: usize) -> CoreTerminal {
    CoreTerminal::new(cols, rows, 1000).unwrap()
}
fn write(t: &mut CoreTerminal, text: &str) {
    t.write(text).unwrap();
}
fn permit(t: &CoreTerminal, keys: &[&str]) {
    t.state()
        .options
        .set(
            "windowOptions",
            V::property_map(keys.iter().map(|key| (*key, V::Bool(true)))),
        )
        .unwrap();
}
fn replies(t: &mut CoreTerminal) -> Vec<String> {
    t.drain_events()
        .into_iter()
        .filter_map(|event| {
            if let TerminalEvent::Data(data) = event {
                Some(String::from_utf16_lossy(&data))
            } else {
                None
            }
        })
        .collect()
}
fn titles(t: &mut CoreTerminal) -> Vec<String> {
    t.drain_events()
        .into_iter()
        .filter_map(|event| {
            if let TerminalEvent::TitleChange(data) = event {
                Some(String::from_utf16_lossy(&data))
            } else {
                None
            }
        })
        .collect()
}
fn strings(values: &[Vec<u16>]) -> Vec<String> {
    values
        .iter()
        .map(|value| String::from_utf16_lossy(value))
        .collect()
}
#[test]
fn source_save_restore_origin() {
    let mut t = terminal(80, 30);
    assert!(!t.state().core.dec_private_modes.borrow().origin);
    write(&mut t, "\x1b[?6h");
    assert!(t.state().core.dec_private_modes.borrow().origin);
    write(&mut t, "\x1b7\x1b[?6l");
    assert!(!t.state().core.dec_private_modes.borrow().origin);
    write(&mut t, "\x1b8");
    assert!(t.state().core.dec_private_modes.borrow().origin);
}
#[test]
fn source_save_restore_wrap() {
    let mut t = terminal(80, 30);
    assert!(t.state().core.dec_private_modes.borrow().wraparound);
    write(&mut t, "\x1b[?7l");
    assert!(!t.state().core.dec_private_modes.borrow().wraparound);
    write(&mut t, "\x1b7\x1b[?7h");
    assert!(t.state().core.dec_private_modes.borrow().wraparound);
    write(&mut t, "\x1b8");
    assert!(!t.state().core.dec_private_modes.borrow().wraparound);
}
#[test]
fn source_cursor_style_parameters() {
    let t = terminal(80, 30);
    for (parameter, style, blink) in [
        (0, None, None),
        (1, Some(CursorStyle::Block), Some(true)),
        (2, Some(CursorStyle::Block), Some(false)),
        (3, Some(CursorStyle::Underline), Some(true)),
        (4, Some(CursorStyle::Underline), Some(false)),
        (5, Some(CursorStyle::Bar), Some(true)),
        (6, Some(CursorStyle::Bar), Some(false)),
    ] {
        t.state
            .borrow_mut()
            .cursor_style(&Params::from_array(&[ParamValue::Param(parameter)]).unwrap())
            .unwrap();
        let state = t.state();
        let modes = state.core.dec_private_modes.borrow();
        assert_eq!(modes.cursor_style, style);
        assert_eq!(modes.cursor_blink, blink);
    }
}
#[test]
fn source_window_defaults_deny_reports() {
    let mut t = terminal(10, 10);
    for sequence in ["\x1b[14t", "\x1b[16t", "\x1b[18t", "\x1b[20t", "\x1b[21t"] {
        write(&mut t, sequence);
    }
    assert!(replies(&mut t).is_empty());
}
#[test]
fn source_window_pixel_size_has_no_headless_report() {
    let mut t = terminal(10, 10);
    permit(&t, &["getWinSizePixels"]);
    write(&mut t, "\x1b[14t");
    assert!(replies(&mut t).is_empty());
}
#[test]
fn source_window_cell_size_has_no_headless_report() {
    let mut t = terminal(10, 10);
    permit(&t, &["getCellSizePixels"]);
    write(&mut t, "\x1b[16t");
    assert!(replies(&mut t).is_empty());
}
#[test]
fn source_window_character_size_reports_live_dimensions() {
    let mut t = terminal(10, 10);
    permit(&t, &["getWinSizeChars"]);
    write(&mut t, "\x1b[18t");
    let mut data = replies(&mut t);
    assert_eq!(data, ["\x1b[8;10;10t"]);
    t.resize(50, 20).unwrap();
    write(&mut t, "\x1b[18t");
    data.extend(replies(&mut t));
    assert_eq!(data, ["\x1b[8;10;10t", "\x1b[8;20;50t"]);
}
fn push_pop(selector: &str) {
    let mut t = terminal(10, 10);
    permit(&t, &["pushTitle", "popTitle"]);
    for value in [1, 2, 3] {
        write(&mut t, &format!("\x1b]0;{value}\x07\x1b[22{selector}t"));
    }
    {
        let s = t.state();
        assert_eq!(
            strings(&s.title_stack),
            if selector == ";1" {
                vec![]
            } else {
                vec!["1", "2", "3"]
            }
        );
        assert_eq!(
            strings(&s.icon_stack),
            if selector == ";2" {
                vec![]
            } else {
                vec!["1", "2", "3"]
            }
        );
    }
    let mut changes = titles(&mut t);
    assert_eq!(changes, ["1", "2", "3"]);
    for _ in 0..4 {
        write(&mut t, &format!("\x1b[23{selector}t"));
    }
    {
        let s = t.state();
        assert!(s.title_stack.is_empty());
        assert!(s.icon_stack.is_empty());
    }
    changes.extend(titles(&mut t));
    assert_eq!(
        changes,
        if selector == ";1" {
            vec!["1", "2", "3"]
        } else {
            vec!["1", "2", "3", "3", "2", "1"]
        }
    );
}
#[test]
fn source_window_push_pop_both() {
    push_pop("");
}
#[test]
fn source_window_push_pop_icon() {
    push_pop(";1");
}
#[test]
fn source_window_push_pop_title() {
    push_pop(";2");
}
#[test]
fn source_deccolm_permission() {
    let mut t = terminal(10, 10);
    write(&mut t, "\x1b[?3l");
    assert_eq!(t.cols(), 10);
    write(&mut t, "\x1b[?3h");
    assert_eq!(t.cols(), 10);
    permit(&t, &["setWinLines"]);
    write(&mut t, "\x1b[?3l");
    assert_eq!(t.cols(), 80);
    write(&mut t, "\x1b[?3h");
    assert_eq!(t.cols(), 132);
}
#[test]
fn source_xtversion_omitted() {
    let mut t = terminal(80, 30);
    write(&mut t, "\x1b[>q");
    assert_eq!(replies(&mut t), ["\x1bP>|xterm.js(6.0.0)\x1b\\"]);
}
#[test]
fn source_xtversion_zero() {
    let mut t = terminal(80, 30);
    write(&mut t, "\x1b[>0q");
    assert_eq!(replies(&mut t), ["\x1bP>|xterm.js(6.0.0)\x1b\\"]);
}
#[test]
fn source_xtversion_nonzero() {
    let mut t = terminal(80, 30);
    write(&mut t, "\x1b[>1q");
    assert!(replies(&mut t).is_empty());
}
#[path = "reply_regressions.rs"]
mod regressions;
fn soft() -> CoreTerminal {
    let mut t = CoreTerminal::new(10, 5, 1).unwrap();
    write(&mut t, "01234567890123");
    t
}
#[test]
fn source_decstr_insert() {
    let mut t = soft();
    write(&mut t, "\x1b[4h");
    assert!(t.state().core.modes.borrow().insert_mode);
    write(&mut t, "\x1b[!p");
    assert!(!t.state().core.modes.borrow().insert_mode);
}
#[test]
fn source_decstr_cursor_visibility() {
    let mut t = soft();
    write(&mut t, "\x1b[?25l");
    assert!(t.state().core.is_cursor_hidden.get());
    write(&mut t, "\x1b[!p");
    assert!(!t.state().core.is_cursor_hidden.get());
}
#[test]
fn source_decstr_margins() {
    let mut t = soft();
    write(&mut t, "\x1b[2;4r");
    {
        let a = t.state().active_buffer();
        let b = a.borrow();
        assert_eq!((b.scroll_top, b.scroll_bottom), (1, 3));
    }
    write(&mut t, "\x1b[!p");
    let a = t.state().active_buffer();
    let b = a.borrow();
    assert_eq!((b.scroll_top, b.scroll_bottom), (0, t.rows() - 1));
}
#[test]
fn source_decstr_attributes() {
    let mut t = soft();
    write(&mut t, "\x1b[1;2;32;43m");
    assert_ne!(t.state().attributes.is_bold(), 0);
    write(&mut t, "\x1b[!p");
    assert_eq!(t.state().attributes.is_bold(), 0);
    assert_eq!(t.state().attributes.fg, 0);
    assert_eq!(t.state().attributes.bg, 0);
}
#[test]
fn source_decstr_saved_cursor() {
    let mut t = soft();
    write(&mut t, "\x1b7");
    {
        let a = t.state().active_buffer();
        let b = a.borrow();
        assert_eq!((b.saved_x, b.saved_y), (4, 1));
    }
    write(&mut t, "\x1b[!p");
    let a = t.state().active_buffer();
    let b = a.borrow();
    assert_eq!((b.saved_x, b.saved_y), (0, 0));
}
#[test]
fn source_decstr_origin() {
    let mut t = soft();
    write(&mut t, "\x1b[?6h");
    assert!(t.state().core.dec_private_modes.borrow().origin);
    write(&mut t, "\x1b[!p");
    assert!(!t.state().core.dec_private_modes.borrow().origin);
}
