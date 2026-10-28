// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
// Assertions ported from common/SortedList.test.ts and MultiKeyMap.test.ts.
use xterm_core::multi_key_map::{FourKeyMap, TwoKeyMap};
use xterm_core::sorted_list::SortedList;

#[test]
fn sorted_insert_maintains_values() {
    let mut list = SortedList::new(|e: &i32| f64::from(*e));
    for (value, expected) in [
        (10, vec![10]),
        (8, vec![8, 10]),
        (15, vec![8, 10, 15]),
        (2, vec![2, 8, 10, 15]),
        (1, vec![1, 2, 8, 10, 15]),
        (6, vec![1, 2, 6, 8, 10, 15]),
    ] {
        list.insert(value).unwrap();
        assert_eq!(list.values().unwrap().collect::<Vec<_>>(), expected);
    }
}

#[test]
fn sorted_insert_duplicate_keys() {
    let mut list = SortedList::new(|e: &i32| f64::from(*e));
    for (value, expected) in [
        (5, vec![5]),
        (5, vec![5, 5]),
        (8, vec![5, 5, 8]),
        (5, vec![5, 5, 5, 8]),
        (8, vec![5, 5, 5, 8, 8]),
        (6, vec![5, 5, 5, 6, 8, 8]),
    ] {
        list.insert(value).unwrap();
        assert_eq!(list.values().unwrap().collect::<Vec<_>>(), expected);
    }
}

#[test]
fn sorted_delete() {
    let mut list = SortedList::new(|e: &i32| f64::from(*e));
    for value in [1, 2, 4, 3, 5] {
        list.insert(value).unwrap();
    }
    assert_eq!(list.values().unwrap().collect::<Vec<_>>(), [1, 2, 3, 4, 5]);
    for (value, expected) in [
        (1, vec![2, 3, 4, 5]),
        (3, vec![2, 4, 5]),
        (4, vec![2, 5]),
        (5, vec![2]),
        (2, vec![]),
    ] {
        list.delete(&value).unwrap();
        assert_eq!(list.values().unwrap().collect::<Vec<_>>(), expected);
    }
}

#[test]
fn sorted_key_iterator() {
    let mut list = SortedList::new(|e: &i32| f64::from(*e));
    for value in [5, 5, 8, 5, 8, 6] {
        list.insert(value).unwrap();
    }
    assert_eq!(
        list.values().unwrap().collect::<Vec<_>>(),
        [5, 5, 5, 6, 8, 8]
    );
    for (key, expected) in [
        (1, vec![]),
        (5, vec![5, 5, 5]),
        (6, vec![6]),
        (8, vec![8, 8]),
        (9, vec![]),
    ] {
        assert_eq!(
            list.get_key_iterator(f64::from(key))
                .unwrap()
                .copied()
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn sorted_clear() {
    let mut list = SortedList::new(|e: &i32| f64::from(*e));
    for value in [1, 2, 4, 3, 5] {
        list.insert(value).unwrap();
    }
    list.clear();
    assert_eq!(list.values().unwrap().collect::<Vec<_>>(), []);
}

#[test]
fn sorted_custom_key() {
    #[derive(Clone, Debug, PartialEq)]
    struct Entry {
        key: i32,
    }
    let mut list = SortedList::new(|e: &Entry| f64::from(e.key));
    for key in [5, 2, 10, 5, 6] {
        list.insert(Entry { key }).unwrap();
    }
    assert_eq!(
        list.values().unwrap().collect::<Vec<_>>(),
        [2, 5, 5, 6, 10].map(|key| Entry { key })
    );
}

#[test]
fn sorted_values_snapshot_survives_mutation() {
    let mut list = SortedList::new(|e: &i32| f64::from(*e));
    for value in [1, 2, 3, 4] {
        list.insert(value).unwrap();
    }
    let mut visited = Vec::new();
    for value in list.values().unwrap() {
        visited.push(value);
        list.delete(&value).unwrap();
    }
    assert_eq!(visited, [1, 2, 3, 4]);
}

#[test]
fn two_key_set_get() {
    let mut map = TwoKeyMap::new();
    assert_eq!(map.get(1, 2), None);
    map.set(1, 2, "foo");
    assert_eq!(map.get(1, 2), Some(&"foo"));
    map.set(1, 3, "bar");
    assert_eq!(map.get(1, 2), Some(&"foo"));
    assert_eq!(map.get(1, 3), Some(&"bar"));
    map.set(2, 2, "foo2");
    map.set(2, 3, "bar2");
    assert_eq!(map.get(1, 2), Some(&"foo"));
    assert_eq!(map.get(1, 3), Some(&"bar"));
    assert_eq!(map.get(2, 2), Some(&"foo2"));
    assert_eq!(map.get(2, 3), Some(&"bar2"));
}

#[test]
fn two_key_clear() {
    let mut map = TwoKeyMap::new();
    assert_eq!(map.get(1, 2), None);
    map.set(1, 2, "foo");
    assert_eq!(map.get(1, 2), Some(&"foo"));
    map.clear();
    assert_eq!(map.get(1, 2), None);
}

#[test]
fn four_key_set_get() {
    let mut map = FourKeyMap::new();
    assert_eq!(map.get(1, 2, 3, 4), None);
    map.set(1, 2, 3, 4, "foo");
    assert_eq!(map.get(1, 2, 3, 4), Some(&"foo"));
    map.set(1, 3, 3, 4, "bar");
    assert_eq!(map.get(1, 2, 3, 4), Some(&"foo"));
    assert_eq!(map.get(1, 3, 3, 4), Some(&"bar"));
    map.set(2, 2, 3, 4, "foo2");
    map.set(2, 3, 3, 4, "bar2");
    assert_eq!(map.get(1, 2, 3, 4), Some(&"foo"));
    assert_eq!(map.get(1, 3, 3, 4), Some(&"bar"));
    assert_eq!(map.get(2, 2, 3, 4), Some(&"foo2"));
    assert_eq!(map.get(2, 3, 3, 4), Some(&"bar2"));
}

#[test]
fn four_key_clear() {
    let mut map = FourKeyMap::new();
    assert_eq!(map.get(1, 2, 3, 4), None);
    map.set(1, 2, 3, 4, "foo");
    assert_eq!(map.get(1, 2, 3, 4), Some(&"foo"));
    map.clear();
    assert_eq!(map.get(1, 2, 3, 4), None);
}

#[test]
fn map_number_and_string_keys_are_the_same_property() {
    let mut map = TwoKeyMap::new();
    for (number, string) in [
        (-0.0, "0"),
        (f64::NAN, "NaN"),
        (f64::INFINITY, "Infinity"),
        (f64::NEG_INFINITY, "-Infinity"),
        (1e21, "1e+21"),
        (1e-7, "1e-7"),
        (1e-6, "0.000001"),
        (4294967295.0, "4294967295"),
    ] {
        map.set(number, "key", "value");
        assert_eq!(map.get(string, "key"), Some(&"value"));
        map.clear();
    }
    map.set("1", "2", "old");
    map.set(1, 2, "new");
    assert_eq!(map.get("1", "2"), Some(&"new"));
}

#[test]
fn sorted_identity_and_batch_order() {
    use std::rc::Rc;
    let mut list = SortedList::new(|e: &Rc<(i32, &'static str)>| f64::from(e.0));
    let a = Rc::new((1, "a"));
    let b = Rc::new((1, "b"));
    let c = Rc::new((1, "c"));
    list.insert(a.clone()).unwrap();
    list.insert(b.clone()).unwrap();
    assert_eq!(
        list.values().unwrap().map(|v| v.1).collect::<Vec<_>>(),
        ["a", "b"]
    );
    list.insert(c.clone()).unwrap();
    assert_eq!(
        list.values().unwrap().map(|v| v.1).collect::<Vec<_>>(),
        ["c", "a", "b"]
    );
    assert!(!list.delete_by(&Rc::new((1, "a")), Rc::ptr_eq).unwrap());
    assert!(list.delete_by(&a, Rc::ptr_eq).unwrap());
    assert_eq!(
        list.values().unwrap().map(|v| v.1).collect::<Vec<_>>(),
        ["c", "b"]
    );
    let mut visited = Vec::new();
    list.for_each_by_key(1.0, |v| visited.push(v.1)).unwrap();
    assert_eq!(visited, ["c", "b"]);
    assert!(!list.delete_by(&a, Rc::ptr_eq).unwrap());
}

#[test]
fn sorted_clear_cancels_pending_deletes_and_inserts() {
    let mut list = SortedList::new(|e: &i32| f64::from(*e));
    list.insert(1).unwrap();
    assert!(list.delete(&1).unwrap());
    list.clear();
    list.insert(2).unwrap();
    assert!(!list.delete(&3).unwrap());
    assert_eq!(list.values().unwrap().collect::<Vec<_>>(), [2]);
}
