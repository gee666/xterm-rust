// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
#[path = "buffer_service_support/reentrant.rs"]
mod support;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};
use xterm_core::buffer::{AttributeData, BufferService};

use support::new_service as service;
fn full_service() -> Rc<BufferService> {
    let service = service();
    while !service.buffer().borrow().lines.is_full() {
        service.scroll(&AttributeData::default(), false).unwrap();
    }
    let active = service.buffer();
    let active = active.borrow();
    for row in 0..active.lines.len() {
        active
            .lines
            .get(row)
            .unwrap()
            .borrow_mut()
            .set_cell_from_codepoint(
                0,
                65 + u32::try_from(row).unwrap(),
                1,
                &AttributeData::default(),
            );
    }
    drop(active);
    service
}
use support::{on_scroll, recorder};
#[test]
fn source_exact_partial_state_and_nested_callback_traces() {
    let mut actual = HashMap::new();
    {
        let service = full_service();
        let active = service.buffer();
        let out = Rc::new(RefCell::new(Vec::new()));
        let save = recorder(&service, &out);
        let marker = active.borrow_mut().add_marker(0);
        let weak = Rc::downgrade(&service);
        let buffer = Rc::downgrade(&active);
        let emit = Rc::clone(&save);
        marker.on_dispose().subscribe(move |_| {
            emit("marker");
            let buffer = buffer.upgrade().unwrap();
            buffer.borrow_mut().x = 2;
            buffer.borrow_mut().ydisp = 0;
            weak.upgrade().unwrap().is_user_scrolling.set(true);
            emit("changed");
        });
        on_scroll(&service, Rc::clone(&save));
        service.scroll(&AttributeData::default(), false).unwrap();
        save("final");
        assert_eq!(marker.line_number(), -1.0);
        assert!(marker.is_disposed());
        actual.insert("trim-read-write", out.borrow().join("^"));
    }
    {
        let service = full_service();
        let out = Rc::new(RefCell::new(Vec::new()));
        let save = recorder(&service, &out);
        let nested = Cell::new(false);
        let weak = Rc::downgrade(&service);
        let emit = Rc::clone(&save);
        service
            .buffer()
            .borrow()
            .lines
            .on_trim_emitter
            .event()
            .subscribe(move |_| {
                emit("trim");
                if !nested.replace(true) {
                    weak.upgrade()
                        .unwrap()
                        .scroll(&AttributeData::default(), true)
                        .unwrap();
                    emit("nested-return");
                }
            });
        on_scroll(&service, Rc::clone(&save));
        service.scroll(&AttributeData::default(), false).unwrap();
        save("final");
        actual.insert("nested-scroll", out.borrow().join("^"));
    }
    {
        let service = service();
        service.buffers.activate_alt_buffer(None).unwrap();
        let alt = service.buffers.alt();
        alt.borrow_mut().x = 3;
        alt.borrow_mut().y = 1;
        let out = Rc::new(RefCell::new(Vec::new()));
        let save = recorder(&service, &out);
        let marker = alt.borrow_mut().add_marker(1);
        let weak = Rc::downgrade(&service);
        let alt_ref = Rc::downgrade(&alt);
        let emit = Rc::clone(&save);
        marker.on_dispose().subscribe(move |_| {
            emit("marker");
            let alt = alt_ref.upgrade().unwrap();
            alt.borrow_mut().x = 8;
            weak.upgrade().unwrap().buffers.normal().borrow_mut().y = 2;
            let added = alt.borrow_mut().add_marker(2);
            let emit_added = Rc::clone(&emit);
            added
                .on_dispose()
                .subscribe(move |_| emit_added("added-marker"));
            emit("changed");
        });
        on_scroll(&service, Rc::clone(&save));
        service.buffers.activate_normal_buffer().unwrap();
        save("final");
        assert!(alt.borrow().markers.borrow().is_empty());
        actual.insert("activation-marker", out.borrow().join("^"));
    }
    {
        let service = service();
        let old = service.buffer();
        let out = Rc::new(RefCell::new(Vec::new()));
        let save = recorder(&service, &out);
        let marker = old.borrow_mut().add_marker(0);
        let weak = Rc::downgrade(&service);
        let old_ref = Rc::downgrade(&old);
        let emit = Rc::clone(&save);
        marker.on_dispose().subscribe(move |_| {
            emit("marker");
            weak.upgrade().unwrap().buffers.normal().borrow_mut().x = 3;
            old_ref.upgrade().unwrap().borrow_mut().x = 2;
            emit("changed");
        });
        on_scroll(&service, Rc::clone(&save));
        service.reset().unwrap();
        save("final");
        actual.insert("reset-marker", out.borrow().join("^"));
    }
    {
        let service = full_service();
        let out = Rc::new(RefCell::new(Vec::new()));
        let save = recorder(&service, &out);
        let activated = Cell::new(false);
        let weak = Rc::downgrade(&service);
        let emit = Rc::clone(&save);
        service
            .buffer()
            .borrow()
            .lines
            .on_trim_emitter
            .event()
            .subscribe(move |_| {
                emit("trim");
                if !activated.replace(true) {
                    weak.upgrade()
                        .unwrap()
                        .buffers
                        .activate_alt_buffer(None)
                        .unwrap();
                    emit("activated");
                }
            });
        on_scroll(&service, Rc::clone(&save));
        service.scroll(&AttributeData::default(), false).unwrap();
        save("final");
        actual.insert("trim-activation", out.borrow().join("^"));
    }
    {
        let service = full_service();
        let out = Rc::new(RefCell::new(Vec::new()));
        let save = recorder(&service, &out);
        let resized = Cell::new(false);
        let weak = Rc::downgrade(&service);
        let emit = Rc::clone(&save);
        service
            .buffer()
            .borrow()
            .lines
            .on_trim_emitter
            .event()
            .subscribe(move |_| {
                emit("trim");
                if !resized.replace(true) {
                    weak.upgrade().unwrap().resize(6, 4).unwrap();
                    emit("resized");
                }
            });
        let emit = Rc::clone(&save);
        service.on_resize().subscribe(move |_| emit("resize"));
        on_scroll(&service, Rc::clone(&save));
        service.scroll(&AttributeData::default(), false).unwrap();
        save("final");
        actual.insert("trim-resize", out.borrow().join("^"));
    }
    let fixture = include_str!("fixtures/buffer-service-reentrant.tsv");
    assert_eq!(fixture.lines().count(), 6);
    for line in fixture.lines() {
        let (case, expected) = line.split_once('\t').unwrap();
        assert_eq!(actual.remove(case).unwrap(), expected, "case {case}");
    }
    assert!(actual.is_empty());
}
