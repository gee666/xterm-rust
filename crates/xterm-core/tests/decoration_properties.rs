//! Live host properties share native decoration indexing and dispatch.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    buffer::marker::Marker,
    decoration::{DecorationOptions, DecorationService},
    options::{CoercionError, OptionObject, OptionValue as V},
};
fn options(marker: Marker) -> Rc<RefCell<DecorationOptions>> {
    Rc::new(RefCell::new(DecorationOptions::new(marker)))
}
#[test]
fn source_fields_are_live_and_never_require_a_native_options_borrow() {
    let service = DecorationService::new();
    let native = options(Marker::new(0));
    let geometry = Rc::new(RefCell::new((0.0, 1.0)));
    let reads = Rc::clone(&geometry);
    let native_in_getter = Rc::clone(&native);
    let source = V::Object(OptionObject::with_properties(
        (),
        |_| Ok(V::Undefined),
        move |_, key| {
            // This mutable borrow would fail if a property read retained a native
            // options borrow while invoking the host.
            native_in_getter.borrow_mut().layer = None;
            Ok(match key {
                "x" => reads.borrow().0.into(),
                "width" => reads.borrow().1.into(),
                _ => V::Undefined,
            })
        },
    ));
    let decoration = service
        .register_decoration_source(native, source, None)
        .unwrap();
    let lookup = |x| {
        service
            .get_decorations_at_cell(x, 0, None)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    };
    assert!(lookup(0.0)[0].ptr_eq(&decoration));
    *geometry.borrow_mut() = (2.0, 2.0);
    assert!(lookup(0.0).is_empty());
    assert!(lookup(2.0)[0].ptr_eq(&decoration));
    assert!(lookup(4.0).is_empty());
}
#[test]
fn captured_string_height_uses_addition_not_numeric_projection() {
    let service = DecorationService::new();
    let source = V::property_map([("height", "2".into())]);
    service
        .register_decoration_source(options(Marker::new(1)), source, None)
        .unwrap();
    // Source 1 + '2' is '12', so numeric post-increment indexes through 11.
    assert_eq!(service.get_decorations_at_cell(0.0, 11, None).count(), 1);
    assert_eq!(service.get_decorations_at_cell(0.0, 12, None).count(), 0);
}
#[test]
fn height_conversion_is_repeated_at_each_loop_comparison() {
    let service = DecorationService::new();
    let conversions = Rc::new(Cell::new(0));
    let count = Rc::clone(&conversions);
    let height = V::Object(OptionObject::new((), move |_| {
        count.set(count.get() + 1);
        Ok(2.0.into())
    }));
    let source = V::property_map([("height", height)]);
    service
        .register_decoration_source(options(Marker::new(0)), source, None)
        .unwrap();
    assert_eq!(conversions.get(), 3);
}
#[test]
fn throwing_lookup_is_fallible_and_exhausts_like_a_generator() {
    let service = DecorationService::new();
    let source = V::Object(OptionObject::with_properties(
        (),
        |_| Ok(V::Undefined),
        |_, key| {
            if key == "width" {
                return Err(CoercionError::new("width getter"));
            }
            Ok(V::Undefined)
        },
    ));
    service
        .register_decoration_source(options(Marker::new(0)), source, None)
        .unwrap();
    let mut values = service.get_decorations_at_cell(0.0, 0, None);
    assert_eq!(
        values.next().unwrap().err().unwrap().to_string(),
        "width getter"
    );
    assert!(values.next().is_none());
}
#[test]
fn disposal_cleanup_follows_event_success_and_is_not_replayed() {
    let service = DecorationService::new();
    let decoration = service
        .register_decoration(options(Marker::new(0)))
        .unwrap()
        .unwrap();
    let phases = Rc::new(RefCell::new(Vec::new()));
    let log = Rc::clone(&phases);
    decoration
        .on_dispose()
        .subscribe(move |_| log.borrow_mut().push("event"));
    let log = Rc::clone(&phases);
    decoration
        .set_host_cleanup(Rc::new(move || {
            log.borrow_mut().push("cleanup");
            Ok(())
        }))
        .unwrap();
    decoration.dispose().unwrap();
    decoration.dispose().unwrap();
    assert_eq!(*phases.borrow(), ["event", "cleanup"]);
}

#[test]
fn host_cleanup_is_installed_before_a_throwing_registration_getter() {
    let service = DecorationService::new();
    let fail = Cell::new(true);
    let source = V::Object(OptionObject::with_properties(
        (),
        |_| Ok(V::Undefined),
        move |_, key| {
            if key == "height" && fail.replace(false) {
                return Err(CoercionError::new("height getter"));
            }
            Ok(V::Undefined)
        },
    ));
    let cleanups = Rc::new(Cell::new(0));
    let initialize = |decoration: &xterm_core::decoration::Decoration| {
        let count = Rc::clone(&cleanups);
        decoration.set_host_cleanup(Rc::new(move || {
            count.set(count.get() + 1);
            Ok(())
        }))
    };
    assert!(service
        .register_decoration_source(options(Marker::new(0)), source, Some(&initialize))
        .is_err());
    assert_eq!(
        service.decorations().unwrap().count(),
        1,
        "source insertion precedes cache getter failure"
    );
    assert_eq!(cleanups.get(), 0);
    service.reset().unwrap();
    assert_eq!(cleanups.get(), 1);
    service.reset().unwrap();
    assert_eq!(cleanups.get(), 1);
}
