//! Source splice permits sparse insertion and deletion spans beyond the tail.
use std::cell::RefCell;
use std::rc::Rc;
use xterm_core::circular_list::CircularList;

fn list(capacity: usize, values: &[i32]) -> Rc<CircularList<i32>> {
    let list = Rc::new(CircularList::new(capacity).unwrap());
    for &value in values {
        list.push(value).unwrap();
    }
    list
}

#[test]
fn start_beyond_length_writes_sparse_slot_without_extending_to_start() {
    let list = list(12, &[1, 2, 3]);
    list.splice(7, 0, &[9]).unwrap();
    assert_eq!(list.len(), 4);
    assert_eq!(list.get(0), Some(1));
    assert_eq!(list.get(2), Some(3));
    assert_eq!(list.get(3), None);
    assert_eq!(list.get(7), Some(9));
}

#[test]
fn deletion_count_is_not_clamped_to_the_remaining_tail() {
    let list = list(8, &[1, 2, 3, 4, 5]);
    list.splice(4, 3, &[]).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0), Some(1));
    assert_eq!(list.get(1), Some(2));
    assert_eq!(list.get(4), Some(5));
}

#[test]
fn delete_beyond_tail_then_insert_uses_source_length_and_event_order() {
    let list = list(8, &[1, 2, 3]);
    let events = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&events);
    let target = Rc::downgrade(&list);
    list.on_delete_emitter.event().subscribe(move |change| {
        output.borrow_mut().push((
            "delete",
            change.index,
            change.amount,
            target.upgrade().unwrap().len(),
        ));
    });
    let output = Rc::clone(&events);
    let target = Rc::downgrade(&list);
    list.on_insert_emitter.event().subscribe(move |change| {
        output.borrow_mut().push((
            "insert",
            change.index,
            change.amount,
            target.upgrade().unwrap().len(),
        ));
    });
    list.splice(7, 1, &[9]).unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(2), Some(3));
    assert_eq!(list.get(7), Some(9));
    assert_eq!(*events.borrow(), [("delete", 7, 1, 2), ("insert", 7, 1, 2)]);
}

#[test]
fn callback_reset_leaves_cached_start_sparse_like_terminal_insert_lines() {
    let list = list(12, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let target = Rc::downgrade(&list);
    list.on_delete_emitter.event().subscribe(move |_| {
        let list = target.upgrade().unwrap();
        list.set_length(3);
        list.set_max_length(13).unwrap();
    });
    list.splice(7, 1, &[99]).unwrap();
    assert_eq!(list.len(), 4);
    assert_eq!(list.get(3), None);
    assert_eq!(list.get(7), Some(99));
}
