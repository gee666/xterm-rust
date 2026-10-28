// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;

#[test]
fn expanding_lowercase_at_last_column_preserves_zero_width_first_range() {
    let h = Harness::new(4, 3, None);
    h.write("aaİB");
    assert!(h.addon.find_next(&term("B"), Some(options())).unwrap());
    // Source lowercase expansion shifts the index rather than correcting it.
    // Selection and decoration layout must retain that observable quirk.
    assert_eq!(h.selection().unwrap().start.col, 4);
    let decorations = h.decorations();
    assert_eq!(decorations.len(), 4);
    assert_eq!(
        decorations
            .iter()
            .filter(|d| d.options().borrow().width == Some(0.0))
            .count(),
        2
    );
    assert_eq!(
        decorations
            .iter()
            .filter(|d| d.options().borrow().width == Some(1.0))
            .count(),
        2
    );
}

#[test]
fn negative_decoration_width_reaches_public_guard_not_a_successful_noop() {
    let h = Harness::new(4, 3, None);
    let manager = DecorationManager::new(h.host.clone());
    let colors = options().read().unwrap().decorations.unwrap();
    let error = manager
        .create_highlights(
            &[SearchResult {
                term: term("x"),
                row: 0,
                col: 5,
                size: 1,
            }],
            &colors,
        )
        .unwrap_err();
    assert_eq!(error.to_string(), "This API only accepts positive integers");
    assert!(h.decorations().is_empty());
}
