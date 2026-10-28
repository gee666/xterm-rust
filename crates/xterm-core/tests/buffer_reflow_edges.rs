// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{reflow::*, BufferLine, CellData, CharData},
    circular_list::{CircularList, ListChange},
    event::EmitError,
    lifecycle::Disposable,
};
fn cell(text: &str, width: i32) -> CellData {
    CellData::from_char_data(&CharData {
        attr: 0,
        chars: text.encode_utf16().collect(),
        width,
        code: None,
    })
}
fn list() -> Rc<CircularList<Rc<RefCell<BufferLine>>>> {
    let lines = Rc::new(CircularList::new(8).unwrap());
    for i in 0..8 {
        lines
            .push(Rc::new(RefCell::new(BufferLine::new(
                6,
                Some(&cell(&i.to_string(), 1)),
                false,
            ))))
            .unwrap();
    }
    lines
}
#[test]
fn layout_emits_adjusted_deletions_before_identity_preserving_reorder() {
    let lines = list();
    let original: Vec<_> = (0..8).map(|i| lines.get(i).unwrap()).collect();
    let observed = Rc::new(RefCell::new(Vec::new()));
    let output = observed.clone();
    let weak = Rc::downgrade(&lines);
    lines.on_delete_emitter.event().subscribe(move |change| {
        let lines = weak.upgrade().unwrap();
        assert_eq!(lines.len(), 8);
        output.borrow_mut().push(*change);
    });
    let result = reflow_larger_create_new_layout(&lines, &[(1, 2), (5, 1)]).unwrap();
    assert_eq!(
        result,
        NewLayoutResult {
            layout: vec![0, 3, 4, 6, 7],
            count_removed: 3
        }
    );
    assert_eq!(
        *observed.borrow(),
        [
            ListChange {
                index: 1,
                amount: 2
            },
            ListChange {
                index: 3,
                amount: 1
            }
        ]
    );
    reflow_larger_apply_new_layout(&lines, &result.layout).unwrap();
    assert_eq!(lines.len(), 5);
    for (i, source) in result.layout.iter().enumerate() {
        assert!(Rc::ptr_eq(&lines.get(i).unwrap(), &original[*source]));
    }
}
#[test]
fn layout_propagates_disposal_during_event_dispatch() {
    let lines = list();
    let weak = Rc::downgrade(&lines);
    lines
        .on_delete_emitter
        .event()
        .subscribe(move |_| weak.upgrade().unwrap().on_delete_emitter.dispose());
    lines
        .on_delete_emitter
        .event()
        .subscribe(|_| panic!("must not reach disposed listener"));
    assert_eq!(
        reflow_larger_create_new_layout(&lines, &[(1, 1)]),
        Err(ReflowError::Dispatch(EmitError::ListenerArrayCleared))
    );
    assert_eq!(lines.len(), 8);
}
#[test]
fn layout_retains_callback_error_identity_and_source_chain() {
    use std::error::Error;
    let lines = list();
    let original = EmitError::callback(std::io::Error::other("marker callback failed"));
    let captured = original.clone();
    lines
        .on_delete_emitter
        .event()
        .try_subscribe(move |_| Err(captured.clone()));
    let error = reflow_larger_create_new_layout(&lines, &[(1, 1)]).unwrap_err();
    assert_eq!(error, ReflowError::Dispatch(original));
    let dispatch = error.source().unwrap().downcast_ref::<EmitError>().unwrap();
    let root = dispatch
        .source()
        .unwrap()
        .downcast_ref::<std::io::Error>()
        .unwrap();
    assert_eq!(root.to_string(), "marker callback failed");
    assert_eq!(lines.len(), 8);
}
#[test]
fn layout_reentrant_listener_can_inspect_and_mutate_storage() {
    let lines = list();
    let weak = Rc::downgrade(&lines);
    lines
        .on_delete_emitter
        .event()
        .subscribe(move |_| weak.upgrade().unwrap().set_length(5));
    assert_eq!(
        reflow_larger_create_new_layout(&lines, &[(1, 1)])
            .unwrap()
            .layout,
        [0, 2, 3, 4]
    );
}
#[test]
fn invalid_layout_and_columns_are_errors() {
    let lines = list();
    for bad in [
        vec![(1, 0)],
        vec![(8, 1)],
        vec![(2, 3), (4, 1)],
        vec![(7, usize::MAX)],
    ] {
        assert_eq!(
            reflow_larger_create_new_layout(&lines, &bad),
            Err(ReflowError::InvalidRange)
        );
        assert_eq!(lines.len(), 8);
    }
    // Layout reads keep CircularList's cyclic semantics, even past length.
    let first = lines.get(0).unwrap();
    reflow_larger_apply_new_layout(&lines, &[8]).unwrap();
    assert_eq!(lines.len(), 1);
    assert!(Rc::ptr_eq(&first, &lines.get(0).unwrap()));
    let rows = vec![lines.get(0).unwrap()];
    for new in [0, 7] {
        assert_eq!(
            reflow_smaller_get_new_line_lengths(&rows, 6, new),
            Err(ReflowError::InvalidColumns)
        );
    }
    assert_eq!(
        reflow_larger_get_lines_to_remove(&lines, 6, 6, 0, &cell("", 1), true),
        Err(ReflowError::InvalidColumns)
    );
    assert_eq!(
        get_wrapped_line_trimmed_length(&[], 0, 6),
        Err(ReflowError::MissingLine)
    );
}
#[test]
fn deletion_observer_can_replace_list_before_layout_application() {
    let original = list();
    let holder = Rc::new(RefCell::new(original.clone()));
    let captured = Rc::downgrade(&holder);
    original.on_delete_emitter.event().subscribe(move |_| {
        *captured.upgrade().unwrap().borrow_mut() = Rc::new(CircularList::new(8).unwrap());
    });
    let result = reflow_larger_create_new_layout(&original, &[(1, 2)]).unwrap();
    assert_eq!(result.layout, [0, 3, 4, 5, 6, 7]);
    let replacement = holder.borrow().clone();
    reflow_larger_apply_new_layout(&replacement, &result.layout).unwrap();
    assert_eq!(replacement.len(), 6);
    assert!((0..6).all(|i| replacement.get(i).is_none()));
    assert_eq!(original.len(), 8);
}
#[test]
fn wide_gap_excluded_only_before_wide_start() {
    let a = Rc::new(RefCell::new(BufferLine::new(5, None, false)));
    let b = Rc::new(RefCell::new(BufferLine::new(5, None, true)));
    b.borrow_mut().set_cell(0, &cell("汉", 2));
    b.borrow_mut().set_cell(1, &cell("", 0));
    assert_eq!(
        get_wrapped_line_trimmed_length(&[a.clone(), b.clone()], 0, 5).unwrap(),
        4
    );
    a.borrow_mut().set_cell(4, &cell(" ", 1));
    assert_eq!(
        get_wrapped_line_trimmed_length(&[a.clone(), b.clone()], 0, 5).unwrap(),
        5
    );
    assert_eq!(get_wrapped_line_trimmed_length(&[a, b], 1, 5).unwrap(), 2);
}
#[test]
fn width_one_supports_narrow_cells_and_rejects_wide_zero_progress() {
    let narrow = Rc::new(RefCell::new(BufferLine::new(4, Some(&cell("a", 1)), false)));
    assert_eq!(
        reflow_smaller_get_new_line_lengths(&[narrow], 4, 1).unwrap(),
        [1, 1, 1, 1]
    );
    let wide = Rc::new(RefCell::new(BufferLine::new(4, None, false)));
    wide.borrow_mut().set_cell(0, &cell("汉", 2));
    wide.borrow_mut().set_cell(1, &cell("", 0));
    assert_eq!(
        reflow_smaller_get_new_line_lengths(&[wide], 4, 1),
        Err(ReflowError::InvalidColumns)
    );
}
#[test]
fn caller_held_borrows_report_errors_instead_of_panicking() {
    let lines = list();
    let row = lines.get(1).unwrap();
    row.borrow_mut().is_wrapped = true;
    let held = row.borrow_mut();
    assert_eq!(
        reflow_larger_get_lines_to_remove(&lines, 4, 6, 20, &cell("", 1), true),
        Err(ReflowError::BorrowedLine)
    );
    assert_eq!(
        get_wrapped_line_trimmed_length(std::slice::from_ref(&row), 0, 4),
        Err(ReflowError::BorrowedLine)
    );
    drop(held);
    assert!(get_wrapped_line_trimmed_length(std::slice::from_ref(&row), 0, 4).is_ok());
}
#[test]
fn widening_preserves_extended_attribute_object_identity() {
    let lines = CircularList::new(2).unwrap();
    let a = Rc::new(RefCell::new(BufferLine::new(4, None, false)));
    let b = Rc::new(RefCell::new(BufferLine::new(4, None, true)));
    a.borrow_mut().set_cell(0, &cell("a", 1));
    a.borrow_mut().set_cell(1, &cell("b", 1));
    let mut combined = cell("e\u{301}", 1);
    combined.bg = xterm_core::buffer::constants::bg_flags::HAS_EXTENDED;
    combined.extended.set_url_id(17);
    b.borrow_mut().set_cell(0, &combined);
    lines.push(a.clone()).unwrap();
    lines.push(b).unwrap();
    assert_eq!(
        reflow_larger_get_lines_to_remove(&lines, 2, 4, 10, &cell("", 1), true).unwrap(),
        [(1, 1)]
    );
    let copied = a.borrow().get_cell(2).unwrap();
    assert_eq!(
        copied.get_chars(),
        "e\u{301}".encode_utf16().collect::<Vec<_>>()
    );
    assert!(copied.extended.same_identity(&combined.extended));
}
#[test]
fn empty_and_unwrapped_rows_do_not_mutate() {
    let empty = CircularList::new(1).unwrap();
    assert_eq!(
        reflow_larger_get_lines_to_remove(&empty, 4, 6, 0, &cell("", 1), true).unwrap(),
        []
    );
    assert_eq!(reflow_smaller_get_new_line_lengths(&[], 6, 4).unwrap(), []);
    let lines = list();
    let refs: Vec<_> = (0..8).map(|i| lines.get(i).unwrap()).collect();
    assert_eq!(
        reflow_larger_get_lines_to_remove(&lines, 4, 6, 0, &cell("", 1), true).unwrap(),
        []
    );
    for (i, row) in refs.iter().enumerate() {
        assert!(Rc::ptr_eq(row, &lines.get(i).unwrap()));
    }
}
