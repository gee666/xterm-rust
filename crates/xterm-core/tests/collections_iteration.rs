use xterm_core::sorted_list::{IterationContext, KeyCursor, SortedList};

#[test]
fn suspended_cursor_observes_mutations_instead_of_snapshotting() {
    let mut list = SortedList::new(|value: &(i32, char)| f64::from(value.0));
    for name in ['a', 'b', 'c'] {
        list.insert((1, name)).unwrap();
    }
    let mut cursor = KeyCursor::new(1.0);
    assert_eq!(list.next_by_key(&mut cursor).unwrap(), Some((1, 'a')));
    list.delete(&(1, 'a')).unwrap();
    list.values().unwrap();
    // Source increments the shared index after resumption. Removing the
    // yielded entry therefore skips b rather than visiting a snapshot.
    assert_eq!(list.next_by_key(&mut cursor).unwrap(), Some((1, 'c')));
    assert_eq!(list.next_by_key(&mut cursor).unwrap(), None);
}

#[test]
fn reentrant_clear_ends_traversal_and_delayed_insert_is_not_flushed() {
    let mut list = SortedList::new(|value: &(i32, char)| f64::from(value.0));
    list.insert((1, 'a')).unwrap();
    list.insert((1, 'b')).unwrap();
    let mut visited = Vec::new();
    list.for_each_by_key_reentrant(1.0, |list, value| {
        visited.push(value.1);
        list.insert((1, 'c')).unwrap();
    })
    .unwrap();
    assert_eq!(visited, ['a', 'b']);
    assert_eq!(
        list.values().unwrap().map(|v| v.1).collect::<Vec<_>>(),
        ['c', 'c', 'a', 'b']
    );
    list.for_each_by_key_reentrant(1.0, |list, _| list.clear())
        .unwrap();
    assert_eq!(list.values().unwrap().collect::<Vec<_>>(), []);
}

#[test]
fn nested_iteration_uses_the_shared_source_work_index() {
    let context = IterationContext::default();
    let mut first = SortedList::new(|value: &(i32, char)| f64::from(value.0))
        .with_iteration_context(context.clone());
    let mut second =
        SortedList::new(|value: &(i32, char)| f64::from(value.0)).with_iteration_context(context);
    for name in ['a', 'b', 'c'] {
        first.insert((1, name)).unwrap();
    }
    second.insert((1, 'x')).unwrap();
    let mut outer = KeyCursor::new(1.0);
    let mut inner = KeyCursor::new(1.0);
    assert_eq!(first.next_by_key(&mut outer).unwrap(), Some((1, 'a')));
    assert_eq!(second.next_by_key(&mut inner).unwrap(), Some((1, 'x')));
    assert_eq!(second.next_by_key(&mut inner).unwrap(), None);
    assert_eq!(first.next_by_key(&mut outer).unwrap(), Some((1, 'c')));
    assert_eq!(first.next_by_key(&mut outer).unwrap(), None);
}

#[test]
fn first_resume_is_lazy_and_observes_clear_before_iteration() {
    let mut list = SortedList::new(|value: &i32| f64::from(*value));
    list.insert(1).unwrap();
    let mut cursor = KeyCursor::new(1.0);
    list.clear();
    assert_eq!(list.next_by_key(&mut cursor).unwrap(), None);
    list.insert(1).unwrap();
    assert_eq!(list.next_by_key(&mut cursor).unwrap(), None);
}
