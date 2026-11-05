use std::{cell::Cell, rc::Rc};
use xterm_web::links::{Position, ProviderReplies, ProviderRequest};

fn request() -> ProviderRequest {
    ProviderRequest::new(Position { x: 4, y: 1 })
}

#[test]
fn selection_reentry_observes_unassigned_outer_result() {
    let table = ProviderReplies::<u8>::default();
    let request = request();
    let selections = Cell::new(0);
    table
        .try_receive_selecting(
            0,
            Some(vec![Rc::new(1)]),
            &request,
            2,
            |_| Ok::<_, ()>(true),
            |_| {
                selections.set(selections.get() + 1);
                table.try_receive_selecting(
                    1,
                    None,
                    &request,
                    2,
                    |_| Ok(true),
                    |_| {
                        selections.set(selections.get() + 1);
                        Ok(())
                    },
                )?;
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(selections.get(), 2);
    // Both successful calls have now assigned true. A later lower reply must
    // not reselect the higher link through the all-providers fallback.
    let selected = table
        .try_receive_selecting(
            1,
            None,
            &request,
            2,
            |_| Ok::<_, ()>(true),
            |_| panic!("already provided request selected again"),
        )
        .unwrap();
    assert!(selected.is_none());
}

#[test]
fn thrown_selection_does_not_commit_shared_result() {
    let table = ProviderReplies::<u8>::default();
    let request = request();
    let marker = Rc::new(7);
    let error = table
        .try_receive_selecting(
            0,
            Some(vec![Rc::new(1)]),
            &request,
            2,
            |_| Ok(true),
            |_| Err(marker.clone()),
        )
        .unwrap_err();
    assert!(Rc::ptr_eq(&error, &marker));
    let selections = Cell::new(0);
    let selected = table
        .try_receive_selecting(
            1,
            None,
            &request,
            2,
            |_| Ok::<_, ()>(true),
            |_| {
                selections.set(selections.get() + 1);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(selected.as_deref(), Some(&1));
    assert_eq!(selections.get(), 1);
}

#[test]
fn caught_nested_selection_failure_does_not_cancel_outer_commit() {
    let table = ProviderReplies::<u8>::default();
    let request = request();
    let marker = "nested hover";
    table
        .try_receive_selecting(
            0,
            Some(vec![Rc::new(1)]),
            &request,
            2,
            |_| Ok::<_, &str>(true),
            |_| {
                let nested = table.try_receive_selecting(
                    1,
                    None,
                    &request,
                    2,
                    |_| Ok(true),
                    |_| Err(marker),
                );
                assert_eq!(nested.unwrap_err(), marker);
                Ok(())
            },
        )
        .unwrap();
    assert!(table
        .try_receive_selecting(
            1,
            None,
            &request,
            2,
            |_| Ok::<_, ()>(true),
            |_| { panic!("successful outer transaction was not committed") }
        )
        .unwrap()
        .is_none());
}

#[test]
fn cached_selection_uses_the_same_transaction_boundary() {
    let table = ProviderReplies::<u8>::default();
    table
        .try_receive_unfiltered(0, Some(vec![Rc::new(1)]), &request(), 2, |_| {
            Ok::<_, ()>(false)
        })
        .unwrap();
    let request = request();
    let selections = Cell::new(0);
    table
        .try_cached_selecting(
            0,
            &request,
            2,
            |_| Ok::<_, ()>(true),
            |_| {
                selections.set(selections.get() + 1);
                table.try_receive_selecting(
                    1,
                    None,
                    &request,
                    2,
                    |_| Ok(true),
                    |_| {
                        selections.set(selections.get() + 1);
                        Ok(())
                    },
                )?;
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(selections.get(), 2);
}
