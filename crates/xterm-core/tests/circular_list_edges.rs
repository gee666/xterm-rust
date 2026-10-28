//! Named L1-L5 boundary and reentrancy cases, beyond upstream's 19 cases.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::circular_list::{CircularList, CircularListError};
use xterm_core::lifecycle::Disposable;

fn list(values: &[i32]) -> Rc<CircularList<i32>> {
    let list = Rc::new(CircularList::new(values.len()).unwrap());
    for &value in values {
        list.push(value).unwrap();
    }
    list
}

#[test]
fn sparse_source_maximum_capacity_and_length_do_not_allocate_all_holes() {
    let capacity = usize::try_from(u32::MAX).unwrap();
    let list = CircularList::new(capacity).unwrap();
    assert_eq!(list.len(), 0);
    list.set(capacity - 1, Some(7)).unwrap();
    assert_eq!(list.get(capacity - 1), Some(7));
    list.set_length(capacity);
    assert_eq!(list.get(capacity - 1), None);
    list.set(capacity - 1, Some(9)).unwrap();
    assert_eq!(list.get(capacity - 1), Some(9));
    assert!(list.is_full());
    assert_eq!(list.get(usize::MAX), list.get(usize::MAX % capacity));
    list.set_max_length(3).unwrap();
    assert_eq!(list.len(), capacity);
    assert_eq!(list.get(0), None);
}

#[test]
fn capacity_growing_after_shrink_repeats_cyclic_values_and_preserves_holes() {
    let list = list(&[1, 2, 3, 4]);
    list.set_max_length(2).unwrap();
    assert_eq!(list.len(), 4);
    list.set_max_length(5).unwrap();
    assert_eq!(
        (0..5).map(|index| list.get(index)).collect::<Vec<_>>(),
        [Some(1), Some(2), Some(1), Some(2), None]
    );
    list.set(1, None).unwrap();
    list.set_max_length(2).unwrap();
    list.set_max_length(5).unwrap();
    assert_eq!(
        (0..5).map(|index| list.get(index)).collect::<Vec<_>>(),
        [Some(1), None, Some(1), None, None]
    );
}

#[test]
fn signed_slots_preserve_js_remainder_and_unwrapped_trim_start() {
    let list = list(&[1, 2, 3, 4, 5]);
    list.set_signed(-1, Some(99)).unwrap();
    assert_eq!(list.get_signed(-1), Some(99));
    assert_eq!(list.get(4), Some(5));
    list.trim_start(1).unwrap();
    assert_eq!(list.get_signed(-1), Some(1));
    list.set_signed(-1, Some(88)).unwrap();
    list.trim_start(5).unwrap();
    assert_eq!(list.get_signed(-1), Some(5));
    for value in 6..=11 {
        list.push(value).unwrap();
    }
    assert_eq!(list.get_signed(-2), Some(99));
    list.set_max_length(6).unwrap();
    assert_eq!(list.get_signed(-1), None);
}

#[test]
fn reentrant_recycle_reads_after_observer_length_change() {
    let list = list(&[1, 2]);
    let target = Rc::downgrade(&list);
    list.on_trim_emitter
        .event()
        .subscribe(move |_| target.upgrade().unwrap().set_length(0));
    assert_eq!(list.recycle().unwrap(), Some(1));
    assert!(list.is_empty());
}

#[test]
fn reentrant_push_observer_can_resize_without_storage_borrow() {
    let list = list(&[1, 2]);
    let target = Rc::downgrade(&list);
    list.on_trim_emitter
        .event()
        .subscribe(move |_| target.upgrade().unwrap().set_max_length(1).unwrap());
    list.push(3).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.max_length(), 1);
    assert_eq!(list.get(0), Some(2));
    assert_eq!(list.get(1), Some(2));
}

#[test]
fn insert_observer_trim_changes_outer_length_adjustment() {
    let list = list(&[1, 2, 3]);
    list.set_length(2);
    let target = Rc::downgrade(&list);
    list.on_insert_emitter
        .event()
        .subscribe(move |_| target.upgrade().unwrap().trim_start(1).unwrap());
    list.splice(1, 0, &[3]).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0), Some(3));
    assert_eq!(list.get(1), Some(2));
}

#[test]
fn delete_observer_length_reset_precedes_insert() {
    let list = list(&[1, 2, 3]);
    let target = Rc::downgrade(&list);
    list.on_delete_emitter
        .event()
        .subscribe(move |_| target.upgrade().unwrap().set_length(0));
    list.splice(1, 1, &[4]).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list.get(0), Some(1));
    assert_eq!(list.get(1), Some(4));
}

#[test]
fn shift_overflow_observer_can_shorten_before_next_trim() {
    let list = list(&[1, 2, 3]);
    let target = Rc::downgrade(&list);
    let count = Rc::new(Cell::new(0));
    let calls = Rc::clone(&count);
    list.on_trim_emitter.event().subscribe(move |amount| {
        assert_eq!(*amount, 1);
        calls.set(calls.get() + 1);
        target.upgrade().unwrap().set_length(2);
    });
    list.shift_elements(0, 1, 5).unwrap();
    assert_eq!(count.get(), 1);
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0), Some(2));
    assert_eq!(list.get(1), Some(1));
}

#[test]
fn multi_trim_reports_each_partial_state_and_zero_count_skips_validation() {
    let list = list(&[1, 2, 3]);
    list.shift_elements(-1, 0, isize::MIN).unwrap();
    let target = Rc::downgrade(&list);
    let states = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&states);
    list.on_trim_emitter.event().subscribe(move |amount| {
        let list = target.upgrade().unwrap();
        output.borrow_mut().push((*amount, list.len(), list.get(0)));
    });
    list.shift_elements(0, 1, 5).unwrap();
    assert_eq!(
        *states.borrow(),
        [(1, 5, Some(2)), (1, 4, Some(1)), (1, 3, Some(1))]
    );
    list.shift_elements(0, 4, 0).unwrap();
    assert_eq!(list.len(), 3);
}

#[test]
fn event_error_interrupts_delete_insert_and_trim_at_source_statements() {
    for family in ["delete", "insert", "trim"] {
        let list = list(&[1, 2, 3]);
        let target = Rc::downgrade(&list);
        let emitter = match family {
            "delete" => &list.on_delete_emitter,
            "insert" => &list.on_insert_emitter,
            _ => {
                let target = Rc::downgrade(&list);
                list.on_trim_emitter
                    .event()
                    .subscribe(move |_| target.upgrade().unwrap().on_trim_emitter.dispose());
                list.on_trim_emitter
                    .event()
                    .subscribe(|_| panic!("after disposed array"));
                assert!(matches!(list.push(4), Err(CircularListError::Dispatch(_))));
                assert_eq!(list.get(0), Some(2));
                assert_eq!(list.len(), 3);
                list.push(5).unwrap();
                assert_eq!(list.get(0), Some(3));
                continue;
            }
        };
        emitter.event().subscribe(move |_| {
            let list = target.upgrade().unwrap();
            if family == "delete" {
                list.on_delete_emitter.dispose();
            } else {
                list.on_insert_emitter.dispose();
            }
        });
        emitter
            .event()
            .subscribe(|_| panic!("after disposed array"));
        assert!(matches!(
            list.splice(1, 1, &[4]),
            Err(CircularListError::Dispatch(_))
        ));
        assert_eq!(list.len(), 2);
        assert_eq!(list.get(0), Some(1));
        assert_eq!(list.get(1), Some(if family == "delete" { 3 } else { 4 }));
        list.splice(1, 1, &[]).unwrap();
        assert_eq!(list.len(), 1);
    }
}

#[test]
fn arithmetic_errors_precede_storage_mutation() {
    let list = list(&[1, 2]);
    list.set_length(usize::MAX);
    assert_eq!(list.push(3), Err(CircularListError::SizeOverflow));
    assert_eq!(
        list.splice(0, 0, &[4]),
        Err(CircularListError::SizeOverflow)
    );
    assert_eq!(
        list.shift_elements(0, usize::MAX, 1),
        Err(CircularListError::SizeOverflow)
    );
    assert_eq!(list.len(), usize::MAX);
    // Growth clears physical slots starting at old length, retaining slots 0/1.
    assert_eq!(list.get(0), Some(1));
    assert_eq!(list.get(1), Some(2));
}

#[test]
fn impossible_sparse_resize_reports_reserve_failure_atomically() {
    let list = list(&[7]);
    list.set_length(usize::MAX);
    assert_eq!(
        list.set_max_length(usize::MAX),
        Err(CircularListError::AllocationFailed)
    );
    assert_eq!(list.max_length(), 1);
    assert_eq!(list.len(), usize::MAX);
    assert_eq!(list.get(0), Some(7));
}
