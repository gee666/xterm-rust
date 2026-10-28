use std::cell::Cell;
use std::rc::Rc;
use xterm_core::sorted_list::{SortedList, SortedListError};

#[test]
fn object_getter_failure_retains_old_and_pending_arrays_until_recovered() {
    let message = "Cannot read properties of undefined (reading 'key')";
    let mut list =
        SortedList::new(|value: &(i32, char)| f64::from(value.0)).with_undefined_key_error(message);
    list.insert((2, 'b')).unwrap();
    assert_eq!(list.values().unwrap().collect::<Vec<_>>(), [(2, 'b')]);
    list.insert((1, 'a')).unwrap();
    assert_eq!(
        list.values().unwrap_err(),
        SortedListError::KeyAccess(message.into())
    );
    assert_eq!(
        list.delete(&(2, 'b')),
        Err(SortedListError::KeyAccess(message.into()))
    );
    list.insert((3, 'c')).unwrap();
    assert_eq!(
        list.values().unwrap().collect::<Vec<_>>(),
        [(1, 'a'), (2, 'b'), (3, 'c')]
    );
    list.clear();
    list.insert((1, 'a')).unwrap();
    assert_eq!(list.values().unwrap().collect::<Vec<_>>(), [(1, 'a')]);
}

#[test]
fn mutable_keys_use_original_left_midpoint_not_a_resort() {
    let mut list = SortedList::new(|value: &Rc<Cell<f64>>| value.get());
    let a = Rc::new(Cell::new(1.0));
    let b = Rc::new(Cell::new(2.0));
    list.insert(a.clone()).unwrap();
    list.insert(b.clone()).unwrap();
    list.flush().unwrap();
    a.set(5.0);
    assert!(list.get_key_iterator(2.0).unwrap().next().is_none());
    assert!(!list.delete_by(&b, Rc::ptr_eq).unwrap());
    assert!(list.delete_by(&a, Rc::ptr_eq).unwrap());
    assert!(Rc::ptr_eq(&list.values().unwrap().next().unwrap(), &b));
}

#[test]
fn optional_undefined_keys_remain_distinct_from_an_ordered_key() {
    let mut list = SortedList::new(|value: &(Option<f64>, char)| value.0.unwrap_or(f64::NAN));
    list.insert((None, 'u')).unwrap();
    assert!(!list.delete(&(None, 'u')).unwrap());
    assert!(list.get_key_iterator(f64::NAN).unwrap().next().is_none());
    assert!(list.get_key_iterator(0.0).unwrap().next().is_none());
    assert_eq!(list.values().unwrap().next(), Some((None, 'u')));
}

#[test]
fn a_callback_error_retains_changes_and_terminates_without_visiting_more() {
    let mut list = SortedList::new(|value: &i32| f64::from(*value));
    list.insert(1).unwrap();
    list.insert(1).unwrap();
    let mut visited = 0;
    let result = list.try_for_each_by_key_reentrant(1.0, |list, value| {
        visited += 1;
        list.delete(&value)?;
        Err(SortedListError::KeyAccess("callback failure".into()))
    });
    assert_eq!(
        result,
        Err(SortedListError::KeyAccess("callback failure".into()))
    );
    assert_eq!(visited, 1);
    assert_eq!(list.values().unwrap().collect::<Vec<_>>(), [1]);
}
