use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    buffer::marker::Marker,
    circular_list::{CircularList, ListChange},
    decoration::{
        DecorationLayer, DecorationOptions, DecorationService, OverviewPosition,
        OverviewRulerOptions,
    },
    event::EmitError,
};
fn register(
    service: &DecorationService,
    options: DecorationOptions,
) -> xterm_core::decoration::Decoration {
    service
        .register_decoration(Rc::new(RefCell::new(options)))
        .unwrap()
        .unwrap()
}
#[test]
fn disposed_marker_rejection_defaults_and_layer_boundaries() {
    let service = DecorationService::new();
    let marker = Marker::new(3);
    marker.dispose().unwrap();
    assert!(service
        .register_decoration(Rc::new(RefCell::new(DecorationOptions::new(marker))))
        .unwrap()
        .is_none());
    let mut options = DecorationOptions::new(Marker::new(3));
    options.overview_ruler_options = Some(OverviewRulerOptions {
        color: "#123456".into(),
        position: None,
    });
    let options = Rc::new(RefCell::new(options));
    let d = service
        .register_decoration(options.clone())
        .unwrap()
        .unwrap();
    assert!(Rc::ptr_eq(&options, &d.options()));
    assert_eq!(
        options
            .borrow()
            .overview_ruler_options
            .as_ref()
            .unwrap()
            .position,
        Some(OverviewPosition::Full)
    );
    assert_eq!(
        service
            .get_decorations_at_cell(0.0, 3, Some(DecorationLayer::Bottom))
            .count(),
        1
    );
    assert_eq!(service.get_decorations_at_cell(1.0, 3, None).count(), 0);
    assert_eq!(
        service
            .get_decorations_at_cell(0.0, 3, Some(DecorationLayer::Top))
            .count(),
        0
    );
    options.borrow_mut().layer = Some(DecorationLayer::Top);
    assert_eq!(
        service
            .get_decorations_at_cell(0.0, 3, Some(DecorationLayer::Top))
            .count(),
        1
    );
}
#[test]
fn disposal_event_order_and_marker_invalidation() {
    let service = DecorationService::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    let output = log.clone();
    service.on_decoration_registered().subscribe(move |d| {
        assert!(!d.is_disposed());
        output.borrow_mut().push("registered");
    });
    let output = log.clone();
    let weak_service = service.clone();
    service.on_decoration_removed().subscribe(move |d| {
        assert!(!d.is_disposed());
        assert_eq!(d.marker().line(), -1);
        assert_eq!(
            weak_service.get_decorations_at_cell(0.0, 3, None).count(),
            0
        );
        output.borrow_mut().push("removed");
    });
    let d = register(&service, DecorationOptions::new(Marker::new(3)));
    let output = log.clone();
    let d2 = d.clone();
    d.on_dispose().subscribe(move |_| {
        assert!(!d2.is_disposed());
        output.borrow_mut().push("disposed");
    });
    d.marker().dispose().unwrap();
    d.dispose().unwrap();
    assert_eq!(*log.borrow(), ["registered", "removed", "disposed"]);
    assert_eq!(service.decorations().unwrap().count(), 0);
    // Explicit disposal above breaks the deliberately retained callback/service handles.
    service.dispose().unwrap();
}
#[test]
fn lazy_colors_cache_absence_success_and_retry_failed_parse() {
    let service = DecorationService::new();
    let d = register(&service, DecorationOptions::new(Marker::new(0)));
    assert!(d
        .background_color(|_| panic!("no parser for absent color"))
        .unwrap()
        .is_none());
    d.options().borrow_mut().background_color = Some("#fff".into());
    assert!(d
        .background_color(|_| panic!("absence cached"))
        .unwrap()
        .is_none());
    d.options().borrow_mut().foreground_color = Some("bad-color".into());
    assert!(d.foreground_color(|_| None).is_err());
    d.options().borrow_mut().foreground_color = Some("#123456".into());
    assert_eq!(
        d.foreground_color(|_| None).unwrap().unwrap().rgba,
        0x123456ff
    );
    d.options().borrow_mut().foreground_color = Some("#ffffff".into());
    assert_eq!(
        d.foreground_color(|_| panic!("success cached"))
            .unwrap()
            .unwrap()
            .rgba,
        0x123456ff
    );
}
#[test]
fn lookup_traverses_live_bucket_while_service_iterator_is_a_snapshot() {
    let service = DecorationService::new();
    let first = register(&service, DecorationOptions::new(Marker::new(0)));
    let second = register(&service, DecorationOptions::new(Marker::new(0)));
    let snapshot = service.decorations().unwrap();
    let mut live = service.get_decorations_at_cell(0.0, 0, None);
    assert!(live.next().unwrap().ptr_eq(&first));
    first.dispose().unwrap();
    assert!(live.next().is_none()); // Removing index zero shifts second below the cursor.
    assert_eq!(snapshot.count(), 2);
    assert!(service
        .get_decorations_at_cell(0.0, 0, None)
        .next()
        .unwrap()
        .ptr_eq(&second));
}
#[test]
fn attach_replaces_listeners_clear_cancels_work_and_batches_sync_requests() {
    let service = DecorationService::new();
    let a = CircularList::<u8>::new(20).unwrap();
    let b = CircularList::<u8>::new(20).unwrap();
    service.attach_to_buffer_lines(&a);
    register(&service, DecorationOptions::new(Marker::new(0)));
    let calls = Rc::new(Cell::new(0));
    let output = calls.clone();
    service
        .line_cache()
        .on_sync_requested()
        .subscribe(move |_| output.set(output.get() + 1));
    a.on_insert_emitter
        .fire(&ListChange {
            index: 0,
            amount: 1,
        })
        .unwrap();
    a.on_delete_emitter
        .fire(&ListChange {
            index: 0,
            amount: 1,
        })
        .unwrap();
    assert_eq!(calls.get(), 1);
    assert!(service.line_cache().has_pending_sync());
    service.line_cache().clear();
    service.line_cache().flush_pending().unwrap();
    assert!(!service.line_cache().has_pending_sync());
    service.attach_to_buffer_lines(&b);
    a.on_insert_emitter
        .fire(&ListChange {
            index: 0,
            amount: 1,
        })
        .unwrap();
    assert!(!service.line_cache().has_pending_sync());
    b.on_insert_emitter
        .fire(&ListChange {
            index: 0,
            amount: 1,
        })
        .unwrap();
    assert!(service.line_cache().has_pending_sync());
}
#[test]
fn live_buffer_activation_replaces_cache_subscriptions() {
    use xterm_core::{
        buffer::BufferSet,
        options::{OptionValue, OptionsService},
    };
    let options = Rc::new(OptionsService::new(
        false,
        std::iter::empty::<(&str, OptionValue)>(),
    ));
    let buffers = BufferSet::new(80, 24, options).unwrap();
    let service = DecorationService::with_buffers(&buffers).unwrap();
    let normal = buffers.normal();
    buffers.activate_alt_buffer(None).unwrap();
    let alt = buffers.alt();
    normal
        .borrow()
        .lines
        .on_insert_emitter
        .fire(&ListChange {
            index: 0,
            amount: 1,
        })
        .unwrap();
    assert!(!service.line_cache().has_pending_sync());
    let marker = alt.borrow_mut().add_marker(0);
    let d = register(&service, DecorationOptions::new(marker));
    alt.borrow()
        .lines
        .on_insert_emitter
        .fire(&ListChange {
            index: 1,
            amount: 1,
        })
        .unwrap();
    assert!(service.line_cache().has_pending_sync());
    service.line_cache().flush_pending().unwrap();
    buffers.activate_normal_buffer().unwrap();
    assert!(d.is_disposed());
    assert!(service.decorations().unwrap().next().is_none());
    alt.borrow()
        .lines
        .on_insert_emitter
        .fire(&ListChange {
            index: 0,
            amount: 1,
        })
        .unwrap();
    assert!(!service.line_cache().has_pending_sync());
}

#[test]
fn completed_cell_iterator_stays_done_when_bucket_later_grows() {
    let service = DecorationService::new();
    register(&service, DecorationOptions::new(Marker::new(0)));
    let mut iterator = service.get_decorations_at_cell(0.0, 0, None);
    assert!(iterator.next().is_some());
    assert!(iterator.next().is_none());
    register(&service, DecorationOptions::new(Marker::new(0)));
    assert!(iterator.next().is_none());
    let mut empty = service.get_decorations_at_cell(0.0, 1, None);
    assert!(empty.next().is_none());
    register(&service, DecorationOptions::new(Marker::new(1)));
    assert!(empty.next().is_none());
}

#[test]
fn render_handle_identity_and_listener_errors_propagate() {
    let service = DecorationService::new();
    let d = register(&service, DecorationOptions::new(Marker::new(0)));
    let element: Rc<dyn std::any::Any> = Rc::new(String::from("host element"));
    d.set_element(Some(element.clone()));
    assert!(Rc::ptr_eq(&element, &d.element().unwrap()));
    let expected = element.clone();
    d.on_render()
        .subscribe(move |value| assert!(Rc::ptr_eq(&expected, value)));
    d.fire_render(element).unwrap();
    d.on_dispose()
        .try_subscribe(|_| Err(EmitError::ListenerArrayCleared));
    assert!(d.dispose().is_err());
    assert!(!d.is_disposed());
    assert_eq!(service.get_decorations_at_cell(0.0, 0, None).count(), 0);
}
