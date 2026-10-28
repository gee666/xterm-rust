// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
#[path = "buffer_service_support/reentrant.rs"]
mod support;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use support::{new_service, on_scroll, recorder};
use xterm_core::{buffer::BufferError, event::EmitError};

#[test]
fn live_raw_fields_and_mutable_owner_slots_match_source_after_reentry() {
    let mut actual = HashMap::new();
    {
        let service = new_service();
        let set = &service.buffers;
        set.activate_alt_buffer(None).unwrap();
        set.alt().borrow_mut().x = 2;
        set.alt().borrow_mut().y = 1;
        let out = Rc::new(RefCell::new(Vec::new()));
        let save = recorder(&service, &out);
        let weak = Rc::downgrade(&service);
        let emit = Rc::clone(&save);
        set.alt()
            .borrow_mut()
            .add_marker(1)
            .on_dispose()
            .subscribe(move |_| {
                emit("marker");
                weak.upgrade().unwrap().buffers.reset().unwrap();
                emit("reset-return");
            });
        on_scroll(&service, Rc::clone(&save));
        set.activate_normal_buffer().unwrap();
        save("final");
        assert!(Rc::ptr_eq(&service.buffer(), &set.normal()));
        actual.insert("reset-during-normal-activation", out.borrow().join("^"));
    }
    {
        let service = new_service();
        let set = &service.buffers;
        let out = Rc::new(RefCell::new(Vec::new()));
        let save = recorder(&service, &out);
        let weak = Rc::downgrade(&service);
        let emit = Rc::clone(&save);
        let transient = Rc::new(RefCell::new(None));
        let capture = Rc::clone(&transient);
        set.normal()
            .borrow_mut()
            .add_marker(0)
            .on_dispose()
            .subscribe(move |_| {
                emit("old-marker");
                let service = weak.upgrade().unwrap();
                let marker = service.buffers.normal().borrow_mut().add_marker(1);
                let emit_dispose = Rc::clone(&emit);
                marker
                    .on_dispose()
                    .subscribe(move |_| emit_dispose("transient-disposed"));
                *capture.borrow_mut() = Some(marker);
                service.buffers.reset().unwrap();
                emit("inner-return");
            });
        on_scroll(&service, Rc::clone(&save));
        set.reset().unwrap();
        save("outer-return");
        let current = set.normal().borrow_mut().add_marker(0);
        let emit = Rc::clone(&save);
        current
            .on_dispose()
            .subscribe(move |_| emit("current-disposed"));
        set.dispose().unwrap();
        save("disposed");
        let transient = transient.borrow();
        let transient = transient.as_ref().unwrap();
        out.borrow_mut().push(format!(
            "markers:{},{},{},{}",
            transient.line_number(),
            u8::from(transient.is_disposed()),
            current.line_number(),
            u8::from(current.is_disposed())
        ));
        assert!(transient.is_disposed());
        assert!(!current.is_disposed());
        actual.insert("nested-reset-owner-slot", out.borrow().join("^"));
    }
    {
        let service = new_service();
        let set = &service.buffers;
        set.alt().borrow().lines.set_max_length(1).unwrap();
        let out = Rc::new(RefCell::new(Vec::new()));
        let save = recorder(&service, &out);
        let weak = Rc::downgrade(&service);
        let emit = Rc::clone(&save);
        set.alt()
            .borrow_mut()
            .add_marker(0)
            .on_dispose()
            .subscribe(move |_| {
                emit("marker");
                weak.upgrade().unwrap().buffers.reset().unwrap();
                emit("reset-return");
            });
        on_scroll(&service, Rc::clone(&save));
        set.activate_alt_buffer(None).unwrap();
        save("final");
        assert!(Rc::ptr_eq(&service.buffer(), &set.alt()));
        assert!(set.alt().borrow().lines.is_empty());
        actual.insert("reset-during-alt-fill", out.borrow().join("^"));
    }
    {
        let service = new_service();
        let set = &service.buffers;
        let out = Rc::new(RefCell::new(Vec::new()));
        let save = recorder(&service, &out);
        let failure = EmitError::callback(std::io::Error::other("exact marker failure"));
        let expected = failure.clone();
        let emit = Rc::clone(&save);
        set.normal()
            .borrow_mut()
            .add_marker(0)
            .on_dispose()
            .try_subscribe(move |_| {
                emit("old-marker");
                Err(failure.clone())
            });
        on_scroll(&service, Rc::clone(&save));
        let BufferError::Dispatch(error) = set.reset().unwrap_err() else {
            panic!("marker error must retain dispatch cause")
        };
        assert_eq!(error, expected);
        save("failed");
        let detached = set.normal();
        let late = detached.borrow_mut().add_marker(1);
        let emit = Rc::clone(&save);
        late.on_dispose().subscribe(move |_| emit("late-disposed"));
        set.reset().unwrap();
        save("retry");
        out.borrow_mut().push(format!(
            "late:{},{}",
            late.line_number(),
            u8::from(late.is_disposed())
        ));
        assert!(!late.is_disposed());
        actual.insert("failed-reset-owner-slot", out.borrow().join("^"));
    }
    let fixture = include_str!("fixtures/buffer-set-owners.tsv");
    assert_eq!(fixture.lines().count(), 4);
    for line in fixture.lines() {
        let (case, expected) = line.split_once('\t').unwrap();
        assert_eq!(actual.remove(case).unwrap(), expected, "case {case}");
    }
    assert!(actual.is_empty());
}
