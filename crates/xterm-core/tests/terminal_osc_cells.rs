// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
use std::{any::Any, rc::Rc};
use xterm_core::{
    buffer::{constants::attributes, CellData},
    terminal::CoreTerminal,
};
fn cell(terminal: &CoreTerminal, row: usize, column: usize) -> CellData {
    let active = terminal.state().active_buffer();
    let buffer = active.borrow();
    let line = buffer.lines.get(buffer.ybase + row).unwrap();
    let result = line.borrow().get_cell(column).unwrap();
    result
}

#[test]
fn utf8_byte_chunking_reaches_the_same_title_and_link_handlers() {
    let input = "\x1b]0;Aé😀\x07\x1b]8;id=é;uri/😀\x1b\\X";
    for split in 0..=input.len() {
        let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
        terminal.write_bytes(&input.as_bytes()[..split]).unwrap();
        terminal.write_bytes(&input.as_bytes()[split..]).unwrap();
        assert_eq!(
            terminal.state().title,
            "Aé😀".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            terminal.state().icon_name,
            "Aé😀".encode_utf16().collect::<Vec<_>>()
        );
        let id = terminal.state().attributes.extended.url_id();
        let data = terminal.state().links.get_link_data(id).unwrap();
        assert_eq!(data.borrow().id, Some("é".encode_utf16().collect()));
        assert_eq!(
            data.borrow().uri,
            "uri/😀".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(cell(&terminal, 0, 0).extended.url_id(), id);
    }
}

#[test]
fn hyperlink_ids_and_uris_keep_lone_surrogates_through_the_real_parser() {
    let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
    let mut input: Vec<u16> = "\x1b]8;id=".encode_utf16().collect();
    input.extend([0xd800, 65, u16::from(b';'), 0xdc00, u16::from(b'u'), 7]);
    terminal.write_utf16(&input).unwrap();
    let id = terminal.state().attributes.extended.url_id();
    let data = terminal.state().links.get_link_data(id).unwrap();
    assert_eq!(data.borrow().id, Some(vec![0xd800, 65]));
    assert_eq!(data.borrow().uri, vec![0xdc00, u16::from(b'u')]);
    terminal.write("X").unwrap();
    assert_eq!(cell(&terminal, 0, 0).extended.url_id(), id);
}

#[test]
fn printing_shares_old_payload_while_osc_clone_drops_only_the_new_payload() {
    let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
    let payload: Rc<dyn Any> = Rc::new(17_u32);
    {
        let state = terminal.state_handle();
        let mut state = state.borrow_mut();
        state
            .attributes
            .extended
            .set_payload(Some(Rc::clone(&payload)));
        state
            .attributes
            .extended
            .set_underline_color(attributes::CM_RGB | 0xabcdef);
        state.attributes.update_extended();
    }
    terminal.write("P").unwrap();
    let prior = cell(&terminal, 0, 0);
    assert!(Rc::ptr_eq(&prior.extended.payload().unwrap(), &payload));
    terminal.write("\x1b]8;;uri\x07A\x1b]8;;\x07B").unwrap();
    assert!(terminal.state().attributes.extended.payload().is_none());
    assert_eq!(
        terminal.state().attributes.extended.underline_color(),
        attributes::CM_RGB | 0xabcdef
    );
    assert!(Rc::ptr_eq(
        &cell(&terminal, 0, 0).extended.payload().unwrap(),
        &payload
    ));
    assert_eq!(cell(&terminal, 0, 0).extended.url_id(), 0);
    assert_ne!(cell(&terminal, 0, 1).extended.url_id(), 0);
    assert!(cell(&terminal, 0, 1).extended.payload().is_none());
    assert_eq!(cell(&terminal, 0, 2).extended.url_id(), 0);
}

#[test]
fn the_engine_cell_carrier_does_not_truncate_valid_js_link_ids() {
    let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
    let id = 1_u64 << 53;
    {
        let state = terminal.state_handle();
        let mut state = state.borrow_mut();
        state.attributes.extended.set_url_id(id);
        state.attributes.update_extended();
    }
    terminal.write("X").unwrap();
    assert_eq!(cell(&terminal, 0, 0).extended.url_id(), id);
}

#[test]
fn wide_combining_cells_retain_the_same_link_id_and_utf16_text() {
    let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
    terminal.write("\x1b]8;;uri\x07好\u{301}").unwrap();
    let id = terminal.state().attributes.extended.url_id();
    let leading = cell(&terminal, 0, 0);
    assert_eq!(leading.get_width(), 2);
    assert_eq!(
        leading.get_chars(),
        "好\u{301}".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(leading.extended.url_id(), id);
    assert_eq!(cell(&terminal, 0, 1).extended.url_id(), id);
}

#[test]
fn closing_a_link_in_alt_buffer_does_not_reopen_it_on_cursor_restore() {
    let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
    terminal.write("\x1b]8;;normal\x07A").unwrap();
    let id = terminal.state().attributes.extended.url_id();
    terminal
        .write("\x1b[?1049h\x1b]8;;\x07B\x1b[?1049l")
        .unwrap();
    assert_eq!(terminal.state().attributes.extended.url_id(), 0);
    assert_eq!(cell(&terminal, 0, 0).extended.url_id(), id);
    assert!(terminal.state().links.get_link_data(id).is_some());
}
