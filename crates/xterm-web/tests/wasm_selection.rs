#![cfg(target_arch = "wasm32")]
// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
#[path = "support/selection_dom.rs"]
mod support;
use js_sys::Promise;
use support::{frame, Setup};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::selection::{
    dom::LinkSelection,
    model::{Dimensions, Position},
};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn real_document_drag_redraw_selection_text_and_cleanup() {
    let setup = Setup::new();
    let down = setup.dispatch("mousedown", 0, 0, 1, false, false);
    assert!(down.default_prevented());
    setup.dispatch("mousemove", 4, 0, 0, false, false);
    setup.dispatch("mouseup", 4, 0, 0, false, false);
    frame().await;
    assert_eq!(setup.selection.selection_start(), Some(Position::new(0, 0)));
    assert_eq!(
        setup.selection.selection_end(setup.dimensions()).unwrap(),
        Some(Position::new(4, 0))
    );
    assert_eq!(setup.host.changes.get(), 1);
    assert_eq!(
        setup.host.redraws.borrow().len(),
        1,
        "one frame coalesces drag updates"
    );
    assert_eq!(
        setup.host.linux.borrow().last().unwrap(),
        &"hell".encode_utf16().collect::<Vec<_>>()
    );
    setup.bindings.dispose().unwrap();
    setup.bindings.dispose().unwrap();
    setup.dispatch("mousedown", 8, 1, 1, false, false);
    setup.dispatch("mousemove", 9, 1, 0, false, false);
    assert_eq!(setup.selection.selection_start(), Some(Position::new(0, 0)));
}
#[wasm_bindgen_test(async)]
async fn word_link_priority_user_input_trim_and_buffer_activation() {
    let setup = Setup::new();
    setup.dispatch("mousedown", 1, 0, 2, false, false);
    setup.dispatch("mouseup", 1, 0, 0, false, false);
    assert_eq!(setup.selection.model().selection_start_length, 5);
    setup.host.user_input.fire(&()).unwrap();
    assert!(!setup.selection.has_selection(setup.dimensions()).unwrap());
    setup.host.link.set(Some(LinkSelection {
        start: Position::new(6, 0),
        length: 4,
    }));
    setup.dispatch("mousedown", 0, 0, 2, false, false);
    assert_eq!(setup.selection.selection_start(), Some(Position::new(6, 0)));
    let active = setup.host.buffers.buffer();
    active.borrow().lines.on_trim_emitter.fire(&1).unwrap();
    assert_eq!(setup.selection.selection_start(), Some(Position::new(0, 0)));
    setup
        .host
        .buffers
        .buffers
        .activate_alt_buffer(None)
        .unwrap();
    assert_eq!(setup.selection.selection_start(), None);
    frame().await;
}
#[wasm_bindgen_test(async)]
async fn right_click_inside_preserves_range_and_outside_selects_word() {
    let setup = Setup::new();
    setup.dispatch("mousedown", 1, 0, 2, false, false);
    setup.dispatch("mouseup", 1, 0, 0, false, false);
    let before = setup.host.changes.get();
    let inside = setup
        .event("contextmenu", 2, 0, 1, false, false)
        .dyn_into::<web_sys::MouseEvent>()
        .unwrap();
    setup.bindings.right_click_select(&inside).unwrap();
    assert_eq!(setup.host.changes.get(), before);
    let outside = setup
        .event("contextmenu", 7, 0, 1, false, false)
        .dyn_into::<web_sys::MouseEvent>()
        .unwrap();
    setup.bindings.right_click_select(&outside).unwrap();
    assert_eq!(setup.selection.selection_start(), Some(Position::new(6, 0)));
    assert_eq!(
        setup.selection.selection_end(setup.dimensions()).unwrap(),
        Some(Position::new(10, 0))
    );
    assert_eq!(setup.host.changes.get(), before + 1);
    frame().await;
}

#[wasm_bindgen_test(async)]
async fn shift_click_extends_from_existing_start() {
    let setup = Setup::new();
    setup.dispatch("mousedown", 0, 0, 1, false, false);
    setup.dispatch("mousemove", 2, 0, 0, false, false);
    setup.dispatch("mouseup", 2, 0, 0, false, false);
    setup.dispatch("mousedown", 4, 0, 1, false, true);
    setup.dispatch("mouseup", 4, 0, 0, false, true);
    assert_eq!(setup.selection.selection_start(), Some(Position::new(0, 0)));
    assert_eq!(
        setup.selection.selection_end(setup.dimensions()).unwrap(),
        Some(Position::new(4, 0))
    );
    frame().await;
    assert_eq!(
        setup.host.redraws.borrow().last().unwrap().end,
        Some(Position::new(4, 0))
    );
}

#[wasm_bindgen_test(async)]
async fn column_drag_extracts_rectangular_rows() {
    let setup = Setup::new();
    setup.dispatch("mousedown", 1, 0, 1, true, false);
    setup.dispatch("mousemove", 3, 1, 0, true, false);
    setup.dispatch("mouseup", 3, 1, 0, true, false);
    let active = setup.host.buffers.buffer();
    let text = setup
        .selection
        .selection_text(&active.borrow(), setup.dimensions())
        .unwrap();
    assert_eq!(text, "el\nel".encode_utf16().collect::<Vec<_>>());
    frame().await;
    assert!(
        setup
            .host
            .redraws
            .borrow()
            .last()
            .unwrap()
            .column_select_mode
    );
}

#[wasm_bindgen_test(async)]
async fn triple_click_selects_entire_wrapped_logical_line() {
    let setup = Setup::new();
    setup
        .host
        .buffers
        .buffer()
        .borrow()
        .set_wrapped(1, true)
        .unwrap();
    setup.dispatch("mousedown", 3, 1, 3, false, false);
    setup.dispatch("mouseup", 3, 1, 0, false, false);
    assert_eq!(setup.selection.selection_start(), Some(Position::new(0, 0)));
    assert_eq!(
        setup.selection.selection_end(setup.dimensions()).unwrap(),
        Some(Position::new(10, 1))
    );
    frame().await;
}

#[wasm_bindgen_test(async)]
async fn drag_interval_scrolls_live_buffer_before_updating_endpoint() {
    let setup = Setup::new();
    {
        let active = setup.host.buffers.buffer();
        let mut buffer = active.borrow_mut();
        for _ in 0..3 {
            let line = buffer.get_blank_line(&Default::default(), false);
            buffer.lines.push(line).unwrap();
        }
        buffer.ybase = 3;
        buffer.ydisp = 1;
    }
    let scrolled = Promise::new(&mut |resolve, _| {
        *setup.host.next_scroll.borrow_mut() = Some(resolve);
    });
    setup.dispatch("mousedown", 1, 0, 1, false, false);
    setup.dispatch("mousemove", 1, 5, 0, false, false);
    JsFuture::from(scrolled).await.unwrap();
    assert_eq!(setup.host.buffers.buffer().borrow().ydisp, 3);
    let dimensions = Dimensions {
        scrollback: 3,
        ..setup.dimensions()
    };
    assert_eq!(
        setup.selection.selection_end(dimensions).unwrap(),
        Some(Position::new(10, 4))
    );
    setup.dispatch("mouseup", 1, 5, 0, false, false);
    frame().await;
}

#[wasm_bindgen_test]
fn forced_mouse_reporting_column_mode_and_alt_cursor_callback() {
    let setup = Setup::new();
    setup
        .selection
        .disable(&setup.host.buffers, setup.dimensions())
        .unwrap();
    setup.host.options.borrow_mut().mouse.mouse_events_active = true;
    setup.dispatch("mousedown", 1, 0, 1, false, false);
    assert_eq!(setup.selection.selection_start(), None);
    setup.dispatch("mousedown", 1, 0, 1, false, true);
    assert_eq!(setup.selection.selection_start(), Some(Position::new(1, 0)));
    setup.dispatch("mouseup", 1, 0, 0, false, true);
    setup.selection.enable();
    setup.dispatch("mousedown", 1, 0, 1, true, false);
    assert_eq!(
        setup.selection.mode(),
        xterm_web::selection::service::SelectionMode::Column
    );
    setup.dispatch("mouseup", 1, 0, 0, true, false);
    assert_eq!(
        *setup.host.cursor_moves.borrow(),
        [(Position::new(1, 0), true)]
    );
}
