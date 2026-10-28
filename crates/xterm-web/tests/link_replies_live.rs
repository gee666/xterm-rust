use std::{cell::Cell, rc::Rc};
use xterm_web::links::{Position, ProviderReplies, ProviderRequest};
fn request() -> ProviderRequest {
    ProviderRequest::new(Position { x: 5, y: 1 })
}
#[test]
fn matching_reads_live_payload_after_provider_reply() {
    let table = ProviderReplies::<Cell<usize>>::default();
    let request = request();
    let link = Rc::new(Cell::new(1));
    assert!(table
        .try_receive_unfiltered(
            0,
            Some(vec![link.clone()]),
            &request,
            1,
            |entry| Ok::<_, ()>(entry.is_some_and(|entry| entry.get() == 5))
        )
        .unwrap()
        .is_none());
    link.set(5);
    assert!(Rc::ptr_eq(
        &table
            .try_cached_index(0, &request, 1, |entry| Ok::<_, ()>(
                entry.is_some_and(|entry| entry.get() == 5)
            ))
            .unwrap()
            .unwrap(),
        &link
    ));
}
#[test]
fn probe_can_mutate_reply_array_without_held_borrow_and_missing_slot_propagates() {
    let table = ProviderReplies::<usize>::default();
    let request = request();
    table
        .try_receive_unfiltered(0, Some(vec![Rc::new(1), Rc::new(2)]), &request, 1, |_| {
            Ok::<_, &str>(false)
        })
        .unwrap();
    let group = table.group(0).unwrap().unwrap();
    let calls = Cell::new(0);
    let result = table.try_cached_index(0, &request, 1, |entry| {
        calls.set(calls.get() + 1);
        if calls.get() == 1 {
            group.borrow_mut().remove(0);
        }
        if entry.is_none() {
            Err("missing slot")
        } else {
            Ok(false)
        }
    });
    assert_eq!(result, Err("missing slot"));
    assert_eq!(calls.get(), 2);
}
#[test]
fn overlap_traversal_retains_old_array_after_map_entry_replacement() {
    let table = ProviderReplies::<usize>::default();
    let request = request();
    table
        .try_receive_unfiltered(
            0,
            Some(vec![Rc::new(1), Rc::new(2), Rc::new(3)]),
            &request,
            1,
            |_| Ok::<_, ()>(false),
        )
        .unwrap();
    let old = table.group(0).unwrap().unwrap();
    table
        .try_filter_complete(1, |link| {
            if **link == 1 {
                table.try_receive_unfiltered(0, Some(vec![Rc::new(99)]), &request, 1, |_| {
                    Ok::<_, ()>(false)
                })?;
                Ok::<_, ()>(false)
            } else {
                Ok(true)
            }
        })
        .unwrap();
    assert_eq!(
        old.borrow().iter().map(|link| **link).collect::<Vec<_>>(),
        vec![2, 3]
    );
    assert_eq!(
        table
            .links(0)
            .unwrap()
            .iter()
            .map(|link| **link)
            .collect::<Vec<_>>(),
        vec![99]
    );
}
#[test]
fn normal_return_restores_local_selection_flag_after_reentrant_reply() {
    let table = ProviderReplies::<usize>::default();
    let request = request();
    let reentered = Cell::new(false);
    table
        .try_receive_unfiltered(0, Some(vec![Rc::new(1)]), &request, 2, |entry| {
            if entry.is_some_and(|link| **link == 1) && !reentered.replace(true) {
                let nested = table.try_receive_unfiltered(
                    1,
                    Some(vec![Rc::new(2)]),
                    &request,
                    2,
                    |entry| Ok::<_, ()>(entry.is_some_and(|link| **link == 2)),
                )?;
                assert_eq!(nested.as_deref(), Some(&2));
            }
            Ok::<_, ()>(false)
        })
        .unwrap();
    let result = table
        .try_cached_index(0, &request, 2, |entry| {
            Ok::<_, ()>(entry.is_some_and(|link| **link == 2))
        })
        .unwrap();
    assert_eq!(result.as_deref(), Some(&2));
}
