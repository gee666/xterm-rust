// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Faithful assertions from common/buffer/BufferSet.test.ts.
use std::rc::Rc;
use xterm_core::{
    buffer::BufferSet,
    options::{OptionValue, OptionsService},
};

fn set() -> BufferSet {
    BufferSet::new(
        80,
        24,
        Rc::new(OptionsService::new(
            false,
            [("scrollback", OptionValue::Number(1000.0))],
        )),
    )
    .unwrap()
}
#[test]
fn constructor_creates_distinct_normal_and_alt_buffers() {
    let set = set();
    assert!(!Rc::ptr_eq(&set.normal(), &set.alt()));
}
#[test]
fn activating_normal_sets_it_active() {
    let set = set();
    set.activate_normal_buffer().unwrap();
    assert!(Rc::ptr_eq(&set.active(), &set.normal()));
}
#[test]
fn activating_alt_sets_it_active() {
    let set = set();
    set.activate_alt_buffer(None).unwrap();
    assert!(Rc::ptr_eq(&set.active(), &set.alt()));
}
#[test]
fn cursor_stays_stationary_when_activating_alt() {
    let set = set();
    set.activate_normal_buffer().unwrap();
    set.active().borrow_mut().x = 30;
    set.active().borrow_mut().y = 10;
    set.activate_alt_buffer(None).unwrap();
    assert_eq!(set.active().borrow().x, 30);
    assert_eq!(set.active().borrow().y, 10);
}
#[test]
fn cursor_stays_stationary_when_activating_normal() {
    let set = set();
    set.activate_alt_buffer(None).unwrap();
    set.active().borrow_mut().x = 30;
    set.active().borrow_mut().y = 10;
    set.activate_normal_buffer().unwrap();
    assert_eq!(set.active().borrow().x, 30);
    assert_eq!(set.active().borrow().y, 10);
}
#[test]
fn markers_clear_on_switch_to_normal() {
    let set = set();
    set.activate_alt_buffer(None).unwrap();
    set.alt().borrow_mut().add_marker(1);
    assert_eq!(set.alt().borrow().markers.borrow().len(), 1);
    set.activate_normal_buffer().unwrap();
    assert_eq!(set.alt().borrow().markers.borrow().len(), 0);
}
