// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
#[path = "support/buffer_reentry.rs"]
mod harness;
use harness::{larger, smaller, trimmed, Harness, ObserverFailure};
use std::{cell::Cell, rc::Rc};
use xterm_core::{
    buffer::Buffer,
    event::EmitError,
    options::{OptionObject, OptionValue as V},
};

#[test]
fn trim_read_write() {
    let h = trimmed();
    let weak = Rc::downgrade(&h);
    let lines = h.buffer.borrow().lines.share();
    h.retain(lines.on_trim_emitter.event().subscribe(move |amount| {
        let h = weak.upgrade().unwrap();
        h.snapshot(&format!("trim,{amount}"));
        {
            let mut b = h.buffer.borrow_mut();
            b.x = 2;
            b.ybase = 4;
            b.ydisp = 3;
            b.saved_y = 8;
        }
        h.snapshot("changed");
    }));
    h.resize(4, 2);
    h.snapshot("final");
    h.assert_fixture("trim-read-write");
}

#[test]
fn trim_nested_resize() {
    let h = trimmed();
    let weak = Rc::downgrade(&h);
    let lines = h.buffer.borrow().lines.share();
    let nested = Cell::new(false);
    h.retain(lines.on_trim_emitter.event().subscribe(move |amount| {
        let h = weak.upgrade().unwrap();
        h.snapshot(&format!("trim,{amount}"));
        if !nested.replace(true) {
            h.resize(6, 3);
            h.snapshot("nested-return");
        }
    }));
    h.resize(4, 2);
    h.snapshot("final");
    h.assert_fixture("trim-nested-resize");
}

#[test]
fn larger_read_write() {
    let h = larger();
    let weak = Rc::downgrade(&h);
    let lines = h.buffer.borrow().lines.share();
    h.retain(lines.on_delete_emitter.event().subscribe(move |event| {
        let h = weak.upgrade().unwrap();
        h.snapshot(&format!("delete,{},{}", event.index, event.amount));
        {
            let mut b = h.buffer.borrow_mut();
            b.x = 3;
            b.y = 2;
            b.saved_y = 10;
        }
        h.snapshot("changed");
    }));
    h.resize(4, 5);
    h.snapshot("final");
    h.assert_fixture("larger-read-write");
}

#[test]
fn larger_nested_resize() {
    let h = larger();
    let weak = Rc::downgrade(&h);
    let lines = h.buffer.borrow().lines.share();
    let nested = Cell::new(false);
    h.retain(lines.on_delete_emitter.event().subscribe(move |event| {
        let h = weak.upgrade().unwrap();
        h.snapshot(&format!("delete,{},{}", event.index, event.amount));
        if !nested.replace(true) {
            h.resize(4, 5);
            h.snapshot("nested-return");
        }
    }));
    h.resize(4, 5);
    h.snapshot("final");
    h.assert_fixture("larger-nested-resize");
}

#[test]
fn smaller_read_write() {
    let h = smaller();
    let weak = Rc::downgrade(&h);
    let lines = h.buffer.borrow().lines.share();
    h.retain(lines.on_insert_emitter.event().subscribe(move |event| {
        let h = weak.upgrade().unwrap();
        h.snapshot(&format!("insert,{},{}", event.index, event.amount));
        {
            let mut b = h.buffer.borrow_mut();
            b.x = 3;
            b.y = 2;
            b.saved_y = 7;
        }
        h.letters(0, "Z");
        h.snapshot("changed");
    }));
    h.resize(2, 5);
    h.snapshot("final");
    h.assert_fixture("smaller-read-write");
}

#[test]
fn smaller_nested_resize() {
    let h = smaller();
    let weak = Rc::downgrade(&h);
    let lines = h.buffer.borrow().lines.share();
    let nested = Cell::new(false);
    h.retain(lines.on_insert_emitter.event().subscribe(move |event| {
        let h = weak.upgrade().unwrap();
        h.snapshot(&format!("insert,{},{}", event.index, event.amount));
        if !nested.replace(true) {
            h.resize(3, 5);
            h.snapshot("nested-return");
        }
    }));
    h.resize(2, 5);
    h.snapshot("final");
    h.assert_fixture("smaller-nested-resize");
}

#[test]
fn scrollback_coercion_reentry() {
    let h = Harness::new(4, 3);
    let weak = Rc::downgrade(&h);
    let object = OptionObject::new((), move |_| {
        let h = weak.upgrade().unwrap();
        h.snapshot("coerce");
        h.buffer.borrow_mut().saved_y = 7;
        Ok(V::Number(2.0))
    });
    h.options.set_raw("scrollback", V::Object(object)).unwrap();
    h.resize(4, 3);
    h.snapshot("final");
    h.assert_fixture("scrollback-coercion-reentry");
}

#[test]
fn windows_getter_reentry() {
    let h = Harness::new(4, 3);
    let object = OptionObject::with_properties(
        Rc::downgrade(&h),
        |_| Ok(V::from("[object Object]")),
        |weak, key| {
            let h = weak.upgrade().unwrap();
            Ok(match key {
                "buildNumber" => {
                    h.snapshot("build");
                    h.buffer.borrow_mut().saved_y += 1;
                    V::Number(21376.0)
                }
                "backend" => {
                    h.snapshot("backend");
                    V::from("conpty")
                }
                _ => V::Undefined,
            })
        },
    );
    h.options.set_raw("windowsPty", V::Object(object)).unwrap();
    h.resize(4, 3);
    h.snapshot("final");
    h.assert_fixture("windows-getter-reentry");
}

#[test]
fn trim_callback_throws() {
    let h = trimmed();
    let weak = Rc::downgrade(&h);
    let lines = h.buffer.borrow().lines.share();
    h.retain(lines.on_trim_emitter.event().try_subscribe(move |_| {
        weak.upgrade().unwrap().snapshot("trim");
        Err(EmitError::callback(ObserverFailure))
    }));
    let error = Buffer::resize_shared(&h.buffer, 4, 2).unwrap_err();
    let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(&error);
    let mut retained = false;
    while let Some(error) = cause {
        retained |= error.downcast_ref::<ObserverFailure>().is_some();
        cause = error.source();
    }
    assert!(
        retained,
        "must retain original callback object through all wrappers"
    );
    h.snapshot("failed-state");
    h.assert_fixture("trim-callback-throws");
}

#[test]
fn sparse_replacement_then_smaller() {
    let h = Harness::with_scrollback(5, 4, 6);
    {
        let mut b = h.buffer.borrow_mut();
        for _ in 0..4 {
            b.lines
                .push(b.get_blank_line(&xterm_core::buffer::AttributeData::default(), false))
                .unwrap();
        }
        b.x = 1;
        b.y = 3;
        b.ybase = 4;
        b.ydisp = 4;
    }
    h.letters(0, "abcde");
    h.letters(1, "fghij");
    h.buffer.borrow().set_wrapped(1, true).unwrap();
    for i in 2..8 {
        h.letters(
            i,
            &char::from_u32(105 + u32::try_from(i).unwrap())
                .unwrap()
                .to_string(),
        );
    }
    let weak = Rc::downgrade(&h);
    let lines = h.buffer.borrow().lines.share();
    h.retain(lines.on_delete_emitter.event().subscribe(move |event| {
        let h = weak.upgrade().unwrap();
        h.snapshot(&format!("delete,{},{}", event.index, event.amount));
        {
            let mut b = h.buffer.borrow_mut();
            b.clear().unwrap();
            b.fill_viewport_rows(None).unwrap();
        }
        let count = h.buffer.borrow().lines.len();
        for i in 0..count {
            let line = h.buffer.borrow().lines.get(i).unwrap();
            let null = h.buffer.borrow().get_null_cell_snapshot(None);
            line.borrow_mut().resize(10, &null);
            h.letters(i, "abcdefghij");
        }
        h.snapshot("refilled");
    }));
    h.resize(10, 4);
    h.snapshot("after-widen");
    assert!(matches!(
        Buffer::resize_shared(&h.buffer, 4, 4),
        Err(xterm_core::buffer::BufferError::MissingLine(9))
    ));
    h.snapshot("failed-state");
    h.assert_fixture("sparse-replacement-then-smaller");
}

#[test]
fn larger_callback_replaces_list() {
    let h = larger();
    let weak = Rc::downgrade(&h);
    let lines = h.buffer.borrow().lines.share();
    h.retain(lines.on_delete_emitter.event().subscribe(move |_| {
        let h = weak.upgrade().unwrap();
        h.snapshot("delete");
        h.buffer.borrow_mut().clear().unwrap();
        h.snapshot("cleared");
    }));
    h.resize(4, 5);
    h.snapshot("final");
    h.assert_fixture("larger-callback-replaces-list");
}
