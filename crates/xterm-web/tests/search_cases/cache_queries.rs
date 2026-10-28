// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_core::terminal::TerminalEvent;

#[test]
fn actual_cursor_linefeed_and_resize_invalidation_refreshes_results_not_only_timers() {
    let h = Harness::new(20, 3, None);
    h.write("old");
    assert!(!h.addon.find_next(&term("missing"), None).unwrap());
    let moves = Rc::new(Cell::new(0));
    let capture = Rc::clone(&moves);
    h.host.terminal.native.on_event().subscribe(move |event| {
        if matches!(event, TerminalEvent::CursorMove) {
            capture.set(capture.get() + 1);
        }
    });
    // Restoring the original cursor suppresses parse-end CursorMove, so the
    // linefeed subscription itself must clear previously cached empty row one.
    h.write("\x1b7\r\nnew\x1b8");
    assert_eq!(moves.get(), 0);
    assert!(h.addon.find_next(&term("new"), None).unwrap());
    assert_eq!(h.selected_text(), term("new"));

    let h = Harness::new(20, 3, None);
    h.write("old");
    assert!(!h.addon.find_next(&term("missing"), None).unwrap());
    h.write("\rnew!");
    assert!(h.addon.find_next(&term("new!"), None).unwrap());
    assert_eq!(h.selected_text(), term("new!"));

    let h = Harness::new(8, 3, None);
    // Default reflowCursorLine=false deliberately excludes the cursor's block.
    // Move to a fresh hard line so this earlier block actually reflows.
    h.write("abcdefghij\r\n");
    assert!(!h.addon.find_next(&term("missing"), None).unwrap());
    h.host.terminal.native.resize(5, 3).unwrap();
    assert!(h.addon.find_next(&term("hi"), None).unwrap());
    assert_eq!(
        h.selection().unwrap().start,
        SearchPosition { row: 1, col: 2 }
    );
    assert_eq!(h.selected_text(), term("hi"));
}
