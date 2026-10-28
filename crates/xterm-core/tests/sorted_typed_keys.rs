use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::sorted_list::{KeyCursor, KeyItem, SortedKey, SortedList, SortedListError, TryKey};
#[derive(Clone)]
struct Entry {
    id: usize,
    key: SortedKey,
}
fn read(value: Option<&Rc<Entry>>) -> Result<SortedKey, SortedListError> {
    Ok(value.map_or(SortedKey::Undefined, |value| value.key.clone()))
}
fn entry(id: usize, key: impl Into<SortedKey>) -> Rc<Entry> {
    Rc::new(Entry {
        id,
        key: key.into(),
    })
}
#[test]
fn subtraction_sort_but_string_relational_search_and_strict_duplicates() {
    let mut list = SortedList::from_reader(TryKey::new(read));
    let ten = entry(10, "10");
    let two = entry(2, "2");
    list.insert(ten.clone()).unwrap();
    list.insert(two.clone()).unwrap();
    assert_eq!(
        list.values().unwrap().map(|e| e.id).collect::<Vec<_>>(),
        [2, 10]
    );
    // Source search is lexicographic here, despite numeric subtraction sorting.
    assert!(!list.delete_by(&ten, Rc::ptr_eq).unwrap());
    assert!(list.delete_by(&two, Rc::ptr_eq).unwrap());
    assert_eq!(
        list.values().unwrap().map(|e| e.id).collect::<Vec<_>>(),
        [10]
    );
    let mut list = SortedList::from_reader(TryKey::new(read));
    list.insert(entry(1, "2")).unwrap();
    list.insert(entry(2, 2.0)).unwrap();
    let mut number = KeyCursor::new(2.0);
    assert!(list.next_key_item(&mut number).unwrap().is_none());
    let mut string = KeyCursor::new("2");
    assert!(
        matches!(list.next_key_item(&mut string).unwrap(), Some(KeyItem::Value(e)) if e.id == 1)
    );
    assert!(list.next_key_item(&mut string).unwrap().is_none());
}
#[test]
fn comparator_failure_does_not_reorder_pending_batch_and_keeps_error_identity() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = calls.clone();
    let fail = Rc::new(Cell::new(true));
    let active = fail.clone();
    let cause: Rc<dyn std::error::Error> = Rc::new(std::io::Error::other("key failed"));
    let expected = cause.clone();
    let mut list = SortedList::from_reader(TryKey::new(move |value: Option<&Rc<Entry>>| {
        let value = value.unwrap();
        output.borrow_mut().push(value.id);
        if active.get() && output.borrow().len() == 4 {
            return Err(SortedListError::Callback(cause.clone()));
        }
        Ok(value.key.clone())
    }));
    for id in [3, 2, 1] {
        list.insert(entry(id, id as f64)).unwrap();
    }
    let error = match list.values() {
        Err(error) => error,
        Ok(_) => panic!("missing failure"),
    };
    let SortedListError::Callback(actual) = error else {
        panic!()
    };
    assert!(Rc::ptr_eq(&actual, &expected));
    assert_eq!(*calls.borrow(), [2, 3, 1, 2]);
    calls.borrow_mut().clear();
    fail.set(false);
    assert_eq!(
        list.values().unwrap().map(|e| e.id).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert_eq!(*calls.borrow(), [2, 3, 1, 2]);
}
#[test]
fn merge_undefined_read_can_throw_after_batch_sort_and_retry_keeps_order() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = calls.clone();
    let fail = Rc::new(Cell::new(false));
    let active = fail.clone();
    let mut list = SortedList::from_reader(TryKey::new(move |value: Option<&Rc<Entry>>| {
        output.borrow_mut().push(value.map(|value| value.id));
        if value.is_none() && active.get() {
            return Err(SortedListError::KeyAccess("undefined read".into()));
        }
        Ok(value.map_or(SortedKey::Undefined, |value| value.key.clone()))
    }));
    list.insert(entry(1, 1.0)).unwrap();
    list.values().unwrap().for_each(drop);
    list.insert(entry(0, 0.0)).unwrap();
    fail.set(true);
    assert!(matches!(list.values(), Err(SortedListError::KeyAccess(_))));
    assert_eq!(*calls.borrow(), [Some(0), Some(1), None]);
    calls.borrow_mut().clear();
    fail.set(false);
    assert_eq!(
        list.values().unwrap().map(|e| e.id).collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(*calls.borrow(), [Some(0), Some(1), None, Some(1)]);
}
#[test]
fn typed_cursor_closes_permanently_when_key_getter_throws() {
    let count = Rc::new(Cell::new(0));
    let calls = count.clone();
    let fail = Rc::new(Cell::new(true));
    let active = fail.clone();
    let mut list = SortedList::from_reader(TryKey::new(move |value: Option<&Rc<Entry>>| {
        calls.set(calls.get() + 1);
        if active.get() && calls.get() == 2 {
            return Err(SortedListError::KeyAccess("cursor failed".into()));
        }
        Ok(value.map_or(SortedKey::Undefined, |value| value.key.clone()))
    }));
    list.insert(entry(1, 1.0)).unwrap();
    let mut cursor = KeyCursor::new(1.0);
    assert!(list.next_key_item(&mut cursor).is_err());
    assert_eq!(count.get(), 2);
    fail.set(false);
    assert!(list.next_key_item(&mut cursor).unwrap().is_none());
    assert_eq!(count.get(), 2);
}
#[test]
fn default_context_and_getter_reentry_can_yield_undefined_without_finishing() {
    let other = Rc::new(RefCell::new(SortedList::new(|value: &f64| *value)));
    for value in 0..5 {
        other.borrow_mut().insert(f64::from(value)).unwrap();
    }
    other.borrow_mut().values().unwrap().for_each(drop);
    let nested = other.clone();
    let mut list = SortedList::from_reader(TryKey::new(move |value: Option<&Rc<Entry>>| {
        nested.borrow_mut().delete(&999.0)?;
        Ok(value.map_or(SortedKey::Undefined, |value| value.key.clone()))
    }));
    list.insert(entry(1, 1.0)).unwrap();
    let mut cursor = KeyCursor::new(1.0);
    assert!(matches!(
        list.next_key_item(&mut cursor).unwrap(),
        Some(KeyItem::Undefined)
    ));
    assert!(list.next_key_item(&mut cursor).unwrap().is_none());
}
#[test]
fn array_undefined_elements_sort_last_without_key_callback_reads() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = calls.clone();
    let mut list = SortedList::from_reader(TryKey::new(move |value: Option<&Option<f64>>| {
        output.borrow_mut().push(value.copied().flatten());
        Ok(value
            .copied()
            .flatten()
            .map_or(SortedKey::Undefined, SortedKey::number))
    }))
    .with_undefined_values(Option::is_none);
    for value in [Some(2.0), None, Some(1.0)] {
        list.insert(value).unwrap();
    }
    assert_eq!(
        list.values().unwrap().collect::<Vec<_>>(),
        [Some(1.0), Some(2.0), None]
    );
    assert_eq!(*calls.borrow(), [Some(1.0), Some(2.0)]);
}
