use xterm_core::sorted_list::{InvalidArrayLength, SortedList};

#[test]
fn duplicate_pending_index_retains_later_removals_like_javascript() {
    let mut list = SortedList::new(|value: &i32| f64::from(*value));
    for value in [1, 2, 3] {
        list.insert(value).unwrap();
    }
    assert!(list.delete(&1));
    assert!(list.delete(&1));
    assert!(list.delete(&2));
    assert_eq!(list.values().unwrap().collect::<Vec<_>>(), [2, 3]);
}

#[test]
fn negative_array_length_is_a_result_and_clear_recovers_failed_flush() {
    let mut list = SortedList::new(|value: &i32| f64::from(*value));
    list.insert(1).unwrap();
    assert!(list.delete(&1));
    assert!(list.delete(&1));
    assert_eq!(list.values().unwrap_err(), InvalidArrayLength);
    assert_eq!(list.values().unwrap().collect::<Vec<_>>(), [1]);
    list.insert(2).unwrap();
    assert_eq!(list.values().unwrap().collect::<Vec<_>>(), [1, 2]);
    list.clear();
    list.insert(3).unwrap();
    assert!(list.delete(&3));
    assert_eq!(list.values().unwrap().collect::<Vec<_>>(), []);
}

#[test]
fn insertion_surfaces_the_same_pending_deletion_error() {
    let mut list = SortedList::new(|value: &i32| f64::from(*value));
    list.insert(1).unwrap();
    list.delete(&1);
    list.delete(&1);
    assert_eq!(list.insert(2), Err(InvalidArrayLength));
    assert_eq!(list.values().unwrap().collect::<Vec<_>>(), [1]);
}
