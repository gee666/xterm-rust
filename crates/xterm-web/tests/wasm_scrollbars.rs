#![cfg(target_arch = "wasm32")]
// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Source-derived real DOM assertions. No source *.test.ts exists for these
//! internal scroll classes. Trusted capture is a separate end-to-end gate.
use std::{
    cell::{Cell, RefCell},
    ops::Deref,
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{EventTarget, HtmlElement, PointerEvent, PointerEventInit};
use xterm_core::{
    lifecycle::{Disposable, DisposableHandle},
    options::OptionValue as V,
};
use xterm_web::{
    links::animation_queue::WindowAnimationQueues,
    scroll::{
        abstract_scrollbar::{Scrollbar, ScrollbarHost, SharedOptions},
        browser_scheduler::{new_scrollable, BrowserScrollable},
        horizontal_scrollbar::HorizontalScrollbar,
        iframe::IframeCache,
        options::ResolvedOptions,
        state::DimensionUpdate,
        vertical_scrollbar::VerticalScrollbar,
    },
};
wasm_bindgen_test_configure!(run_in_browser);
#[path = "support/trusted_scrollbar.rs"]
mod trusted;

struct Fixture<B: Deref<Target = Scrollbar>> {
    bar: Rc<B>,
    model: BrowserScrollable,
    options: SharedOptions,
    parent: HtmlElement,
    starts: Rc<Cell<usize>>,
    ends: Rc<Cell<usize>>,
    subscription: DisposableHandle,
}
impl<B: Deref<Target = Scrollbar>> Drop for Fixture<B> {
    fn drop(&mut self) {
        self.subscription.dispose();
        self.bar.dispose().unwrap();
        self.parent.remove();
    }
}
fn parts() -> (
    BrowserScrollable,
    SharedOptions,
    ScrollbarHost,
    Rc<Cell<usize>>,
    Rc<Cell<usize>>,
    HtmlElement,
) {
    let window = web_sys::window().unwrap();
    let mut queues = WindowAnimationQueues::default();
    let model = new_scrollable(queues.for_window(&window), false, 0.0);
    model
        .set_scroll_dimensions(
            DimensionUpdate {
                width: Some(200.0),
                height: Some(100.0),
                scroll_width: Some(2000.0),
                scroll_height: Some(1000.0),
            },
            false,
        )
        .unwrap();
    let options = Rc::new(RefCell::new(
        ResolvedOptions::resolve(&js_sys::Object::new().into(), false).unwrap(),
    ));
    let starts = Rc::new(Cell::new(0));
    let ends = Rc::new(Cell::new(0));
    let a = starts.clone();
    let b = ends.clone();
    let host = ScrollbarHost {
        drag_start: Rc::new(move || {
            a.set(a.get() + 1);
            Ok(())
        }),
        drag_end: Rc::new(move || {
            b.set(b.get() + 1);
            Ok(())
        }),
        iframe_cache: Rc::new(IframeCache::new()),
    };
    let doc = window.document().unwrap();
    let parent = doc
        .create_element("div")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    parent.set_class_name("xterm xterm-scrollable-element");
    parent
        .style()
        .set_css_text("position:relative;width:200px;height:100px");
    doc.body().unwrap().append_child(&parent).unwrap();
    (model, options, host, starts, ends, parent)
}
fn observe<B: Deref<Target = Scrollbar> + 'static>(
    model: &BrowserScrollable,
    bar: &Rc<B>,
) -> DisposableHandle {
    let weak = Rc::downgrade(bar);
    model.on_scroll().try_subscribe(move |event| {
        if let Some(bar) = weak.upgrade() {
            bar.handle_scroll(event)
                .map_err(xterm_web::js_error::callback_error)?;
        }
        Ok(())
    })
}
fn vertical(windows: bool, lazy: bool, page: bool) -> Fixture<VerticalScrollbar> {
    let (model, options, host, starts, ends, parent) = parts();
    options.borrow_mut().lazy_render = V::Bool(lazy);
    options.borrow_mut().scroll_by_page = V::Bool(page);
    let bar =
        Rc::new(VerticalScrollbar::new(model.clone(), options.clone(), host, windows).unwrap());
    parent.append_child(&bar.dom_node()).unwrap();
    bar.render().unwrap();
    let subscription = observe(&model, &bar);
    Fixture {
        bar,
        model,
        options,
        parent,
        starts,
        ends,
        subscription,
    }
}
fn pointer(kind: &str, x: f64, y: f64, button: i16, buttons: u16) -> PointerEvent {
    let init = PointerEventInit::new();
    init.set_pointer_id(999);
    init.set_button(button);
    init.set_buttons(buttons);
    init.set_client_x(x as i32);
    init.set_client_y(y as i32);
    init.set_cancelable(true);
    init.set_bubbles(true);
    PointerEvent::new_with_event_init_dict(kind, &init).unwrap()
}
fn css(element: &HtmlElement, key: &str) -> String {
    element.style().get_property_value(key).unwrap()
}
fn window() -> EventTarget {
    web_sys::window().unwrap().into()
}
#[wasm_bindgen_test]
fn horizontal_and_vertical_layout_roles_and_hidden_arrow_nodes_match_source() {
    let f = vertical(false, false, false);
    let node = f.bar.dom_node();
    let slider = f.bar.slider_node().unwrap();
    assert_eq!(node.get_attribute("role").as_deref(), Some("presentation"));
    assert_eq!(node.get_attribute("aria-hidden").as_deref(), Some("true"));
    assert_eq!(css(&node, "width"), "10px");
    assert_eq!(css(&node, "height"), "100px");
    assert_eq!(css(&node, "right"), "0px");
    assert_eq!(css(&slider, "height"), "20px");
    assert_eq!(css(&slider, "width"), "10px");
    assert_eq!(css(&slider, "contain"), "strict");
    assert_eq!(node.child_element_count(), 5);
    let up = node
        .query_selector(".xterm-arrow-up")
        .unwrap()
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    assert_eq!(css(&up, "display"), "none");
    let (model, options, host, starts, ends, parent) = parts();
    let bar =
        Rc::new(HorizontalScrollbar::new(model.clone(), options.clone(), host, false).unwrap());
    parent.append_child(&bar.dom_node()).unwrap();
    bar.render().unwrap();
    let subscription = observe(&model, &bar);
    let h = Fixture {
        bar,
        model,
        options,
        parent,
        starts,
        ends,
        subscription,
    };
    assert_eq!(css(&h.bar.dom_node(), "width"), "190px");
    assert_eq!(css(&h.bar.dom_node(), "height"), "10px");
    assert_eq!(css(&h.bar.slider_node().unwrap(), "width"), "20px");
}
#[wasm_bindgen_test]
fn horizontal_arrows_throw_the_source_error_after_base_initialization() {
    let (model, options, host, _, _, parent) = parts();
    options.borrow_mut().horizontal_has_arrows = V::Bool(true);
    let error = HorizontalScrollbar::new(model, options, host, false)
        .err()
        .unwrap();
    assert_eq!(
        js_sys::Reflect::get(&error, &"message".into())
            .unwrap()
            .as_string()
            .as_deref(),
        Some("horizontalHasArrows is not supported in xterm.js")
    );
    parent.remove();
}
#[wasm_bindgen_test]
fn track_clicks_center_or_page_and_nonprimary_clicks_do_not_start_drag() {
    let f = vertical(false, false, false);
    let rect = f.bar.dom_node().get_bounding_client_rect();
    let event = pointer("pointerdown", rect.left() + 5.0, rect.top() + 75.0, 2, 0);
    f.bar.dom_node().dispatch_event(event.as_ref()).unwrap();
    assert_eq!(f.model.current_scroll_position().top, 731.0);
    assert_eq!(f.starts.get(), 0);
    assert!(!event.default_prevented());
    let p = vertical(false, false, true);
    let rect = p.bar.dom_node().get_bounding_client_rect();
    p.bar
        .dom_node()
        .dispatch_event(pointer("pointerdown", rect.left() + 5.0, rect.top() + 75.0, 2, 0).as_ref())
        .unwrap();
    assert_eq!(p.model.current_scroll_position().top, 100.0);
}
#[wasm_bindgen_test]
fn slider_drag_updates_scroll_and_windows_reset_threshold_is_strict() {
    let f = vertical(true, false, false);
    let slider = f.bar.slider_node().unwrap();
    let rect = slider.get_bounding_client_rect();
    let x = rect.left() + 5.0;
    let y = rect.top() + 5.0;
    let down = pointer("pointerdown", x, y, 0, 1);
    slider.dispatch_event(down.as_ref()).unwrap();
    assert!(down.default_prevented());
    assert!(slider.class_list().contains("xterm-active"));
    assert_eq!(f.starts.get(), 1);
    window()
        .dispatch_event(pointer("pointermove", x + 140.0, y + 20.0, 0, 1).as_ref())
        .unwrap();
    assert_eq!(f.model.current_scroll_position().top, 225.0);
    window()
        .dispatch_event(pointer("pointermove", x + 141.0, y + 20.0, 0, 1).as_ref())
        .unwrap();
    assert_eq!(f.model.current_scroll_position().top, 0.0);
    window()
        .dispatch_event(pointer("pointerup", x, y, 0, 0).as_ref())
        .unwrap();
    assert_eq!(f.ends.get(), 1);
    assert!(!slider.class_list().contains("xterm-active"));
}
#[wasm_bindgen_test]
fn lazy_scroll_waits_for_render_and_updates_vertical_arrows_and_scrollbar_size() {
    let f = vertical(false, true, false);
    let slider = f.bar.slider_node().unwrap();
    f.model
        .set_scroll_position_now(xterm_web::scroll::state::PositionUpdate {
            top: Some(450.0),
            left: None,
        })
        .unwrap();
    assert_eq!(css(&slider, "top"), "0px");
    f.bar.render().unwrap();
    assert_eq!(css(&slider, "top"), "40px");
    {
        let mut o = f.options.borrow_mut();
        o.vertical_has_arrows = V::Bool(true);
        o.vertical_scrollbar_size = 20.0.into();
    }
    f.bar.update_options(&f.options).unwrap();
    f.bar.render().unwrap();
    assert_eq!(css(&f.bar.dom_node(), "width"), "20px");
    assert_eq!(css(&slider, "width"), "20px");
    let up = f
        .bar
        .dom_node()
        .query_selector(".xterm-arrow-up")
        .unwrap()
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    assert_eq!(css(&up, "display"), "");
    assert_eq!(css(&up, "width"), "20px");
}
#[wasm_bindgen_test]
fn dispose_during_drag_does_not_invent_a_drag_end_or_clear_the_active_class() {
    let f = vertical(false, false, false);
    let slider = f.bar.slider_node().unwrap();
    let rect = slider.get_bounding_client_rect();
    slider
        .dispatch_event(pointer("pointerdown", rect.left() + 5.0, rect.top() + 5.0, 0, 1).as_ref())
        .unwrap();
    assert_eq!(f.starts.get(), 1);
    f.bar.dispose().unwrap();
    window()
        .dispatch_event(pointer("pointerup", 0.0, 0.0, 0, 0).as_ref())
        .unwrap();
    assert_eq!(f.ends.get(), 0);
    assert!(slider.class_list().contains("xterm-active"));
}
