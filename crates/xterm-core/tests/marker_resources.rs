//! Source Marker.register traversal includes duplicates and late appends.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::marker::{Marker, MarkerResource},
    event::EmitError,
    lifecycle::to_disposable,
};

#[test]
fn fallible_and_native_resources_share_live_insertion_order() {
    let marker = Marker::new(4);
    let log = Rc::new(RefCell::new(Vec::new()));
    let weak = marker.downgrade();
    let resource_log = Rc::clone(&log);
    let resource: MarkerResource = Rc::new(move || {
        resource_log.borrow_mut().push("fallible");
        let marker = weak.upgrade().unwrap();
        assert!(marker.is_disposed());
        assert_eq!(marker.line_number(), -1.0);
        marker.dispose()?;
        let log = Rc::clone(&resource_log);
        marker.register(to_disposable(move || log.borrow_mut().push("appended")));
        Ok(())
    });
    assert!(Rc::ptr_eq(
        &resource,
        &marker.register_fallible(Rc::clone(&resource))
    ));
    let native_log = Rc::clone(&log);
    marker.register(to_disposable(move || {
        native_log.borrow_mut().push("native")
    }));
    marker.register_fallible(resource);
    marker.dispose().unwrap();
    marker.dispose().unwrap();
    assert_eq!(
        *log.borrow(),
        ["fallible", "native", "fallible", "appended", "appended"]
    );
    let late_log = Rc::clone(&log);
    marker.register_fallible(Rc::new(move || {
        late_log.borrow_mut().push("late");
        Ok(())
    }));
    marker.dispose().unwrap();
    assert_eq!(log.borrow().len(), 5);
}

#[test]
fn fallible_resource_error_stops_without_retry_or_early_release() {
    let marker = Marker::new(2);
    let log = Rc::new(RefCell::new(Vec::new()));
    let first_log = Rc::clone(&log);
    marker.register_fallible(Rc::new(move || {
        first_log.borrow_mut().push("error");
        Err(EmitError::ListenerArrayCleared)
    }));
    let resource: MarkerResource = Rc::new(|| panic!("cleanup continued after throw"));
    marker.register_fallible(Rc::clone(&resource));
    assert!(matches!(
        marker.dispose(),
        Err(EmitError::ListenerArrayCleared)
    ));
    assert!(marker.is_disposed());
    assert_eq!(marker.line_number(), -1.0);
    assert_eq!(Rc::strong_count(&resource), 2);
    marker.dispose().unwrap();
    assert_eq!(*log.borrow(), ["error"]);
    drop(marker);
    assert_eq!(Rc::strong_count(&resource), 1);
}

#[test]
fn successful_disposal_releases_resources_and_listener_error_keeps_them() {
    let resource: MarkerResource = Rc::new(|| Ok(()));
    let marker = Marker::new(0);
    marker.register_fallible(Rc::clone(&resource));
    marker.dispose().unwrap();
    assert_eq!(Rc::strong_count(&resource), 1);

    let marker = Marker::new(0);
    marker
        .on_dispose()
        .try_subscribe(|_| Err(EmitError::ListenerArrayCleared));
    marker.register_fallible(Rc::clone(&resource));
    assert!(marker.dispose().is_err());
    assert_eq!(Rc::strong_count(&resource), 2);
    marker.dispose().unwrap();
    drop(marker);
    assert_eq!(Rc::strong_count(&resource), 1);
}
