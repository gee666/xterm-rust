use std::{
    cell::Cell,
    panic::{catch_unwind, AssertUnwindSafe},
    rc::Rc,
};
use xterm_core::circular_list::CircularList;

struct OnDrop(Option<Box<dyn FnOnce()>>);
impl Drop for OnDrop {
    fn drop(&mut self) {
        self.0.take().unwrap()();
    }
}
type List = CircularList<Rc<OnDrop>>;

fn value(action: impl FnOnce() + 'static) -> Rc<OnDrop> {
    Rc::new(OnDrop(Some(Box::new(action))))
}
fn inert() -> Rc<OnDrop> {
    value(|| {})
}

#[test]
fn replacing_and_removing_slots_allows_cleanup_to_read_and_write() {
    for operation in 0..6 {
        let list = Rc::new(List::new(3).unwrap());
        let weak = Rc::downgrade(&list);
        let called = Rc::new(Cell::new(false));
        let seen = called.clone();
        list.set(
            0,
            Some(value(move || {
                let list = weak.upgrade().unwrap();
                assert_eq!(list.max_length(), 3);
                assert_eq!(list.get(0).is_some(), operation % 2 == 0);
                list.set(1, Some(inert())).unwrap();
                seen.set(true);
            })),
        )
        .unwrap();
        let replacement = (operation % 2 == 0).then(inert);
        match operation {
            0 | 1 => list.set(0, replacement).unwrap(),
            2 | 3 => list.set_signed(0, replacement).unwrap(),
            4 | 5 => list.set_number(0.0, replacement).unwrap(),
            _ => unreachable!(),
        }
        assert!(called.get());
        assert!(list.get(1).is_some());
    }
}

#[test]
fn expanding_length_publishes_all_removals_before_cleanup() {
    let list = Rc::new(List::new(4).unwrap());
    let weak = Rc::downgrade(&list);
    let called = Rc::new(Cell::new(false));
    let seen = called.clone();
    list.set(1, Some(inert())).unwrap();
    list.set(
        0,
        Some(value(move || {
            let list = weak.upgrade().unwrap();
            assert_eq!(list.len(), 2);
            assert!(list.get(0).is_none());
            assert!(list.get(1).is_none());
            list.set(1, Some(inert())).unwrap();
            list.set_length(3);
            seen.set(true);
        })),
    )
    .unwrap();
    list.set_length(2);
    assert!(called.get());
    assert_eq!(list.len(), 3);
    assert!(list.get(1).is_some());
}

#[test]
fn capacity_change_publishes_copied_slots_before_cleanup() {
    for capacity in [2, 6] {
        let list = Rc::new(List::new(4).unwrap());
        let retained = inert();
        list.push(retained.clone()).unwrap();
        let weak = Rc::downgrade(&list);
        let expected = retained.clone();
        let called = Rc::new(Cell::new(false));
        let seen = called.clone();
        // Stale physical slots are discarded, not copied by a capacity change.
        list.set(
            3,
            Some(value(move || {
                let list = weak.upgrade().unwrap();
                assert_eq!(list.max_length(), capacity);
                assert_eq!(list.len(), 1);
                assert!(Rc::ptr_eq(&list.get(0).unwrap(), &expected));
                assert!(list.get(1).is_none());
                list.push(inert()).unwrap();
                seen.set(true);
            })),
        )
        .unwrap();
        list.set_max_length(capacity).unwrap();
        assert!(called.get());
        assert_eq!(list.len(), 2);
        assert!(list.get(1).is_some());
    }
}

#[test]
fn cleanup_panic_keeps_committed_storage_reusable() {
    for operation in 0..3 {
        let list = Rc::new(List::new(4).unwrap());
        let weak = Rc::downgrade(&list);
        list.set(
            1,
            Some(value(move || {
                let list = weak.upgrade().unwrap();
                list.set(0, Some(inert())).unwrap();
                panic!("circular value cleanup failed");
            })),
        )
        .unwrap();
        let error = catch_unwind(AssertUnwindSafe(|| match operation {
            0 => list.set(1, None).unwrap(),
            1 => list.set_length(2),
            2 => list.set_max_length(2).unwrap(),
            _ => unreachable!(),
        }))
        .unwrap_err();
        assert_eq!(
            error.downcast_ref::<&str>(),
            Some(&"circular value cleanup failed")
        );
        assert!(list.get(0).is_some());
        assert!(list.get(1).is_none());
        assert_eq!(list.max_length(), if operation == 2 { 2 } else { 4 });
        assert_eq!(list.len(), if operation == 1 { 2 } else { 0 });
        list.set(1, Some(inert())).unwrap();
        assert!(list.get(1).is_some());
    }
}

#[test]
fn push_releases_overwritten_value_before_trim_event() {
    let list = Rc::new(List::new(1).unwrap());
    let weak = Rc::downgrade(&list);
    let dropped = Rc::new(Cell::new(false));
    let seen = dropped.clone();
    list.push(value(move || {
        let list = weak.upgrade().unwrap();
        assert_eq!(list.len(), 1);
        assert!(list.get(0).is_some());
        seen.set(true);
    }))
    .unwrap();
    let trimmed = Rc::new(Cell::new(0));
    let events = trimmed.clone();
    list.on_trim_emitter.event().subscribe(move |amount| {
        assert!(dropped.get());
        events.set(events.get() + amount);
    });
    list.push(inert()).unwrap();
    assert_eq!(trimmed.get(), 1);
}
