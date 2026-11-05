// Copyright (c) 2016 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::Cell, rc::Rc};
use xterm_core::{
    buffer::{constants::bg_flags, BufferLine, CellData},
    circular_list::CircularList,
};

#[test]
fn extended_map_read_retains_replaced_map() {
    let mut line = BufferLine::new(2, None, false);
    let mut cell = CellData::default();
    cell.attributes.bg = bg_flags::HAS_EXTENDED;
    cell.attributes.extended.set_url_id(7);
    line.set_cell(0, &cell);
    let read = line.prepare_extended_read();
    line.fill(&CellData::default(), false);
    assert_eq!(read.get(0).unwrap().url_id(), 7);
    assert_eq!(line.get_extended(0).unwrap().url_id(), 0);
}

#[test]
fn extended_map_read_observes_in_place_set_and_resize_deletion() {
    let mut line = BufferLine::new(2, None, false);
    let mut cell = CellData::default();
    cell.attributes.bg = bg_flags::HAS_EXTENDED;
    cell.attributes.extended.set_url_id(7);
    line.set_cell(1, &cell);
    let read = line.prepare_extended_read();
    cell.attributes.extended.set_url_id(9);
    line.set_cell(1, &cell);
    assert_eq!(read.get(1).unwrap().url_id(), 9);
    line.resize(1, &CellData::default());
    assert!(read.get(1).is_none());
}

#[test]
fn phased_read_captures_start_before_conversion() {
    let list = CircularList::new(3).unwrap();
    for value in [10, 20, 30] {
        list.push(value).unwrap();
    }
    list.push(40).unwrap();
    let read = list.prepare_number_read();
    assert_eq!(
        read.get_with::<()>(|start| {
            assert_eq!(start, 1.0);
            list.push(50).unwrap();
            Ok(start)
        })
        .unwrap(),
        Some(50)
    );
    assert_eq!(list.get(0), Some(30));
}

#[test]
fn phased_read_observes_live_capacity_but_original_array() {
    let list = CircularList::new(4).unwrap();
    for value in [10, 20, 30, 40] {
        list.push(value).unwrap();
    }
    let read = list.prepare_number_read();
    assert_eq!(
        read.get_with::<()>(|start| {
            list.set_max_length(2).unwrap();
            list.set(1, Some(99)).unwrap();
            Ok(start + 3.0)
        })
        .unwrap(),
        Some(20)
    );
    assert_eq!(list.get(1), Some(99));
}

#[test]
fn phased_read_retains_backing_across_repeated_replacement() {
    let list = CircularList::new(4).unwrap();
    list.set(3, Some(73)).unwrap();
    let first = list.prepare_number_read();
    list.set_max_length(6).unwrap();
    let second = list.prepare_number_read();
    list.set_max_length(8).unwrap();
    assert_eq!(first.get_with::<()>(|_| Ok(3.0)).unwrap(), Some(73));
    assert_eq!(second.get_with::<()>(|_| Ok(3.0)).unwrap(), None);
}

#[test]
fn phased_read_keeps_signed_fractional_and_nan_properties() {
    let list = CircularList::new(4).unwrap();
    for (index, value) in [(-1.0, 1), (0.5, 2), (f64::NAN, 3)] {
        list.set_number(index, Some(value)).unwrap();
        let result = list
            .prepare_number_read()
            .get_with::<()>(|start| Ok(start + index));
        assert_eq!(result.unwrap(), Some(value));
    }
}

#[test]
fn phased_read_preserves_error_without_later_lookup() {
    let list = CircularList::<u8>::new(4).unwrap();
    let token = Rc::new(());
    let result = list.prepare_number_read().get_with(|_| {
        list.set_max_length(2).unwrap();
        Err(Rc::clone(&token))
    });
    assert!(Rc::ptr_eq(&result.unwrap_err(), &token));
}

#[test]
fn phased_read_keeps_removed_values_alive_only_until_read_ends() {
    let list = CircularList::new(4).unwrap();
    let value = Rc::new(Cell::new(12));
    let weak = Rc::downgrade(&value);
    list.set(3, Some(value)).unwrap();
    let read = list.prepare_number_read();
    list.set_max_length(2).unwrap();
    assert!(weak.upgrade().is_some());
    assert!(read.get_with::<()>(|_| Ok(0.0)).unwrap().is_none());
    assert!(weak.upgrade().is_none());
}
