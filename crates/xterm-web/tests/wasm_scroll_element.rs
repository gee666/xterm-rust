#![cfg(target_arch = "wasm32")]
// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
use js_sys::{Function, Object, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{Event, HtmlElement};
use xterm_core::platform::{Navigator, Platform};
use xterm_web::{
    links::animation_queue::WindowAnimationQueues,
    scroll::{
        browser_scheduler::{new_scrollable, BrowserScrollable},
        element::SmoothScrollableElement,
        environment::ScrollEnvironment,
        state::{DimensionUpdate, PositionUpdate},
    },
};
wasm_bindgen_test_configure!(run_in_browser);
#[path = "support/scroll_element_contracts.rs"]
mod contracts;
#[path = "support/scroll_mouse_replay.rs"]
mod mouse_replay;
#[path = "support/scroll_element_replay.rs"]
mod replay;

struct Setup {
    parent: HtmlElement,
    element: Rc<SmoothScrollableElement>,
    model: BrowserScrollable,
}
impl Setup {
    fn new(options: &JsValue) -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let parent: HtmlElement = document.create_element("div").unwrap().unchecked_into();
        parent.set_class_name("xterm");
        let style = document.create_element("style").unwrap();
        style.set_text_content(Some(include_str!("../assets/xterm.css")));
        parent.append_child(&style).unwrap();
        document.body().unwrap().append_child(&parent).unwrap();
        let screen: HtmlElement = document.create_element("div").unwrap().unchecked_into();
        let environment = environment(&window);
        let queue = WindowAnimationQueues::default().for_window(&window);
        let model = new_scrollable(queue, false, 0.0);
        let element =
            SmoothScrollableElement::new(screen, options.clone(), model.clone(), environment)
                .unwrap();
        parent
            .append_child(&element.get_dom_node().unwrap())
            .unwrap();
        Self {
            parent,
            element,
            model,
        }
    }
    fn dimensions(&self) {
        self.element
            .set_scroll_dimensions(DimensionUpdate {
                width: Some(100.0),
                height: Some(40.0),
                scroll_width: Some(100.0),
                scroll_height: Some(400.0),
            })
            .unwrap();
    }
}
impl Drop for Setup {
    fn drop(&mut self) {
        self.element.dispose().unwrap();
        self.parent.remove();
    }
}
fn environment(window: &web_sys::Window) -> Rc<ScrollEnvironment> {
    let navigator = window.navigator();
    let user_agent = navigator.user_agent().unwrap();
    let platform = navigator.platform().unwrap();
    let platform = Platform::detect(
        false,
        Some(Navigator {
            user_agent: &user_agent,
            platform: &platform,
        }),
    )
    .unwrap();
    Rc::new(ScrollEnvironment::new(platform, window.clone()))
}
fn options(values: &[(&str, JsValue)]) -> JsValue {
    let options = Object::new();
    for (key, value) in values {
        Reflect::set(&options, &(*key).into(), value).unwrap();
    }
    options.into()
}
fn wheel() -> Event {
    Function::new_no_args(
        "const e = new WheelEvent('wheel', {view:window,deltaY:40,deltaMode:0,bubbles:true,cancelable:true});\n         Object.defineProperties(e, {wheelDeltaY:{value:-120},wheelDeltaX:{value:0}}); return e;",
    )
    .call0(&JsValue::NULL)
    .unwrap()
    .unchecked_into()
}

#[wasm_bindgen_test]
fn model_scroll_updates_dom_before_forwarding_the_event() {
    let setup = Setup::new(&options(&[
        ("useShadows", true.into()),
        ("horizontal", 2.into()),
    ]));
    setup.dimensions();
    let observations = Rc::new(RefCell::new(Vec::new()));
    let saved = observations.clone();
    let weak = Rc::downgrade(&setup.element);
    setup.element.on_scroll().subscribe(move |event| {
        let element = weak.upgrade().unwrap();
        let node = element.get_dom_node().unwrap();
        saved.borrow_mut().push((
            event.current.top,
            node.query_selector(".xterm-shadow-top").unwrap().is_some(),
        ));
    });
    setup
        .element
        .set_scroll_position(
            PositionUpdate {
                top: Some(25.0),
                left: None,
            },
            false,
        )
        .unwrap();
    assert_eq!(*observations.borrow(), [(25.0, true)]);
    let node = setup.element.get_dom_node().unwrap();
    assert_eq!(node.get_attribute("role").as_deref(), Some("presentation"));
    assert!(node
        .query_selector(".xterm-vertical .xterm-slider")
        .unwrap()
        .is_some());
    setup.element.dispose().unwrap();
    setup
        .model
        .set_scroll_position_now(PositionUpdate {
            top: Some(30.0),
            left: None,
        })
        .unwrap();
    assert_eq!(
        observations.borrow().len(),
        1,
        "external model survives but detached consumer does not fire"
    );
}
#[wasm_bindgen_test]
fn wheel_scroll_is_consumed_then_option_disables_the_listener() {
    let setup = Setup::new(&options(&[("horizontal", 2.into())]));
    setup.dimensions();
    let event = wheel();
    setup
        .element
        .get_dom_node()
        .unwrap()
        .dispatch_event(&event)
        .unwrap();
    assert!(event.default_prevented());
    assert_eq!(setup.element.get_scroll_position().top, 50.0);
    setup
        .element
        .update_options(&options(&[("handleMouseWheel", false.into())]))
        .unwrap();
    let event = wheel();
    setup
        .element
        .get_dom_node()
        .unwrap()
        .dispatch_event(&event)
        .unwrap();
    assert!(!event.default_prevented());
    assert_eq!(setup.element.get_scroll_position().top, 50.0);
    setup
        .element
        .update_options(&options(&[("handleMouseWheel", true.into())]))
        .unwrap();
    let event = wheel();
    setup
        .element
        .get_dom_node()
        .unwrap()
        .dispatch_event(&event)
        .unwrap();
    assert_eq!(setup.element.get_scroll_position().top, 100.0);
}
#[wasm_bindgen_test]
fn lazy_render_requires_explicit_render_and_preserves_fractional_position() {
    let setup = Setup::new(&options(&[
        ("lazyRender", true.into()),
        ("useShadows", false.into()),
    ]));
    setup.dimensions();
    setup
        .element
        .set_scroll_position(
            PositionUpdate {
                top: Some(5.5),
                left: None,
            },
            false,
        )
        .unwrap();
    assert_eq!(setup.element.get_scroll_position().top, 5.5);
    setup.element.render_now().unwrap();
    let vertical: HtmlElement = setup
        .element
        .get_dom_node()
        .unwrap()
        .query_selector(".xterm-vertical")
        .unwrap()
        .unwrap()
        .unchecked_into();
    assert_eq!(
        vertical.style().get_property_value("height").unwrap(),
        "40px"
    );
    let eager = Setup::new(&JsValue::UNDEFINED);
    assert!(eager.element.render_now().is_err());
}
