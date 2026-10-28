//! JavaScript Number loop boundaries, separate from ordinary buffer-row traces.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::marker::Marker,
    decoration::{DecorationError, DecorationOptions, DecorationService},
};
const EDGE: i64 = 1_i64 << 53;
#[test]
fn finite_large_line_spans_use_source_end_rounding_and_number_increment() {
    for (line, height, expected) in [
        (EDGE, 1, [0, 0, 0, 0]),
        (EDGE - 1, 2, [0, 1, 0, 0]),
        (EDGE + 2, 3, [0, 1, 0, 0]),
        (EDGE + 4, 1, [0, 0, 0, 0]),
    ] {
        let service = DecorationService::new();
        let mut options = DecorationOptions::new(Marker::new(line));
        options.height = Some(height);
        let decoration = service
            .register_decoration(Rc::new(RefCell::new(options)))
            .unwrap()
            .unwrap();
        for (offset, expected) in [-1, 0, 1, 2].into_iter().zip(expected) {
            assert_eq!(
                service
                    .get_decorations_at_cell(0.0, line + offset, None)
                    .count(),
                expected,
                "line{line} height{height} offset{offset}"
            );
        }
        decoration.dispose().unwrap();
        assert_eq!(service.decorations().unwrap().count(), 0);
    }
}
#[test]
fn callback_error_source_chain_retains_the_actual_object() {
    use std::error::Error;
    use xterm_core::event::EmitError;
    let service = DecorationService::new();
    let cause = Rc::new(std::io::Error::other("decoration callback cause"));
    let expected = Rc::clone(&cause);
    service
        .on_decoration_registered()
        .try_subscribe(move |_| Err(EmitError::Callback(cause.clone())));
    let options = DecorationOptions::new(Marker::new(0));
    let error = match service.register_decoration(Rc::new(RefCell::new(options))) {
        Err(error) => error,
        Ok(_) => panic!("callback error lost"),
    };
    let actual = error
        .source()
        .unwrap()
        .source()
        .unwrap()
        .downcast_ref::<std::io::Error>()
        .unwrap();
    assert!(std::ptr::eq(actual, expected.as_ref()));
}

#[test]
fn non_progressing_reference_span_is_a_typed_native_error_not_a_hang() {
    let service = DecorationService::new();
    let mut options = DecorationOptions::new(Marker::new(EDGE));
    options.height = Some(2);
    assert!(matches!(
        service.register_decoration(Rc::new(RefCell::new(options))),
        Err(DecorationError::NonAdvancingSpan(_))
    ));
    assert_eq!(service.decorations().unwrap().count(), 0);
}
