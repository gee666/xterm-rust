//! Independent source-derived regressions found during task64 review.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::marker::Marker,
    decoration::{DecorationOptions, DecorationService},
};

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
