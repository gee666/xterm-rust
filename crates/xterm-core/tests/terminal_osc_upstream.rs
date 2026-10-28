// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// All 12 OSC cases from InputHandler.test.ts, preserving every expected value.
use xterm_core::terminal::{
    ColorIndex as Index, ColorRequest as Request, CoreTerminal, TerminalEvent,
};

fn terminal() -> CoreTerminal {
    CoreTerminal::new(80, 30, 1000).unwrap()
}
fn colors(terminal: &mut CoreTerminal, data: &str) -> Vec<Vec<Request>> {
    terminal
        .write_utf16(&data.encode_utf16().collect::<Vec<_>>())
        .unwrap();
    terminal
        .drain_events()
        .into_iter()
        .filter_map(|event| match event {
            TerminalEvent::Color(batch) => Some(batch),
            _ => None,
        })
        .collect()
}
fn report(index: u8) -> Request {
    Request::Report {
        index: Index::Indexed(index),
    }
}
fn set(index: u8, color: [u8; 3]) -> Request {
    Request::Set {
        index: Index::Indexed(index),
        color,
    }
}
fn restore(index: u8) -> Request {
    Request::Restore {
        index: Some(Index::Indexed(index)),
    }
}

#[test]
fn upstream_4_query_color_events() {
    let mut terminal = terminal();
    assert_eq!(
        colors(&mut terminal, "\x1b]4;0;?\x07"),
        vec![vec![report(0)]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]4;123;?\x07"),
        vec![vec![report(123)]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]4;0;?;123;?\x07"),
        vec![vec![report(0), report(123)]]
    );
}
#[test]
fn upstream_4_set_color_events() {
    let mut terminal = terminal();
    assert_eq!(
        colors(&mut terminal, "\x1b]4;0;rgb:01/02/03\x07"),
        vec![vec![set(0, [1, 2, 3])]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]4;123;#aabbcc\x07"),
        vec![vec![set(123, [170, 187, 204])]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]4;0;rgb:aa/bb/cc;123;#001122\x07"),
        vec![vec![set(0, [170, 187, 204]), set(123, [0, 17, 34])]]
    );
}
#[test]
fn upstream_4_ignores_invalid_values() {
    let mut terminal = terminal();
    assert_eq!(
        colors(
            &mut terminal,
            "\x1b]4;0;rgb:aa/bb/cc;45;rgb:1/22/333;123;#001122\x07"
        ),
        vec![vec![set(0, [170, 187, 204]), set(123, [0, 17, 34])]]
    );
}
#[test]
fn upstream_8_hyperlink_with_id() {
    let mut terminal = terminal();
    colors(&mut terminal, "\x1b]8;id=100;http://localhost:3000\x07");
    let id = terminal.state().attributes.extended.url_id();
    assert_ne!(id, 0);
    let data = terminal.state().links.get_link_data(id).unwrap();
    assert_eq!(data.borrow().id, Some("100".encode_utf16().collect()));
    assert_eq!(
        data.borrow().uri,
        "http://localhost:3000".encode_utf16().collect::<Vec<_>>()
    );
    colors(&mut terminal, "\x1b]8;;\x07");
    assert_eq!(terminal.state().attributes.extended.url_id(), 0);
}
#[test]
fn upstream_8_hyperlink_with_semicolon() {
    let mut terminal = terminal();
    colors(&mut terminal, "\x1b]8;;http://localhost:3000;abc=def\x07");
    let id = terminal.state().attributes.extended.url_id();
    assert_ne!(id, 0);
    let data = terminal.state().links.get_link_data(id).unwrap();
    assert_eq!(data.borrow().id, None);
    assert_eq!(
        data.borrow().uri,
        "http://localhost:3000;abc=def"
            .encode_utf16()
            .collect::<Vec<_>>()
    );
    colors(&mut terminal, "\x1b]8;;\x07");
    assert_eq!(terminal.state().attributes.extended.url_id(), 0);
}
#[test]
fn upstream_104_restore_events() {
    let mut terminal = terminal();
    assert_eq!(
        colors(&mut terminal, "\x1b]104;0\x07\x1b]104;43\x07"),
        vec![vec![restore(0)], vec![restore(43)]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]104;0;43\x07"),
        vec![vec![restore(0), restore(43)]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]104\x07"),
        vec![vec![Request::Restore { index: None }]]
    );
}

#[test]
fn upstream_10_foreground_set_and_query() {
    let mut terminal = terminal();
    assert_eq!(
        colors(&mut terminal, "\x1b]10;?\x07"),
        vec![vec![Request::Report {
            index: Index::Foreground
        }]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]10;?;?;?;?\x07"),
        vec![
            vec![Request::Report {
                index: Index::Foreground
            }],
            vec![Request::Report {
                index: Index::Background
            }],
            vec![Request::Report {
                index: Index::Cursor
            }]
        ]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]10;rgb:01/02/03\x07"),
        vec![vec![Request::Set {
            index: Index::Foreground,
            color: [1, 2, 3]
        }]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]10;#aabbcc\x07"),
        vec![vec![Request::Set {
            index: Index::Foreground,
            color: [170, 187, 204]
        }]]
    );
    assert_eq!(
        colors(
            &mut terminal,
            "\x1b]10;rgb:aa/bb/cc;#001122;rgb:12/34/56\x07"
        ),
        vec![
            vec![Request::Set {
                index: Index::Foreground,
                color: [170, 187, 204]
            }],
            vec![Request::Set {
                index: Index::Background,
                color: [0, 17, 34]
            }],
            vec![Request::Set {
                index: Index::Cursor,
                color: [18, 52, 86]
            }]
        ]
    );
}
#[test]
fn upstream_110_restore_foreground() {
    assert_eq!(
        colors(&mut terminal(), "\x1b]110\x07"),
        vec![vec![Request::Restore {
            index: Some(Index::Foreground)
        }]]
    );
}
#[test]
fn upstream_11_background_set_and_query() {
    let mut terminal = terminal();
    assert_eq!(
        colors(&mut terminal, "\x1b]11;?\x07"),
        vec![vec![Request::Report {
            index: Index::Background
        }]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]11;?;?;?;?\x07"),
        vec![
            vec![Request::Report {
                index: Index::Background
            }],
            vec![Request::Report {
                index: Index::Cursor
            }]
        ]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]11;rgb:01/02/03\x07"),
        vec![vec![Request::Set {
            index: Index::Background,
            color: [1, 2, 3]
        }]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]11;#aabbcc\x07"),
        vec![vec![Request::Set {
            index: Index::Background,
            color: [170, 187, 204]
        }]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]11;#001122;rgb:12/34/56\x07"),
        vec![
            vec![Request::Set {
                index: Index::Background,
                color: [0, 17, 34]
            }],
            vec![Request::Set {
                index: Index::Cursor,
                color: [18, 52, 86]
            }]
        ]
    );
}
#[test]
fn upstream_111_restore_background() {
    assert_eq!(
        colors(&mut terminal(), "\x1b]111\x07"),
        vec![vec![Request::Restore {
            index: Some(Index::Background)
        }]]
    );
}
#[test]
fn upstream_12_cursor_set_and_query() {
    let mut terminal = terminal();
    assert_eq!(
        colors(&mut terminal, "\x1b]12;?\x07"),
        vec![vec![Request::Report {
            index: Index::Cursor
        }]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]12;?;?;?;?\x07"),
        vec![vec![Request::Report {
            index: Index::Cursor
        }]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]12;rgb:01/02/03\x07"),
        vec![vec![Request::Set {
            index: Index::Cursor,
            color: [1, 2, 3]
        }]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]12;#aabbcc\x07"),
        vec![vec![Request::Set {
            index: Index::Cursor,
            color: [170, 187, 204]
        }]]
    );
}
#[test]
fn upstream_112_restore_cursor() {
    assert_eq!(
        colors(&mut terminal(), "\x1b]112\x07"),
        vec![vec![Request::Restore {
            index: Some(Index::Cursor)
        }]]
    );
}
