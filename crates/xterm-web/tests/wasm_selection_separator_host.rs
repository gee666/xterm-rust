#![cfg(target_arch = "wasm32")]
// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
#[path = "support/selection_dom.rs"]
mod support;
use support::{frame, Setup};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_web::selection::{dom::LinkSelection, model::Position};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn non_word_actions_and_link_priority_never_request_the_separator() {
    let setup = Setup::new();
    *setup.host.word_separator.borrow_mut() = Err(JsValue::from_str("unused separator"));
    setup.dispatch("mousedown", 1, 0, 1, false, false);
    setup.dispatch("mousemove", 4, 0, 0, false, false);
    setup.dispatch("mouseup", 4, 0, 0, false, false);
    assert_eq!(setup.host.separator_reads.get(), 0);
    assert_eq!(
        setup.selection.selection_end(setup.dimensions()).unwrap(),
        Some(Position::new(4, 0))
    );
    setup.host.link.set(Some(LinkSelection {
        start: Position::new(6, 0),
        length: 4,
    }));
    setup.dispatch("mousedown", 0, 0, 2, false, false);
    setup.dispatch("mouseup", 0, 0, 0, false, false);
    assert_eq!(setup.host.separator_reads.get(), 0);
    assert_eq!(setup.selection.selection_start(), Some(Position::new(6, 0)));
    assert_eq!(setup.selection.model().selection_start_length, 4);
    frame().await;
}

#[wasm_bindgen_test(async)]
async fn ordinary_word_click_and_word_drag_read_current_separators() {
    let setup = Setup::new();
    setup.dispatch("mousedown", 1, 0, 2, false, false);
    assert_eq!(setup.host.separator_reads.get(), 1);
    assert_eq!(setup.selection.model().selection_start_length, 5);
    *setup.host.word_separator.borrow_mut() = Ok(" s".encode_utf16().collect());
    setup.dispatch("mousemove", 7, 0, 0, false, false);
    assert_eq!(setup.host.separator_reads.get(), 2);
    assert_eq!(
        setup.selection.selection_end(setup.dimensions()).unwrap(),
        Some(Position::new(8, 0))
    );
    setup.dispatch("mouseup", 7, 0, 0, false, false);
    assert_eq!(setup.host.separator_reads.get(), 2);
    frame().await;
}
