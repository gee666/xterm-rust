// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
#[path = "support/terminal_snapshot.rs"]
mod projection;
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{marker::Marker, AttributeData},
    options::OptionValue,
    terminal::CoreTerminal,
};

#[test]
fn erase_protection_markers_and_both_buffers_match_full_source_traces() {
    let mut current = None;
    let mut terminal = CoreTerminal::new(8, 4, 6).unwrap();
    let mut markers = Vec::<Marker>::new();
    let log = Rc::new(RefCell::new(Vec::<String>::new()));
    let mut snapshots = 0;
    for (row, line) in include_str!("fixtures/csi-erase.tsv").lines().enumerate() {
        let columns: Vec<_> = line.split('\t').collect();
        assert_eq!(columns.len(), 7);
        let trace: usize = columns[0].parse().unwrap();
        if current != Some(trace) {
            terminal = CoreTerminal::new(8, 4, 6).unwrap();
            markers.clear();
            current = Some(trace);
            let target = Rc::clone(&log);
            terminal.set_event_handler(Some(Rc::new(move |event| {
                // Live projection preserves marker-disposal/event interleaving.
                if let Some(value) = projection::event(&event) {
                    target.borrow_mut().push(value);
                }
                Ok(())
            })));
        }
        log.borrow_mut().clear();
        let kind = columns[1];
        let argument = columns[2];
        match kind {
            "W" => terminal
                .write_utf16(&projection::parse_hex(argument))
                .unwrap(),
            "C" => {
                let position: Vec<usize> =
                    argument.split(',').map(|s| s.parse().unwrap()).collect();
                let active = terminal.state().active_buffer();
                let mut b = active.borrow_mut();
                b.x = position[0];
                b.y = position[1];
            }
            "O" => terminal
                .state()
                .options
                .set("scrollOnEraseInDisplay", OptionValue::Bool(argument == "1"))
                .unwrap(),
            "S" => terminal
                .state()
                .buffer_service
                .is_user_scrolling
                .set(argument == "1"),
            "M" => {
                let active = terminal.state().active_buffer();
                let marker = active.borrow_mut().add_marker(argument.parse().unwrap());
                let label = markers.len();
                let target = Rc::clone(&log);
                let weak = marker.downgrade();
                marker.on_dispose().subscribe(move |_| {
                    target
                        .borrow_mut()
                        .push(format!("M{label}:{}", weak.upgrade().unwrap().line()));
                });
                markers.push(marker);
            }
            "A" => {
                let active = terminal.state().active_buffer();
                let mut b = active.borrow_mut();
                let line = b.get_blank_line(&AttributeData::default(), false);
                b.lines.push(line).unwrap();
                b.ybase = 1;
            }
            _ => panic!("unknown fixture operation {kind}"),
        }
        let actual = projection::snapshot(&terminal);
        assert_eq!(
            actual, columns[3],
            "seed0xe2a5e111 row{row} trace{trace} op{kind}:{argument}"
        );
        assert_eq!(
            u8::from(terminal.state().buffer_service.is_user_scrolling.get()).to_string(),
            columns[4],
            "user scroll row{row}"
        );
        let actual = markers
            .iter()
            .enumerate()
            .map(|(i, m)| format!("{i}:{}:{}", m.line(), u8::from(m.is_disposed())))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(actual, columns[5], "markers row{row} trace{trace}");
        assert_eq!(
            log.borrow().join("/"),
            columns[6],
            "ordered events row{row} trace{trace} {kind}:{argument}"
        );
        // A live event handler consumes the core queue. The shared projection
        // has already compared every delivered event above, including markers.
        assert!(
            projection::events(&mut terminal).is_empty(),
            "already delivered events must not replay at row{row}"
        );
        snapshots += 1;
    }
    assert_eq!(snapshots, 3937);
    assert_eq!(current, Some(336));
}
