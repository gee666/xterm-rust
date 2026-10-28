// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
use xterm_core::{
    buffer::{constants::content, AttributeData, Buffer, ScratchCell},
    options::{OptionObject, OptionValue as V, OptionsService},
};
fn buffer() -> Buffer {
    Buffer::new(true, 4, 2, Rc::new(OptionsService::default())).unwrap()
}
#[test]
fn repeated_calls_update_held_handles_but_not_owned_snapshots() {
    let b = buffer();
    let first = b.get_null_cell(Some(&AttributeData {
        fg: 3,
        ..Default::default()
    }));
    let snapshot = first.snapshot();
    let second = b.get_null_cell(Some(&AttributeData {
        fg: 9,
        ..Default::default()
    }));
    assert!(first.same_identity(&second));
    assert_eq!(first.fg(), 9);
    assert_eq!(snapshot.fg, 3);
    let white = b.get_whitespace_cell(None);
    assert!(!first.same_identity(&white));
    assert_eq!(white.snapshot().get_code(), Some(32));
    let again = b.get_whitespace_cell(Some(&AttributeData {
        bg: 11,
        ..Default::default()
    }));
    assert!(white.same_identity(&again));
    assert_eq!(white.bg(), 11);
    assert_eq!(first.bg(), 0);
}
#[test]
fn default_reset_preserves_mutated_content_width_and_combined_data() {
    let b = buffer();
    let held = b.get_null_cell(None);
    held.set_content(content::IS_COMBINED_MASK | (2 << content::WIDTH_SHIFT));
    held.set_combined_data(vec![0xd800, 0x301]);
    held.set_fg(123);
    held.set_bg(456);
    let reset = b.get_null_cell(None);
    assert!(held.same_identity(&reset));
    assert_eq!((held.fg(), held.bg()), (0, 0));
    assert_eq!(held.snapshot().get_width(), 2);
    assert_eq!(held.snapshot().get_chars(), [0xd800, 0x301]);
}
#[test]
fn supplied_extended_attributes_share_identity_and_default_reset_replaces_it() {
    let b = buffer();
    let mut attr = AttributeData::default();
    attr.extended.set_url_id(7);
    let held = b.get_null_cell(Some(&attr));
    assert!(held.extended().same_identity(&attr.extended));
    attr.extended.set_url_id(12);
    assert_eq!(held.extended().url_id(), 12);
    b.get_null_cell(None);
    assert_eq!(held.extended().url_id(), 0);
    assert!(!held.extended().same_identity(&attr.extended));
}
#[test]
fn source_backend_getter_mutation_changes_resize_padding_on_both_paths() {
    for shared in [false, true] {
        let options = Rc::new(OptionsService::default());
        let mut b = Buffer::new(true, 4, 2, options.clone()).unwrap();
        b.fill_viewport_rows(None).unwrap();
        let held = b.get_null_cell(None);
        let reads = Rc::new(Cell::new(0));
        let object = OptionObject::with_properties(
            (held.clone(), reads.clone()),
            |_| Ok(V::from("[object Object]")),
            |(cell, reads): &(ScratchCell, Rc<Cell<usize>>), key| {
                if key == "backend" {
                    reads.set(reads.get() + 1);
                    cell.set_fg(123);
                    Ok(V::from("conpty"))
                } else {
                    Ok(V::Undefined)
                }
            },
        );
        options.set_raw("windowsPty", V::Object(object)).unwrap();
        if shared {
            let handle = Rc::new(RefCell::new(b));
            Buffer::resize_shared(&handle, 4, 3).unwrap();
            b = Rc::try_unwrap(handle)
                .unwrap_or_else(|_| panic!("unexpected retained owner"))
                .into_inner();
        } else {
            b.resize(4, 3).unwrap();
        }
        assert_eq!(reads.get(), 1);
        assert_eq!(held.fg(), 123);
        assert_eq!(b.lines.get(2).unwrap().borrow().get_fg(0), Some(123));
    }
}
struct Payload(Weak<RefCell<Buffer>>);
impl Drop for Payload {
    fn drop(&mut self) {
        let b = self.0.upgrade().unwrap();
        b.borrow().get_null_cell(None).set_fg(77);
    }
}
#[test]
fn replacing_payload_can_reenter_scratch_without_a_held_cell_borrow() {
    let b = Rc::new(RefCell::new(buffer()));
    let held = b.borrow().get_null_cell(None);
    let mut extended = held.extended();
    extended.set_payload(Some(Rc::new(Payload(Rc::downgrade(&b)))));
    drop(extended);
    b.borrow().get_null_cell(None);
    assert_eq!(held.fg(), 77);
}
