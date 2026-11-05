//! Host delivery shares the native CoreService admission and effect ordering.
mod service_support;
use service_support::Harness;
use std::{cell::Cell, rc::Rc};
use xterm_core::{
    core_service::{BufferPosition, CoreError},
    event::EmitError,
};

#[test]
fn host_delivery_obeys_all_admission_and_scroll_branches() {
    let h = Harness::new([]);
    h.listen();
    for disabled in [false, true] {
        for user in [false, true] {
            for scroll in [false, true] {
                for display in [0, 1] {
                    h.options.set("disableStdin", disabled.into()).unwrap();
                    h.options.set("scrollOnUserInput", scroll.into()).unwrap();
                    h.position.set(BufferPosition {
                        ybase: 1,
                        ydisp: display,
                    });
                    h.records.order.borrow_mut().clear();
                    h.core
                        .trigger_data_event_with(user, |logger| {
                            assert!(std::ptr::eq(logger, h.logger.as_ref()));
                            h.records.order.borrow_mut().push("raw".into());
                            Ok(())
                        })
                        .unwrap();
                    let mut expected = Vec::new();
                    if !disabled {
                        if user && scroll && display != 1 {
                            expected.push("scroll");
                        }
                        if user {
                            expected.push("user");
                        }
                        expected.push("raw");
                    }
                    assert_eq!(*h.records.order.borrow(), expected);
                }
            }
        }
    }
}

#[test]
fn admitted_delivery_survives_option_changes_and_reenters_without_borrow() {
    let h = Harness::new([]);
    h.listen();
    h.position.set(BufferPosition { ybase: 1, ydisp: 0 });
    let options = Rc::clone(&h.options);
    let core = Rc::downgrade(&h.core);
    h.core.on_request_scroll_to_bottom().subscribe(move |_| {
        options.set("disableStdin", true.into()).unwrap();
        core.upgrade()
            .unwrap()
            .trigger_data_event_with(true, |_| {
                panic!("nested input must see disableStdin");
            })
            .unwrap();
    });
    h.core
        .trigger_data_event_with(true, |_| {
            h.records.order.borrow_mut().push("raw".into());
            h.options.set("disableStdin", false.into()).unwrap();
            h.core.trigger_data_event_with(false, |_| {
                h.records.order.borrow_mut().push("nested".into());
                Ok(())
            })
        })
        .unwrap();
    assert_eq!(
        *h.records.order.borrow(),
        ["scroll", "user", "raw", "nested"]
    );
}

#[test]
fn effect_and_delivery_errors_stop_at_the_source_boundary() {
    let h = Harness::new([]);
    h.listen();
    let reached = Cell::new(false);
    let listener = h
        .core
        .on_user_input()
        .try_subscribe(|_| Err(EmitError::callback(std::io::Error::other("user error"))));
    assert!(h
        .core
        .trigger_data_event_with(true, |_| {
            reached.set(true);
            Ok(())
        })
        .is_err());
    assert!(!reached.get());
    listener.dispose();
    let error = h
        .core
        .trigger_data_event_with(true, |_| {
            reached.set(true);
            Err(CoreError::Dispatch(EmitError::callback(
                std::io::Error::other("delivery error"),
            )))
        })
        .unwrap_err();
    assert!(reached.get());
    assert_eq!(error.to_string(), "delivery error");
}
