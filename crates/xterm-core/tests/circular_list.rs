// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Assertions ported from common/CircularList.test.ts.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::circular_list::{CircularList, CircularListError};
use xterm_core::lifecycle::Disposable;

fn list<T: Clone>(capacity: usize, values: &[T]) -> CircularList<T> {
    let list = CircularList::new(capacity).unwrap();
    for value in values {
        list.push(value.clone()).unwrap();
    }
    list
}

fn contents<T: Clone + PartialEq + std::fmt::Debug>(list: &CircularList<T>, values: &[T]) {
    assert_eq!(list.len(), values.len());
    for (index, value) in values.iter().enumerate() {
        assert_eq!(list.get(index).as_ref(), Some(value));
    }
}

#[test]
fn push_values_onto_array() {
    contents(
        &list(5, &["1", "2", "3", "4", "5"]),
        &["1", "2", "3", "4", "5"],
    );
}

#[test]
fn push_discards_old_values_at_max_length() {
    let list = list(2, &["1", "2"]);
    contents(&list, &["1", "2"]);
    list.push("3").unwrap();
    contents(&list, &["2", "3"]);
    list.push("4").unwrap();
    contents(&list, &["3", "4"]);
}

#[test]
fn max_length_increases_size() {
    let list = list(2, &["1", "2"]);
    contents(&list, &["1", "2"]);
    list.set_max_length(4).unwrap();
    list.push("3").unwrap();
    list.push("4").unwrap();
    contents(&list, &["1", "2", "3", "4"]);
    list.push("wrapped").unwrap();
    contents(&list, &["2", "3", "4", "wrapped"]);
}

#[test]
fn max_length_getter() {
    let list = list(2, &[]);
    assert_eq!(list.max_length(), 2);
    list.push("1").unwrap();
    list.push("2").unwrap();
    assert_eq!(list.max_length(), 2);
    list.push("3").unwrap();
    assert_eq!(list.max_length(), 2);
    list.set_max_length(4).unwrap();
    assert_eq!(list.max_length(), 4);
}

#[test]
fn length_capped_at_maximum() {
    let list = list(2, &[]);
    assert_eq!(list.len(), 0);
    list.push("1").unwrap();
    assert_eq!(list.len(), 1);
    list.push("2").unwrap();
    assert_eq!(list.len(), 2);
    list.push("3").unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn splice_deletes_items() {
    let list = list(2, &["1", "2"]);
    list.splice(0, 1, &[]).unwrap();
    contents(&list, &["2"]);
    list.push("3").unwrap();
    list.splice(1, 1, &[]).unwrap();
    contents(&list, &["2"]);
}

#[test]
fn splice_inserts_items() {
    let list = list(2, &["1"]);
    list.splice(0, 0, &["2"]).unwrap();
    contents(&list, &["2", "1"]);
    list.splice(1, 0, &["3"]).unwrap();
    contents(&list, &["3", "1"]);
}

#[test]
fn splice_deletes_then_inserts_items() {
    let list = list(3, &["1", "2"]);
    list.splice(0, 1, &["3", "4"]).unwrap();
    contents(&list, &["3", "4", "2"]);
}

#[test]
fn splice_wraps_when_more_inserted_than_deleted() {
    let list = list(3, &["1", "2"]);
    list.splice(1, 0, &["3", "4"]).unwrap();
    contents(&list, &["3", "4", "2"]);
}

#[test]
fn trim_removes_from_beginning() {
    let list = list(5, &["1", "2", "3", "4", "5"]);
    list.trim_start(1).unwrap();
    contents(&list, &["2", "3", "4", "5"]);
    list.trim_start(2).unwrap();
    contents(&list, &["4", "5"]);
}

#[test]
fn trim_removes_all_when_count_exceeds_length() {
    let list = list(5, &["1"]);
    list.trim_start(2).unwrap();
    assert_eq!(list.len(), 0);
}

#[test]
fn shift_count_zero_does_not_mutate() {
    let list = list(5, &[1, 2]);
    list.shift_elements(0, 0, 1).unwrap();
    contents(&list, &[1, 2]);
}

#[test]
fn shift_invalid_arguments_match_upstream_errors() {
    let list = list(5, &[1]);
    for start in [-1, 1] {
        assert_eq!(
            list.shift_elements(start, 1, 1).unwrap_err().to_string(),
            "start argument out of range"
        );
    }
    assert_eq!(
        list.shift_elements(0, 1, -1).unwrap_err().to_string(),
        "Cannot shift elements in list beyond index 0"
    );
}

#[test]
fn shift_one_element_forward() {
    let list = list(5, &[1, 2]);
    list.shift_elements(0, 1, 1).unwrap();
    contents(&list, &[1, 1]);
}

#[test]
fn shift_elements_forward() {
    let list = list(5, &[1, 2, 3, 4]);
    list.shift_elements(0, 2, 2).unwrap();
    contents(&list, &[1, 2, 1, 2]);
}

#[test]
fn shift_forward_expands_list() {
    let list = list(5, &[1, 2]);
    list.shift_elements(0, 2, 2).unwrap();
    contents(&list, &[1, 2, 1, 2]);
}

#[test]
fn shift_forward_wraps_list() {
    let list = list(5, &[1, 2, 3, 4, 5]);
    list.shift_elements(2, 2, 3).unwrap();
    contents(&list, &[3, 4, 5, 3, 4]);
}

#[test]
fn shift_one_element_backwards() {
    let list = list(5, &[1, 2]);
    list.shift_elements(1, 1, -1).unwrap();
    contents(&list, &[2, 2]);
}

#[test]
fn shift_elements_backwards() {
    let list = list(5, &[1, 2, 3, 4]);
    list.shift_elements(2, 2, -2).unwrap();
    contents(&list, &[3, 4, 3, 4]);
}

#[test]
fn recycle_retains_shared_identity_and_requires_full_buffer() {
    let list = CircularList::new(2).unwrap();
    let first = Rc::new(Cell::new(1));
    list.push(Rc::clone(&first)).unwrap();
    assert_eq!(list.recycle().unwrap_err(), CircularListError::NotFull);
    list.push(Rc::new(Cell::new(2))).unwrap();
    let recycled = list.recycle().unwrap().unwrap();
    assert!(Rc::ptr_eq(&recycled, &first));
    recycled.set(3);
    assert_eq!(list.get(1).unwrap().get(), 3);
}

#[test]
fn emits_delete_insert_trim_in_order() {
    let list = list(3, &[1, 2, 3]);
    let events = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&events);
    list.on_delete_emitter.event().subscribe(move |change| {
        output
            .borrow_mut()
            .push(format!("delete:{}:{}", change.index, change.amount))
    });
    let output = Rc::clone(&events);
    list.on_insert_emitter.event().subscribe(move |change| {
        output
            .borrow_mut()
            .push(format!("insert:{}:{}", change.index, change.amount))
    });
    let output = Rc::clone(&events);
    list.on_trim_emitter
        .event()
        .subscribe(move |amount| output.borrow_mut().push(format!("trim:{amount}")));
    list.splice(1, 1, &[4, 5]).unwrap();
    assert_eq!(*events.borrow(), ["delete:1:1", "insert:1:2", "trim:1"]);
    list.trim_start(0).unwrap();
    assert_eq!(events.borrow().last().unwrap(), "trim:0");
    list.dispose();
    list.push(6).unwrap();
    assert_eq!(events.borrow().len(), 4);
}

#[test]
fn length_growth_clears_physical_slots_and_capacity_shrink_retains_length() {
    let list = list(3, &[1, 2, 3, 4]);
    list.set_length(1);
    list.set_length(3);
    assert_eq!(list.get(0), None);
    assert_eq!(list.get(1), None);
    assert_eq!(list.get(2), Some(4));
    list.set_max_length(2).unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(2), None);
    list.set_max_length(2).unwrap();
}

#[test]
fn pop_keeps_slot_and_boundary_validation_is_explicit() {
    let list = list(2, &[1]);
    assert_eq!(list.pop().unwrap(), Some(1));
    assert_eq!(list.get(0), Some(1));
    assert_eq!(list.pop(), Err(CircularListError::InvalidRange));
    assert_eq!(list.set_max_length(0), Err(CircularListError::ZeroCapacity));
    assert!(matches!(
        CircularList::<i32>::new(0),
        Err(CircularListError::ZeroCapacity)
    ));
}

#[test]
fn callbacks_can_mutate_same_list_without_borrow_conflicts() {
    let list = Rc::new(list(3, &[1, 2, 3]));
    let weak = Rc::downgrade(&list);
    list.on_delete_emitter
        .event()
        .subscribe(move |_| weak.upgrade().unwrap().push(9).unwrap());
    list.splice(1, 1, &[4]).unwrap();
    contents(&list, &[4, 3, 9]);
}
