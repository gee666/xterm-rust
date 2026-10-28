// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    task::{Context, Poll, Waker},
};
use xterm_core::{
    event::EmitError,
    parser::{handler::HandlerResult, osc::OscHandler},
    terminal::{
        ColorIndex as Index, ColorRequest as Request, CoreTerminal, TerminalError, TerminalEvent,
    },
};
fn colors(terminal: &mut CoreTerminal, input: &str) -> Vec<Vec<Request>> {
    terminal.write(input).unwrap();
    terminal
        .drain_events()
        .into_iter()
        .filter_map(|event| match event {
            TerminalEvent::Color(batch) => Some(batch),
            _ => None,
        })
        .collect()
}

#[test]
fn indexed_validation_and_pair_alignment_preserve_all_source_edges() {
    let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
    assert_eq!(
        colors(
            &mut terminal,
            "\x1b]4;000;?;255;#fff;256;?;-1;?; 1;?;１２;?;1;bad;2;?;99\x07"
        ),
        vec![vec![
            Request::Report {
                index: Index::Indexed(0)
            },
            Request::Set {
                index: Index::Indexed(255),
                color: [240; 3]
            },
            Request::Report {
                index: Index::Indexed(2)
            }
        ]]
    );
    assert!(colors(
        &mut terminal,
        "\x1b]4;;?;+0;?;999999999999999999999999;?\x07"
    )
    .is_empty());
}
#[test]
fn special_invalid_slots_consume_offset_and_restore_payload_is_ignored() {
    let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
    assert_eq!(
        colors(&mut terminal, "\x1b]10;bad;?;#fff;?\x07"),
        vec![
            vec![Request::Report {
                index: Index::Background
            }],
            vec![Request::Set {
                index: Index::Cursor,
                color: [240; 3]
            }]
        ]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]10;;;?\x07"),
        vec![vec![Request::Report {
            index: Index::Cursor
        }]]
    );
    assert_eq!(
        colors(&mut terminal, "\x1b]104;;0; 1;+2;255;256;0003;１２;3\x07"),
        vec![vec![
            Request::Restore {
                index: Some(Index::Indexed(0))
            },
            Request::Restore {
                index: Some(Index::Indexed(255))
            },
            Request::Restore {
                index: Some(Index::Indexed(3))
            },
            Request::Restore {
                index: Some(Index::Indexed(3))
            }
        ]]
    );
    assert!(colors(&mut terminal, "\x1b]104; \x07").is_empty());
    assert_eq!(
        colors(&mut terminal, "\x1b]110;ignored;?\x07"),
        vec![vec![Request::Restore {
            index: Some(Index::Foreground)
        }]]
    );
}

#[test]
fn title_event_observes_new_title_before_osc0_icon_mutation_and_utf16_is_lossless() {
    let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
    terminal.write("\x1b]1;old\x07").unwrap();
    terminal.drain_events();
    let state = terminal.state_handle();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let log = Rc::clone(&seen);
    terminal.set_event_handler(Some(Rc::new(move |event| {
        if let TerminalEvent::TitleChange(title) = event {
            log.borrow_mut().push((
                title,
                state.borrow().title.clone(),
                state.borrow().icon_name.clone(),
            ));
        }
        Ok(())
    })));
    let data = [27, 93, 48, 59, 0xd800, 88, 0xdc00, 7];
    terminal.write_utf16(&data).unwrap();
    let title = vec![0xd800, 88, 0xdc00];
    assert_eq!(
        &*seen.borrow(),
        &[(title.clone(), title.clone(), "old".encode_utf16().collect())]
    );
    assert_eq!(terminal.state().icon_name, title);
    terminal.write("\x1b]2;\x07\x1b]2;\x07").unwrap();
    assert_eq!(seen.borrow().len(), 3);
}

#[test]
fn listener_failures_stop_later_osc_side_effects_and_preserve_error_identity() {
    let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
    terminal.write("\x1b]1;old\x07").unwrap();
    let original = EmitError::callback(std::io::Error::other("OSC listener sentinel"));
    let error = original.clone();
    terminal.set_event_handler(Some(Rc::new(move |event| {
        if matches!(event, TerminalEvent::TitleChange(_)) {
            Err(error.clone())
        } else {
            Ok(())
        }
    })));
    let returned = terminal.write("\x1b]0;new\x07").unwrap_err();
    assert!(matches!(returned, TerminalError::Event(error) if error == original));
    assert_eq!(
        terminal.state().icon_name,
        "old".encode_utf16().collect::<Vec<_>>()
    );
}

#[test]
fn special_color_callbacks_are_separate_and_failure_stops_later_events() {
    let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
    let count = Rc::new(Cell::new(0));
    let seen = Rc::clone(&count);
    terminal.set_event_handler(Some(Rc::new(move |event| {
        if let TerminalEvent::Color(batch) = event {
            assert_eq!(batch.len(), 1);
            seen.set(seen.get() + 1);
            if seen.get() == 2 {
                return Err(EmitError::ListenerArrayCleared);
            }
        }
        Ok(())
    })));
    assert!(terminal.write("\x1b]10;?;?;?\x07").is_err());
    assert_eq!(count.get(), 2);
}

#[test]
fn every_title_color_and_link_sequence_works_across_each_chunk_split() {
    for input in [
        "\x1b]0;title\x07",
        "\x1b]1;icon\x1b\\",
        "\x1b]2;title\x1b\\",
        "\x1b]4;1;#123\x07",
        "\x1b]8;id=a;uri;a\x1b\\",
        "\x1b]10;?;#fff;?\x07",
        "\x1b]11;?;?\x07",
        "\x1b]12;?\x07",
        "\x1b]104;0;255\x07",
        "\x1b]110\x07",
        "\x1b]111\x07",
        "\x1b]112\x07",
    ] {
        let data: Vec<u16> = input.encode_utf16().collect();
        let mut expected = CoreTerminal::new(4, 3, 2).unwrap();
        let expected_colors = colors(&mut expected, input);
        for split in 0..=data.len() {
            let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
            terminal.write_utf16(&data[..split]).unwrap();
            terminal.write_utf16(&data[split..]).unwrap();
            let actual: Vec<_> = terminal
                .drain_events()
                .into_iter()
                .filter_map(|event| match event {
                    TerminalEvent::Color(batch) => Some(batch),
                    _ => None,
                })
                .collect();
            assert_eq!(actual, expected_colors, "{input:?} split{split}");
            assert_eq!(terminal.state().title, expected.state().title);
            assert_eq!(terminal.state().icon_name, expected.state().icon_name);
            assert_eq!(
                terminal.state().attributes.extended.url_id(),
                expected.state().attributes.extended.url_id()
            );
        }
    }
}

#[test]
fn custom_pending_false_resumes_into_builtin_without_replaying_input() {
    let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
    terminal.parser_mut().osc.register_handler(
        2.0,
        Rc::new(RefCell::new(OscHandler::new(|_| {
            HandlerResult::Pending(Box::pin(async { false }))
        }))),
    );
    assert!(matches!(
        terminal.write("\x1b]2;title\x07A"),
        Err(TerminalError::Pending)
    ));
    assert!(terminal.state().title.is_empty());
    let HandlerResult::Pending(mut pending) = terminal.take_pending().unwrap() else {
        panic!("expected owned pending handler");
    };
    assert_eq!(
        pending
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Ready(false)
    );
    terminal.resume(false).unwrap();
    assert_eq!(
        terminal.state().title,
        "title".encode_utf16().collect::<Vec<_>>()
    );
    let active = terminal.state().active_buffer();
    assert_eq!(active.borrow().x, 1);
}

#[test]
fn hyperlink_params_closing_and_cells_keep_source_identity_rules() {
    let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
    terminal
        .write("\x1b]8;  id=first:id=second  ;uri;semicolon\x07A")
        .unwrap();
    let id = terminal.state().attributes.extended.url_id();
    let data = terminal.state().links.get_link_data(id).unwrap();
    assert_eq!(data.borrow().id, Some("first".encode_utf16().collect()));
    assert_eq!(
        data.borrow().uri,
        "uri;semicolon".encode_utf16().collect::<Vec<_>>()
    );
    terminal
        .write("\x1b]8;nonempty;\x07\x1b]8;malformed\x07")
        .unwrap();
    assert_eq!(terminal.state().attributes.extended.url_id(), id);
    terminal.write("\x1b]8;;\x07B").unwrap();
    let buffer = terminal.state().active_buffer();
    let line = buffer.borrow().lines.get(0).unwrap();
    assert_eq!(line.borrow().get_cell(0).unwrap().extended.url_id(), id);
    assert_eq!(line.borrow().get_cell(1).unwrap().extended.url_id(), 0);
    terminal.write("\x1b]8;id=;new\x07").unwrap();
    let id = terminal.state().attributes.extended.url_id();
    assert!(terminal
        .state()
        .links
        .get_link_data(id)
        .unwrap()
        .borrow()
        .id
        .is_none());
}

#[test]
fn print_tracks_url_before_wrap_and_soft_hyphens_do_not_add_lines() {
    let mut terminal = CoreTerminal::new(4, 3, 0).unwrap();
    terminal.write("\x1b]8;;uri\x07abcdX").unwrap();
    let buffer = terminal.state().active_buffer();
    assert_eq!(buffer.borrow().markers.borrow().len(), 1);
    terminal.write("\u{ad}").unwrap();
    assert_eq!(buffer.borrow().markers.borrow().len(), 1);
    terminal.write("Y").unwrap();
    assert_eq!(buffer.borrow().markers.borrow().len(), 2);
    let id = terminal.state().attributes.extended.url_id();
    terminal.write("\x1b]8;;\x07\r\n\r\n\r\n").unwrap();
    assert!(terminal.state().links.get_link_data(id).is_none());
}
