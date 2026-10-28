use xterm_core::terminal::{CoreTerminal, TerminalEvent};

#[test]
fn reverse_index_refreshes_only_its_scroll_region_in_both_buffers() {
    for alternate in [false, true] {
        let mut terminal = CoreTerminal::new(4, 4, 6).unwrap();
        if alternate {
            terminal.write("\x1b[?1049h").unwrap();
        }
        terminal.write("AAAA\r\nBBBB\r\nCCCC\r\nDDDD\x1b[2;3r\x1b[2;1H").unwrap();
        terminal.drain_events();
        terminal.write("\x1bM").unwrap();
        let renders: Vec<_> = terminal.drain_events().into_iter().filter_map(|event| {
            if let TerminalEvent::Render { start, end } = event { Some((start, end)) } else { None }
        }).collect();
        assert_eq!(renders, [(1, 2)], "alternate={alternate}");
        let state = terminal.state();
        let buffer = state.active_buffer();
        let buffer = buffer.borrow();
        assert_eq!((buffer.x, buffer.y), (0, 1));
        let text: Vec<_> = (0..4).map(|row| {
            buffer.lines.get(row).unwrap().borrow().translate_to_string(true, None, None, None)
        }).collect();
        assert_eq!(text, ["AAAA", "", "BBBB", "DDDD"].map(|text| text.encode_utf16().collect::<Vec<_>>()));
    }
}
