#![cfg(target_arch = "wasm32")]
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen_test::*;
use xterm_core::{
    circular_list::ListChange,
    decoration::{DecorationAnchor, DecorationLayer},
};
#[path = "support/buffer_decoration.rs"]
mod support;
use support::{element, frame, microtask, property, Fixture};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn live_geometry_layers_element_handle_and_dimension_callback_order() {
    let fixture = Fixture::new();
    fixture.buffers.normal().borrow_mut().ydisp = 2;
    let decoration = fixture.register(5, |options| {
        options.x = Some(3.0);
        options.width = Some(2.0);
        options.height = Some(2);
        options.anchor = Some(DecorationAnchor::Right);
        options.layer = Some(DecorationLayer::Top);
    });
    assert!(fixture.renderer.has_pending_refresh());
    assert!(decoration.element().is_none());
    let observed = Rc::new(RefCell::new(Vec::new()));
    let output = observed.clone();
    decoration.on_render().subscribe(move |handle| {
        let element = handle.downcast_ref::<web_sys::HtmlElement>().unwrap();
        output.borrow_mut().push(property(element, "right"));
    });
    frame().await;
    let element = element(&decoration);
    assert!(element.class_list().contains("xterm-decoration-top-layer"));
    for (name, value) in [
        ("width", "17px"),
        ("height", "33px"),
        ("top", "49.5px"),
        ("line-height", "16.5px"),
        ("right", "24.75px"),
        ("display", "block"),
    ] {
        assert_eq!(property(&element, name), value, "{name}");
    }
    let original = decoration.element().unwrap();
    let mut dimensions = fixture.dimensions.get();
    dimensions.css.cell.width = 10.0;
    fixture.dimensions.set(dimensions);
    fixture.changed.fire(&dimensions).unwrap();
    frame().await;
    assert!(Rc::ptr_eq(&original, &decoration.element().unwrap()));
    assert_eq!(property(&element, "right"), "30px");
    // The source emits onRender before its dimension-triggered X-position update.
    assert_eq!(observed.borrow().last().unwrap(), "24.75px");
}

#[wasm_bindgen_test(async)]
async fn source_zero_defaults_anchor_changes_and_creation_only_layer_class() {
    let fixture = Fixture::new();
    let decoration = fixture.register(0, |options| {
        options.x = Some(3.0);
        options.anchor = Some(DecorationAnchor::Right);
        options.layer = Some(DecorationLayer::Top);
    });
    frame().await;
    let node = element(&decoration);
    {
        let options = decoration.options();
        let mut options = options.borrow_mut();
        options.width = Some(0.0);
        options.height = Some(0);
        options.x = Some(0.0);
        options.anchor = Some(DecorationAnchor::Left);
        options.layer = Some(DecorationLayer::Bottom);
    }
    fixture.changed.fire(&fixture.dimensions.get()).unwrap();
    frame().await;
    assert_eq!(property(&node, "width"), "8px");
    assert_eq!(property(&node, "height"), "16.5px");
    assert_eq!(property(&node, "left"), "");
    assert_eq!(property(&node, "right"), "24.75px");
    assert!(node.class_list().contains("xterm-decoration-top-layer"));
    let outside = fixture.register(1, |options| options.x = Some(100.0));
    frame().await;
    // Source creation hides an oversized x, then refreshStyle overwrites display.
    assert_eq!(property(&element(&outside), "display"), "block");
}

#[wasm_bindgen_test(async)]
async fn viewport_alternate_screen_and_marker_disposal_use_real_nodes() {
    let fixture = Fixture::new();
    let decoration = fixture.register(5, |_| {});
    frame().await;
    let node = element(&decoration);
    let rendered = Rc::new(Cell::new(0));
    let output = rendered.clone();
    decoration
        .on_render()
        .subscribe(move |_| output.set(output.get() + 1));
    fixture.buffers.normal().borrow_mut().ydisp = 7;
    fixture.refresh();
    assert_eq!(property(&node, "display"), "none");
    assert_eq!(rendered.get(), 1);
    fixture.buffers.activate_alt_buffer(None).unwrap();
    fixture.refresh();
    assert_eq!(property(&node, "display"), "none");
    fixture.buffers.activate_normal_buffer().unwrap();
    fixture.buffers.normal().borrow_mut().ydisp = 0;
    fixture.refresh();
    assert_eq!(property(&node, "display"), "block");
    assert!(node.is_connected());
    decoration.marker().dispose().unwrap();
    assert!(!node.is_connected());
    assert!(decoration.is_disposed());
    assert!(decoration.element().is_some());
    assert_eq!(fixture.renderer.container().child_element_count(), 0);
}

#[wasm_bindgen_test(async)]
async fn registration_during_render_keeps_source_frame_sentinel_and_snapshot() {
    let fixture = Fixture::new();
    let first = fixture.register(0, |_| {});
    let done = Rc::new(Cell::new(false));
    let once = done.clone();
    let service = Rc::downgrade(&fixture.decorations);
    let buffers = Rc::downgrade(&fixture.buffers);
    first.on_render().subscribe(move |_| {
        if once.replace(true) {
            return;
        }
        let service = service.upgrade().unwrap();
        let buffers = buffers.upgrade().unwrap();
        let marker = buffers.active().borrow_mut().add_marker(1);
        service
            .register_decoration(Rc::new(RefCell::new(
                xterm_core::decoration::DecorationOptions::new(marker),
            )))
            .unwrap()
            .unwrap();
    });
    frame().await;
    assert_eq!(fixture.renderer.container().child_element_count(), 1);
    assert!(!fixture.renderer.has_pending_refresh());
    fixture.refresh();
    assert_eq!(fixture.renderer.container().child_element_count(), 2);
    fixture.dpr.fire(&2.0).unwrap();
    assert!(fixture.renderer.has_pending_refresh());
    frame().await;
}

#[wasm_bindgen_test(async)]
async fn queued_component_refresh_after_dispose_matches_detached_source_container() {
    let fixture = Fixture::new();
    let decoration = fixture.register(0, |_| {});
    let count = Rc::new(Cell::new(0));
    let output = count.clone();
    decoration
        .on_render()
        .subscribe(move |_| output.set(output.get() + 1));
    fixture.renderer.dispose();
    assert!(!fixture.renderer.container().is_connected());
    frame().await;
    assert_eq!(count.get(), 1);
    assert_eq!(fixture.renderer.container().child_element_count(), 1);
    assert!(!element(&decoration).is_connected());
}

#[wasm_bindgen_test(async)]
async fn real_microtask_cache_sync_is_independent_of_renderer_lifetime() {
    let fixture = Fixture::new();
    let decoration = fixture.register(10, |_| {});
    frame().await;
    fixture.renderer.dispose();
    fixture
        .buffers
        .normal()
        .borrow()
        .lines
        .on_insert_emitter
        .fire(&ListChange {
            index: 5,
            amount: 1,
        })
        .unwrap();
    assert_eq!(decoration.marker().line(), 11);
    assert!(fixture.decorations.line_cache().has_pending_sync());
    microtask().await;
    assert!(!fixture.decorations.line_cache().has_pending_sync());
    assert!(fixture
        .decorations
        .get_decorations_at_cell(0.0, 11, None)
        .next()
        .unwrap()
        .ptr_eq(&decoration));
}

#[wasm_bindgen_test(async)]
async fn exported_terminal_owns_the_same_decoration_service_and_overlay() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let parent = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
    let terminal = xterm_web::terminal::Terminal::new(js_sys::Object::new().into()).unwrap();
    terminal.open(parent.clone().into()).unwrap();
    let marker = terminal
        .shared_native()
        .with_state(|state| state.active_buffer().borrow_mut().add_marker(0))
        .unwrap();
    let decoration = terminal
        .decoration_service()
        .register_decoration(Rc::new(RefCell::new(
            xterm_core::decoration::DecorationOptions::new(marker),
        )))
        .unwrap()
        .unwrap();
    terminal.refresh(0.0, 23.0).unwrap();
    frame().await;
    let screen = terminal.screen_element().unwrap();
    assert!(screen
        .query_selector(".xterm-decoration-container .xterm-decoration")
        .unwrap()
        .is_some());
    assert!(element(&decoration).is_connected());
    terminal.dispose().unwrap();
    assert!(!element(&decoration).is_connected());
    parent.remove();
}
