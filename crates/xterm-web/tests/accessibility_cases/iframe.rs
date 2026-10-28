// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use crate::accessibility_support::{clock::Clock, errors::selection_error, Host};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{HtmlElement, HtmlIFrameElement};
use xterm_web::accessibility::{AccessibilityManager, LocalizableStrings, SelectionRange};

#[wasm_bindgen_test]
fn iframe_rows_and_focus_use_structural_dom_but_keep_source_text_realm_guard() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let clock = Clock::new(&window);
    let frame: HtmlIFrameElement = document.create_element("iframe").unwrap().unchecked_into();
    document.body().unwrap().append_child(&frame).unwrap();
    let owner = frame.content_document().unwrap();
    let style = owner.create_element("style").unwrap();
    style.set_text_content(Some(xterm_web::XTERM_CSS));
    owner.body().unwrap().append_child(&style).unwrap();
    let root: HtmlElement = owner.create_element("div").unwrap().unchecked_into();
    root.set_class_name("xterm");
    owner.body().unwrap().append_child(&root).unwrap();
    let host = Host::new();
    let manager = AccessibilityManager::new(
        window,
        owner.clone(),
        Some(&root),
        host.clone(),
        LocalizableStrings::new(),
    )
    .unwrap();
    assert_eq!(manager.container().owner_document().unwrap(), owner);
    let rows = manager
        .container()
        .query_selector_all("[role=listitem]")
        .unwrap();
    let first: HtmlElement = rows.item(0).unwrap().unchecked_into();
    let adjacent: HtmlElement = rows.item(1).unwrap().unchecked_into();
    // Element endpoints work across realms because the source does not perform
    // an Element instanceof assertion here. Offsets still index its column map.
    owner
        .get_selection()
        .unwrap()
        .unwrap()
        .set_base_and_extent(&first, 0, &first, 1)
        .unwrap();
    owner
        .dispatch_event(&web_sys::Event::new("selectionchange").unwrap())
        .unwrap();
    assert_eq!(
        *host.selections.borrow(),
        [SelectionRange {
            row: 2.0,
            column: 0.0,
            length: 1.0
        }]
    );
    adjacent.focus().unwrap();
    first.focus().unwrap();
    assert_eq!(*host.scrolls.borrow(), [-1]);
    clock.advance(1000.0).unwrap();
    let node = manager
        .container()
        .query_selector("[role=listitem]")
        .unwrap()
        .unwrap()
        .first_child()
        .unwrap();
    owner
        .get_selection()
        .unwrap()
        .unwrap()
        .set_base_and_extent(&node, 0, &node, 1)
        .unwrap();
    let error = selection_error(&owner).unwrap();
    // The original module explicitly uses `node instanceof Text` from its own
    // realm, so iframe Text nodes reach getAttribute and throw TypeError. Do not
    // silently "fix" that source-visible branch with a nodeType substitution.
    assert!(
        error.is_instance_of::<js_sys::TypeError>(),
        "expected source Text realm failure, got {error:?}"
    );
    manager.dispose().unwrap();
    assert_eq!(clock.pending(), 0);
    frame.remove();
}
