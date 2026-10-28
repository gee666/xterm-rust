// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use crate::accessibility_support::{text, units, Fixture};
use js_sys::{Function, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use xterm_web::accessibility::{Row, SelectionRange};

use crate::accessibility_support::errors::selection_error;
#[wasm_bindgen_test]
fn reversed_utf16_selection_maps_astral_and_wide_cells_without_losing_units() {
    let f = Fixture::new();
    let node = f.row(0).first_child().unwrap();
    let selection = f.document.get_selection().unwrap().unwrap();
    selection.set_base_and_extent(&node, 3, &node, 1).unwrap();
    f.selection_change().unwrap();
    assert_eq!(
        *f.host.selections.borrow(),
        [SelectionRange {
            row: 2.0,
            column: 1.0,
            length: 2.0
        }]
    );
    assert_eq!(text(&f.row(0)), units("A😀字"));
}
#[wasm_bindgen_test]
fn collapsed_selection_clears_only_inside_tree_and_missing_selection_is_ignored() {
    let f = Fixture::new();
    let selection = f.document.get_selection().unwrap().unwrap();
    selection
        .collapse_with_offset(Some(&f.row(0).first_child().unwrap()), 0)
        .unwrap();
    f.selection_change().unwrap();
    assert_eq!(f.host.clears.get(), 1);
    selection
        .collapse_with_offset(Some(&f.before.first_child().unwrap()), 0)
        .unwrap();
    f.selection_change().unwrap();
    assert_eq!(f.host.clears.get(), 1);
    selection.remove_all_ranges().unwrap();
    f.selection_change().unwrap();
    assert_eq!(f.host.clears.get(), 1);
    assert!(f.host.selections.borrow().is_empty());
}
#[wasm_bindgen_test]
fn selection_across_outside_nodes_clamps_to_first_and_last_visible_rows() {
    let f = Fixture::new();
    let selection = f.document.get_selection().unwrap().unwrap();
    selection
        .set_base_and_extent(
            &f.after.first_child().unwrap(),
            1,
            &f.before.first_child().unwrap(),
            1,
        )
        .unwrap();
    f.selection_change().unwrap();
    assert_eq!(
        *f.host.selections.borrow(),
        [SelectionRange {
            row: 2.0,
            column: 0.0,
            length: 20.0
        }]
    );
}
#[wasm_bindgen_test]
fn stale_row_and_unmapped_nested_text_do_not_invent_terminal_ranges() {
    let f = Fixture::new();
    let selection = f.document.get_selection().unwrap().unwrap();
    let node = f.row(0).first_child().unwrap();
    f.row(0)
        .set_attribute("aria-posinset", "not a number")
        .unwrap();
    selection.set_base_and_extent(&node, 0, &node, 1).unwrap();
    f.selection_change().unwrap();
    assert!(f.host.selections.borrow().is_empty());
    f.row(0).set_attribute("aria-posinset", "3").unwrap();
    let span = f.document.create_element("span").unwrap();
    span.set_text_content(Some("xy"));
    span.set_attribute("aria-posinset", "3").unwrap();
    f.row(0).append_child(&span).unwrap();
    let text = span.first_child().unwrap();
    selection.set_base_and_extent(&text, 0, &text, 1).unwrap();
    f.selection_change().unwrap();
    assert!(f.host.selections.borrow().is_empty());
}
#[wasm_bindgen_test]
fn offsets_beyond_stale_column_map_use_last_column_plus_one() {
    let f = Fixture::new();
    f.row(0).set_text_content(Some("longer string"));
    let node = f.row(0).first_child().unwrap();
    f.document
        .get_selection()
        .unwrap()
        .unwrap()
        .set_base_and_extent(&node, 0, &node, 6)
        .unwrap();
    f.selection_change().unwrap();
    assert_eq!(
        *f.host.selections.borrow(),
        [SelectionRange {
            row: 2.0,
            column: 0.0,
            length: 6.0
        }]
    );
}
#[wasm_bindgen_test]
fn equal_mapped_eol_range_throws_the_source_error_through_dom_callback() {
    let f = Fixture::new();
    f.host.lines.borrow_mut()[2] = Row {
        text: units("12345678"),
        columns: (0..=8).collect(),
    };
    f.render();
    let start = f.row(0).first_child().unwrap();
    let end = f.row(1).first_child().unwrap();
    let selection = f.document.get_selection().unwrap().unwrap();
    selection.set_base_and_extent(&start, 8, &end, 0).unwrap();
    assert!(!selection.is_collapsed());
    let error = selection_error(&f.document).unwrap();
    assert_eq!(
        Reflect::get(&error, &"message".into())
            .unwrap()
            .as_string()
            .as_deref(),
        Some("invalid range")
    );
    assert!(f.host.selections.borrow().is_empty());
}
#[wasm_bindgen_test]
fn detached_row_keeps_weak_column_metadata_during_reentrant_focus_rotation() {
    let f = Fixture::new();
    let row = f.row(0);
    let node = row.first_child().unwrap();
    let last = f.row(2);
    f.row(1).focus().unwrap();
    let callback = Closure::<dyn Fn() -> Result<(), JsValue>>::new(move || last.focus());
    Function::new_with_args("element,callback", "const original=element.getAttribute;let first=true;Object.defineProperty(element,'getAttribute',{configurable:true,value:function(name){if(first&&name==='aria-posinset'){first=false;callback();}return original.call(this,name);}})")
        .call2(&JsValue::UNDEFINED,&row,callback.as_ref()).unwrap();
    f.document
        .get_selection()
        .unwrap()
        .unwrap()
        .set_base_and_extent(&node, 0, &node, 3)
        .unwrap();
    f.selection_change().unwrap();
    assert!(row.parent_node().is_none());
    assert_eq!(
        *f.host.selections.borrow(),
        [SelectionRange {
            row: 2.0,
            column: 0.0,
            length: 3.0
        }]
    );
    Reflect::delete_property(&row, &"getAttribute".into()).unwrap();
}
