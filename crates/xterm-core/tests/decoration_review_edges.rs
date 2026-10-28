//! Independent source-derived regressions found during task64 review.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::marker::Marker,
    decoration::{DecorationOptions, DecorationService},
};

#[test]
fn held_readonly_bucket_observes_removal_without_becoming_a_snapshot() {
    let service = DecorationService::new();
    let decoration = service
        .register_decoration(Rc::new(RefCell::new(DecorationOptions::new(Marker::new(
            0,
        )))))
        .unwrap()
        .unwrap();
    let bucket = service.line_cache().decorations_on_line(0).unwrap();
    assert_eq!(bucket.len(), 1);
    decoration.dispose().unwrap();
    assert_eq!(bucket.len(), 0);
}

#[test]
fn cold_cell_iterator_observes_registration_before_first_next() {
    let service = DecorationService::new();
    let mut iterator = service.get_decorations_at_cell(0.0, 0, None);
    let decoration = service
        .register_decoration(Rc::new(RefCell::new(DecorationOptions::new(Marker::new(
            0,
        )))))
        .unwrap()
        .unwrap();
    assert!(iterator
        .next()
        .is_some_and(|value| value.ptr_eq(&decoration)));
}
