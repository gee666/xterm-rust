// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use crate::accessibility_support::{attribute, clock::Clock, text, units, Fixture, Host};
use wasm_bindgen_test::*;
use xterm_web::accessibility::{AccessibilityEvent, AccessibilityManager, LocalizableStrings};

#[wasm_bindgen_test]
fn row_tree_roles_metadata_empty_text_and_dimensions_use_real_dom() {
    let f = Fixture::new();
    assert!(f
        .root
        .first_element_child()
        .unwrap()
        .is_same_node(Some(&f.manager.container())));
    assert!(f.root.query_selector(".existing").unwrap().is_some());
    assert_eq!(f.tree().get_attribute("role").as_deref(), Some("list"));
    assert_eq!(f.tree().children().length(), 3);
    for index in 0..3 {
        let row = f.row(index);
        assert_eq!(row.get_attribute("role").as_deref(), Some("listitem"));
        assert_eq!(row.tab_index(), -1);
        assert!(!row.has_attribute("class"));
        assert_eq!(
            row.get_attribute("aria-posinset"),
            Some((index + 3).to_string())
        );
        assert_eq!(attribute(&row, "aria-setsize"), units("6"));
        assert_eq!(row.style().get_property_value("height").unwrap(), "16px");
    }
    assert_eq!(text(&f.row(0)), units("A😀字"));
    assert_eq!(text(&f.row(1)), [0xa0]);
    assert_eq!(text(&f.row(2)), units("tail"));
    assert_eq!(
        f.live().get_attribute("aria-live").as_deref(),
        Some("assertive")
    );
    assert_eq!(
        f.manager
            .container()
            .style()
            .get_property_value("width")
            .unwrap(),
        "64px"
    );
    assert_eq!(
        f.manager
            .container()
            .style()
            .get_property_value("font-size")
            .unwrap(),
        "12px"
    );
    assert!((f.row(0).get_bounding_client_rect().width() - 40.0).abs() < 0.01);
    assert!((f.row(1).get_bounding_client_rect().width() - 8.0).abs() < 0.01);
}
#[wasm_bindgen_test]
fn missing_open_root_fails_and_cleans_owned_unattached_state() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let clock = Clock::new(&window);
    let before = document
        .query_selector_all(".xterm-accessibility")
        .unwrap()
        .length();
    let result = AccessibilityManager::new(
        window,
        document.clone(),
        None,
        Host::new(),
        LocalizableStrings::new(),
    );
    let error = result.err().unwrap();
    assert_eq!(
        js_sys::Reflect::get(&error, &"message".into())
            .unwrap()
            .as_string()
            .unwrap(),
        "Cannot enable accessibility before Terminal.open"
    );
    assert_eq!(
        document
            .query_selector_all(".xterm-accessibility")
            .unwrap()
            .length(),
        before
    );
    assert_eq!(clock.pending(), 0);
}
#[wasm_bindgen_test]
fn zero_height_negative_height_zero_width_and_zero_last_column_follow_source_guards() {
    let f = Fixture::new();
    let mut dimensions = f.host.dimensions.get();
    dimensions.cell_height = 0.0;
    dimensions.canvas_width = 200.0;
    f.host.dimensions.set(dimensions);
    f.host.emit(AccessibilityEvent::Dimensions).unwrap();
    assert_eq!(
        f.manager
            .container()
            .style()
            .get_property_value("width")
            .unwrap(),
        "64px"
    );
    dimensions.cell_height = -1.0;
    f.host.dimensions.set(dimensions);
    f.host.emit(AccessibilityEvent::Dimensions).unwrap();
    assert_eq!(
        f.manager
            .container()
            .style()
            .get_property_value("width")
            .unwrap(),
        "200px"
    );
    assert_eq!(
        f.row(0).style().get_property_value("height").unwrap(),
        "16px"
    );
    dimensions.cell_height = 16.0;
    f.host.dimensions.set(dimensions);
    f.root.style().set_property("display", "none").unwrap();
    f.host.emit(AccessibilityEvent::Dimensions).unwrap();
    assert_eq!(
        f.row(0).style().get_property_value("transform").unwrap(),
        ""
    );
    f.root.style().remove_property("display").unwrap();
    f.host.lines.borrow_mut()[2].columns = vec![0];
    f.render();
    assert_eq!(
        f.row(0).style().get_property_value("transform").unwrap(),
        ""
    );
}
#[wasm_bindgen_test]
fn dimensions_guards_and_resize_preserve_rows_until_the_render_event() {
    let f = Fixture::new();
    let first = f.row(0);
    let mut dimensions = f.host.dimensions.get();
    dimensions.cell_height = f64::NAN;
    dimensions.canvas_width = 128.0;
    f.host.dimensions.set(dimensions);
    f.host.emit(AccessibilityEvent::Dimensions).unwrap();
    assert_eq!(
        f.manager
            .container()
            .style()
            .get_property_value("width")
            .unwrap(),
        "64px"
    );
    dimensions.cell_height = 20.0;
    dimensions.cell_width = 10.0;
    dimensions.font_size = 14.0;
    f.host.dimensions.set(dimensions);
    f.host.emit(AccessibilityEvent::Dimensions).unwrap();
    assert_eq!(
        f.manager
            .container()
            .style()
            .get_property_value("width")
            .unwrap(),
        "128px"
    );
    assert_eq!(
        f.row(0).style().get_property_value("height").unwrap(),
        "20px"
    );
    assert!((f.row(0).get_bounding_client_rect().width() - 50.0).abs() < 0.01);
    f.host.rows.set(4);
    f.host.emit(AccessibilityEvent::Resize(4)).unwrap();
    assert_eq!(f.tree().children().length(), 4);
    assert_eq!(f.row(0), first);
    assert_eq!(f.row(3).get_attribute("aria-posinset"), None);
    assert!(text(&f.row(3)).is_empty());
    f.render();
    assert_eq!(text(&f.row(3)), units("last"));
    f.host.rows.set(2);
    f.host.emit(AccessibilityEvent::Resize(2)).unwrap();
    assert_eq!(f.tree().children().length(), 2);
    assert_eq!(f.row(0), first);
    f.manager.dispose().unwrap();
    f.manager.dispose().unwrap();
    assert_eq!(f.clock.pending(), 0);
    assert!(!f.root.contains(Some(&f.manager.container())));
    f.host
        .emit(AccessibilityEvent::Character(vec![65]))
        .unwrap();
    assert!(text(&f.live()).is_empty());
}
