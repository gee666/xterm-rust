#![cfg(target_arch = "wasm32")]
#[path = "support/mouse_dom.rs"]
pub mod support;
use std::rc::Rc;
use support::{input::event, Host};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Event, HtmlElement};
use xterm_web::{
    dom::{BrowserDom, DomOptions},
    input::dom::mouse::MouseBindings,
};
wasm_bindgen_test_configure!(run_in_browser);
#[path = "cases/mouse_getters.rs"]
mod getters;
#[path = "cases/mouse_operands.rs"]
mod operands;
#[path = "cases/mouse_reentry.rs"]
mod reentry;
#[path = "cases/mouse_trace.rs"]
mod trace;

struct Setup {
    parent: HtmlElement,
    dom: BrowserDom,
    host: Rc<Host>,
    mouse: MouseBindings,
}
impl Setup {
    fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let parent: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
        document.body().unwrap().append_child(&parent).unwrap();
        let dom = BrowserDom::new(&parent, &DomOptions::default()).unwrap();
        dom.screen.style().set_property("padding", "0px").unwrap();
        let host = Rc::new(Host::new());
        let mouse = MouseBindings::new(&dom, host.clone()).unwrap();
        Self {
            parent,
            dom,
            host,
            mouse,
        }
    }
    fn pointer(&self, name: &str, buttons: u16) -> Event {
        let rect = self.dom.screen.get_bounding_client_rect();
        event(
            "MouseEvent",
            name,
            &[
                ("clientX", (rect.left() + 4.0).into()),
                ("clientY", (rect.top() + 4.0).into()),
                ("button", 0.into()),
                ("buttons", buttons.into()),
            ],
        )
    }
    fn wheel(&self) -> Event {
        let rect = self.dom.screen.get_bounding_client_rect();
        event(
            "WheelEvent",
            "wheel",
            &[
                ("clientX", (rect.left() + 4.0).into()),
                ("clientY", (rect.top() + 4.0).into()),
                ("deltaY", 1.into()),
                ("deltaMode", 1.into()),
            ],
        )
    }
    fn gesture(&self, name: &str, translation: f64) -> Event {
        let event = event("Event", name, &[]);
        let rect = self.dom.screen.get_bounding_client_rect();
        for (name, value) in [
            ("translationY", translation),
            ("clientX", rect.left() + 4.0),
            ("clientY", rect.top() + 4.0),
        ] {
            js_sys::Reflect::set(&event, &name.into(), &value.into()).unwrap();
        }
        event
    }
}
impl Drop for Setup {
    fn drop(&mut self) {
        self.mouse.dispose().unwrap();
        // The browser runner rejects unexpected page errors. Expected exception
        // cases assert identity through their own temporary window error trap.
        self.parent.remove();
    }
}

#[wasm_bindgen_test]
fn global_drag_release_cleanup_focus_and_forced_selection() {
    let setup = Setup::new();
    setup.host.input.mouse.set_active_protocol("ANY").unwrap();
    setup.host.input.mouse.set_active_encoding("SGR").unwrap();
    assert!(!setup.host.selection_enabled.get());
    let down = setup.pointer("mousedown", 1);
    setup.dom.element.dispatch_event(&down).unwrap();
    assert!(down.default_prevented());
    assert_eq!(setup.host.focus_calls.get(), 1);
    assert_eq!(setup.host.input.data.borrow().len(), 1);
    let up = setup.pointer("mouseup", 0);
    setup.dom.document.dispatch_event(&up).unwrap();
    assert_eq!(setup.host.input.data.borrow().len(), 2);
    setup
        .dom
        .document
        .dispatch_event(&setup.pointer("mousemove", 1))
        .unwrap();
    assert_eq!(
        setup.host.input.data.borrow().len(),
        2,
        "last release removed global drag"
    );
    setup.host.force_selection.set(true);
    setup
        .dom
        .element
        .dispatch_event(&setup.pointer("mousedown", 1))
        .unwrap();
    assert_eq!(setup.host.focus_calls.get(), 2);
    assert_eq!(setup.host.input.data.borrow().len(), 2);
    setup.mouse.dispose().unwrap();
    assert_eq!(setup.host.gesture_disposals.get(), 1);
    setup
        .dom
        .element
        .dispatch_event(&setup.pointer("mousedown", 1))
        .unwrap();
    assert_eq!(
        setup.host.focus_calls.get(),
        2,
        "disposed listener must not run"
    );
}

#[wasm_bindgen_test]
fn wheel_veto_nonpassive_cancellation_and_alt_buffer_keys() {
    let setup = Setup::new();
    setup.host.input.mouse.set_active_protocol("ANY").unwrap();
    setup.host.input.mouse.set_active_encoding("SGR").unwrap();
    let wheel = setup.wheel();
    setup.dom.element.dispatch_event(&wheel).unwrap();
    assert!(wheel.default_prevented());
    assert_eq!(setup.host.input.data.borrow().len(), 1);
    assert!(
        setup.host.input.data.borrow()[0].1,
        "SGR output uses user data"
    );
    setup
        .host
        .input
        .mouse
        .set_active_encoding("DEFAULT")
        .unwrap();
    setup.dom.element.dispatch_event(&setup.wheel()).unwrap();
    assert!(
        !setup.host.input.data.borrow()[1].1,
        "default encoding uses binary"
    );
    setup.host.input.data.borrow_mut().clear();
    setup.host.veto.set(true);
    let wheel = setup.wheel();
    setup.dom.element.dispatch_event(&wheel).unwrap();
    assert!(
        wheel.default_prevented(),
        "active wheel cancels even on veto"
    );
    assert_eq!(setup.host.wheel_calls.get(), 3);
    assert!(setup.host.input.data.borrow().is_empty());
    setup.host.input.mouse.set_active_protocol("NONE").unwrap();
    let mut snapshot = setup.host.snapshot.get();
    snapshot.has_scrollback = false;
    snapshot.application_cursor = true;
    setup.host.snapshot.set(snapshot);
    let wheel = setup.wheel();
    setup.dom.element.dispatch_event(&wheel).unwrap();
    assert!(
        !wheel.default_prevented(),
        "passive veto leaves browser default alone"
    );
    setup.host.veto.set(false);
    let wheel = setup.wheel();
    setup.dom.element.dispatch_event(&wheel).unwrap();
    assert!(wheel.default_prevented());
    assert_eq!(
        setup.host.input.data.borrow().as_slice(),
        &[(vec![27, 79, 66], true)]
    );
}

#[wasm_bindgen_test]
fn alt_cursor_mode_option_and_blur_follow_live_state() {
    let setup = Setup::new();
    setup.host.input.mouse.set_active_protocol("ANY").unwrap();
    assert!(setup
        .dom
        .element
        .class_list()
        .contains("enable-mouse-events"));
    let mut snapshot = setup.host.snapshot.get();
    snapshot.routing.require_alt = true;
    setup.host.snapshot.set(snapshot);
    setup
        .host
        .option_changes
        .fire(&"mouseEventsRequireAlt")
        .unwrap();
    assert!(setup.host.selection_enabled.get());
    assert!(!setup
        .dom
        .element
        .class_list()
        .contains("enable-mouse-events"));
    setup
        .dom
        .document
        .dispatch_event(&event(
            "KeyboardEvent",
            "keydown",
            &[("key", "Alt".into()), ("altKey", true.into())],
        ))
        .unwrap();
    assert!(setup
        .dom
        .element
        .class_list()
        .contains("enable-mouse-events"));
    web_sys::window()
        .unwrap()
        .dispatch_event(&event("Event", "blur", &[]))
        .unwrap();
    assert!(!setup
        .dom
        .element
        .class_list()
        .contains("enable-mouse-events"));
    setup.host.input.mouse.set_active_protocol("NONE").unwrap();
    setup
        .dom
        .document
        .dispatch_event(&event(
            "KeyboardEvent",
            "keydown",
            &[("altKey", true.into())],
        ))
        .unwrap();
    assert!(!setup
        .dom
        .element
        .class_list()
        .contains("enable-mouse-events"));
}

#[wasm_bindgen_test]
fn synthetic_gesture_consumer_routes_viewport_keys_and_wheel() {
    // Recognition is task108. This test dispatches explicit source gesture events.
    let setup = Setup::new();
    let change = setup.gesture("-xterm-gesturechange", 18.0);
    setup.dom.screen.dispatch_event(&change).unwrap();
    assert!(change.default_prevented());
    assert_eq!(setup.host.touch_scroll.borrow().as_slice(), &[18.0]);
    let mut snapshot = setup.host.snapshot.get();
    snapshot.has_scrollback = false;
    setup.host.snapshot.set(snapshot);
    setup
        .dom
        .screen
        .dispatch_event(&setup.gesture("-xterm-gesturestart", 0.0))
        .unwrap();
    setup
        .dom
        .screen
        .dispatch_event(&setup.gesture("-xterm-gesturechange", 36.0))
        .unwrap();
    assert_eq!(
        setup.host.input.data.borrow().as_slice(),
        &[(vec![27, 91, 65], true), (vec![27, 91, 65], true)]
    );
    setup.host.input.mouse.set_active_protocol("ANY").unwrap();
    setup.host.input.mouse.set_active_encoding("SGR").unwrap();
    setup.host.veto.set(true);
    setup
        .dom
        .screen
        .dispatch_event(&setup.gesture("-xterm-gesturestart", 0.0))
        .unwrap();
    setup
        .dom
        .screen
        .dispatch_event(&setup.gesture("-xterm-gesturechange", -18.0))
        .unwrap();
    assert_eq!(setup.host.input.data.borrow().len(), 3);
    assert_eq!(
        setup.host.wheel_calls.get(),
        0,
        "touch wheel bypasses custom native wheel callback"
    );
}
