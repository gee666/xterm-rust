// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{AttributeData, BufferService, BufferSet},
    options::{OptionValue, OptionsService},
};

fn options() -> Rc<OptionsService> {
    Rc::new(OptionsService::new(
        false,
        [
            ("cols", OptionValue::Number(10.0)),
            ("rows", OptionValue::Number(5.0)),
            ("scrollback", OptionValue::Number(4.0)),
        ],
    ))
}
#[test]
fn activation_noops_fill_clear_markers_and_event_order() {
    let service = BufferService::new(options()).unwrap();
    let set = &service.buffers;
    assert_eq!(set.normal().borrow().lines.len(), 5);
    assert_eq!(set.alt().borrow().lines.len(), 0);
    assert_eq!(set.alt().borrow().lines.max_length(), 5);
    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = Rc::clone(&events);
    service
        .on_scroll()
        .subscribe(move |position| seen.borrow_mut().push(format!("scroll{position}")));
    let seen = Rc::clone(&events);
    set.on_buffer_activate()
        .subscribe(move |_| seen.borrow_mut().push("activate".into()));
    set.activate_normal_buffer().unwrap();
    assert!(events.borrow().is_empty());
    set.normal().borrow_mut().x = 3;
    set.normal().borrow_mut().y = 2;
    let attr = AttributeData {
        fg: 123,
        bg: 45,
        ..AttributeData::default()
    };
    set.activate_alt_buffer(Some(&attr)).unwrap();
    assert_eq!(set.alt().borrow().x, 3);
    assert_eq!(set.alt().borrow().y, 2);
    assert_eq!(
        set.alt().borrow().lines.get(0).unwrap().borrow().get_fg(0),
        Some(123)
    );
    assert_eq!(&*events.borrow(), &["scroll0", "activate"]);
    events.borrow_mut().clear();
    set.activate_alt_buffer(None).unwrap();
    assert!(events.borrow().is_empty());
    let marker = set.alt().borrow_mut().add_marker(1);
    let seen = Rc::clone(&events);
    marker
        .on_dispose()
        .subscribe(move |_| seen.borrow_mut().push("marker".into()));
    set.alt().borrow_mut().x = 4;
    set.alt().borrow_mut().y = 1;
    set.activate_normal_buffer().unwrap();
    assert_eq!(set.normal().borrow().x, 4);
    assert_eq!(set.normal().borrow().y, 1);
    assert_eq!(set.alt().borrow().lines.len(), 0);
    assert!(marker.is_disposed());
    assert_eq!(&*events.borrow(), &["marker", "scroll0", "activate"]);
}
#[test]
fn activation_callbacks_can_switch_back_synchronously() {
    let service = BufferService::new(options()).unwrap();
    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = Rc::clone(&events);
    service
        .on_scroll()
        .subscribe(move |_| seen.borrow_mut().push("scroll"));
    let weak = Rc::downgrade(&service.buffers);
    let seen = Rc::clone(&events);
    service
        .buffers
        .on_buffer_activate()
        .subscribe(move |event| {
            let set = weak.upgrade().unwrap();
            let is_alt = Rc::ptr_eq(&event.active_buffer, &set.alt());
            seen.borrow_mut()
                .push(if is_alt { "alt" } else { "normal" });
            if is_alt {
                set.activate_normal_buffer().unwrap();
            }
        });
    service.buffers.activate_alt_buffer(None).unwrap();
    assert!(Rc::ptr_eq(&service.buffer(), &service.buffers.normal()));
    assert!(service.buffers.alt().borrow().lines.is_empty());
    assert_eq!(&*events.borrow(), &["scroll", "alt", "scroll", "normal"]);
}

#[test]
fn reset_replaces_identities_and_disposes_old_markers_before_activation() {
    let service = BufferService::new(options()).unwrap();
    let old_normal = service.buffers.normal();
    let old_alt = service.buffers.alt();
    let marker = old_normal.borrow_mut().add_marker(1);
    service.is_user_scrolling.set(true);
    service.reset().unwrap();
    assert!(!Rc::ptr_eq(&old_normal, &service.buffers.normal()));
    assert!(!Rc::ptr_eq(&old_alt, &service.buffers.alt()));
    assert!(marker.is_disposed());
    assert!(!service.is_user_scrolling.get());
    assert!(Rc::ptr_eq(&service.buffer(), &service.buffers.normal()));
}
#[test]
fn live_options_rebuild_tabs_resize_history_and_stop_after_dispose() {
    let options = options();
    let set = BufferSet::new(10, 5, Rc::clone(&options)).unwrap();
    assert_eq!(set.normal().borrow().lines.max_length(), 9);
    options.set("scrollback", OptionValue::Number(2.0)).unwrap();
    assert_eq!(set.normal().borrow().lines.max_length(), 7);
    assert_eq!(set.alt().borrow().lines.max_length(), 5);
    options
        .set("tabStopWidth", OptionValue::Number(3.0))
        .unwrap();
    assert_eq!(
        set.normal()
            .borrow()
            .tabs
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [0, 3, 6, 9]
    );
    assert_eq!(set.alt().borrow().tabs, set.normal().borrow().tabs);
    set.dispose().unwrap();
    options
        .set("tabStopWidth", OptionValue::Number(2.0))
        .unwrap();
    assert_eq!(
        set.normal()
            .borrow()
            .tabs
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [0, 3, 6, 9]
    );
}
#[test]
fn option_callback_errors_are_propagated_and_original_cause_retained() {
    let options = options();
    let set = BufferSet::new(10, 5, Rc::clone(&options)).unwrap();
    let normal = set.normal();
    let held = normal.borrow_mut();
    let error = options
        .set("scrollback", OptionValue::Number(3.0))
        .unwrap_err();
    let xterm_core::options::OptionError::Dispatch(error) = error else {
        panic!("callback error must propagate through dispatch");
    };
    assert!(matches!(
        error.callback_ref::<xterm_core::buffer::BufferError>(),
        Some(xterm_core::buffer::BufferError::BorrowedLine)
    ));
    drop(held);
    // The option committed before listener dispatch, as in JavaScript.
    assert_eq!(options.get("scrollback").unwrap(), OptionValue::Number(3.0));
    set.resize(10, 5).unwrap();
    assert_eq!(normal.borrow().lines.max_length(), 8);
}
#[test]
fn dimensions_commit_before_resize_events_and_unchanged_resizes_still_emit() {
    let service = Rc::new(BufferService::new(options()).unwrap());
    let weak = Rc::downgrade(&service);
    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = Rc::clone(&events);
    service.on_resize().subscribe(move |event| {
        let service = weak.upgrade().unwrap();
        assert_eq!((service.cols(), service.rows()), (event.cols, event.rows));
        assert_eq!(
            (
                service.buffer().borrow().cols(),
                service.buffer().borrow().rows()
            ),
            (event.cols, event.rows)
        );
        seen.borrow_mut()
            .push((event.cols_changed, event.rows_changed));
    });
    service.resize(10, 5).unwrap();
    service.resize(12, 5).unwrap();
    service.resize(12, 3).unwrap();
    assert_eq!(
        &*events.borrow(),
        &[(false, false), (true, false), (false, true)]
    );
    service.resize(1, 1).unwrap(); // Internal resize is not constructor minimum enforcement.
    assert_eq!(
        service
            .buffer()
            .borrow()
            .lines
            .get(0)
            .unwrap()
            .borrow()
            .len(),
        1
    );
}
#[test]
fn display_clamp_suppression_and_scrolling_flag_boundaries() {
    let service = BufferService::new(options()).unwrap();
    let active = service.buffer();
    active.borrow_mut().ybase = 4;
    active.borrow_mut().ydisp = 4;
    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = Rc::clone(&events);
    service
        .on_scroll()
        .subscribe(move |event| seen.borrow_mut().push(*event));
    service.scroll_lines(-2, true).unwrap();
    assert_eq!(active.borrow().ydisp, 2);
    assert!(events.borrow().is_empty());
    service.scroll_lines(i64::MIN, false).unwrap();
    assert_eq!(active.borrow().ydisp, 0);
    assert!(service.is_user_scrolling.get());
    service.scroll_lines(-1, false).unwrap();
    assert!(service.is_user_scrolling.get());
    service.scroll_lines(i64::MAX, false).unwrap();
    assert_eq!(active.borrow().ydisp, 4);
    assert!(!service.is_user_scrolling.get());
    service.scroll_lines(0, false).unwrap();
    assert_eq!(&*events.borrow(), &[0, 4]);
}
#[test]
fn disposal_failure_marks_owner_disposed_before_later_calls() {
    let service = BufferService::new(options()).unwrap();
    let marker = service.buffer().borrow_mut().add_marker(0);
    marker.on_dispose().try_subscribe(|_| {
        Err(xterm_core::event::EmitError::callback(
            std::io::Error::other("intentional marker failure"),
        ))
    });
    assert!(service.dispose().is_err());
    assert!(marker.is_disposed());
    // Source DisposableStore marks disposed before entering callbacks. It does
    // not resume the interrupted resource loop on another dispose call.
    let alt_marker = service.buffers.alt().borrow_mut().add_marker(0);
    service.dispose().unwrap();
    assert!(!alt_marker.is_disposed());

    let set = BufferSet::new(10, 5, options()).unwrap();
    set.dispose().unwrap();
    let old = set.normal();
    let post_dispose_marker = old.borrow_mut().add_marker(0);
    set.reset().unwrap();
    // MutableDisposable refuses replacements after disposal. Raw normal/alt
    // fields still change, but the old raw buffer is not disposed a second time.
    assert!(!post_dispose_marker.is_disposed());
    assert!(!Rc::ptr_eq(&old, &set.normal()));
}

#[test]
fn constructor_minimum_and_js_falsy_or_numeric_coercion() {
    for value in [
        OptionValue::Null,
        OptionValue::Undefined,
        OptionValue::Bool(false),
        OptionValue::Number(f64::NAN),
        OptionValue::Number(-5.0),
        OptionValue::from(""),
    ] {
        let options = options();
        options.set_raw("cols", value.clone()).unwrap();
        options.set_raw("rows", value).unwrap();
        let service = BufferService::new(options).unwrap();
        assert_eq!((service.cols(), service.rows()), (2, 1));
    }
    let options = options();
    options.set_raw("cols", OptionValue::from("3")).unwrap();
    assert_eq!(BufferService::new(options).unwrap().cols(), 3);
}
#[test]
fn cached_blank_attributes_and_wrapped_flag_follow_latest_scroll() {
    let service = BufferService::new(options()).unwrap();
    for (cols, fg, bg, wrapped) in [
        (10, 2, 3, false),
        (10, 2, 3, true),
        (10, 4, 3, false),
        (10, 4, 5, true),
        (12, 4, 5, false),
    ] {
        if service.cols() != cols {
            service.resize(cols, 5).unwrap();
        }
        service
            .scroll(
                &AttributeData {
                    fg,
                    bg,
                    ..AttributeData::default()
                },
                wrapped,
            )
            .unwrap();
        let active = service.buffer();
        let active = active.borrow();
        let last = active
            .lines
            .get(active.ybase + active.scroll_bottom)
            .unwrap();
        let last = last.borrow();
        assert_eq!(last.len(), cols);
        assert_eq!(last.get_fg(0), Some(fg));
        assert_eq!(last.get_bg(0), Some(bg));
        assert_eq!(last.is_wrapped, wrapped);
    }
}
