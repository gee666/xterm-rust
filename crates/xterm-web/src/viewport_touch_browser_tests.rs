//! Exercise touch recognition through the real terminal mouse/Viewport consumer.
use super::tests::Fixture;
use js_sys::{Array, Function, Object, Reflect};
use std::fmt::Write;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{Event, HtmlElement};

struct Clock(JsValue);
impl Clock {
    fn new() -> Self {
        let create = Function::new_no_args(concat!(
            include_str!("../../../tools/touch_live/browser_host.js"),
            "\nreturn createBrowserTouchHost();"
        ));
        Self(create.call0(&JsValue::UNDEFINED).unwrap())
    }
    fn time(&self, value: f64) {
        Reflect::set(&self.0, &"time".into(), &value.into()).unwrap();
    }
    fn flush(&self) {
        let callback: Function = Reflect::get(&self.0, &"flush".into())
            .unwrap()
            .dyn_into()
            .unwrap();
        callback.call0(&self.0).unwrap();
    }
}
impl Drop for Clock {
    fn drop(&mut self) {
        let restore: Function = Reflect::get(&self.0, &"restore".into())
            .unwrap()
            .dyn_into()
            .unwrap();
        restore.call0(&self.0).unwrap();
    }
}
fn send(screen: &HtmlElement, kind: &str, identifier: f64, y: f64) -> Event {
    let touch = Object::new();
    for (name, value) in [
        ("identifier", identifier.into()),
        ("target", screen.clone().into()),
        ("pageX", 0.0.into()),
        ("pageY", y.into()),
        ("clientX", 0.0.into()),
        ("clientY", y.into()),
    ] {
        Reflect::set(&touch, &name.into(), &value).unwrap();
    }
    let list = Array::new();
    list.push(&touch);
    Reflect::set(
        &list,
        &"item".into(),
        &Function::new_with_args("i", "return this[i];"),
    )
    .unwrap();
    let document = screen.owner_document().unwrap();
    let event = document.create_event("Event").unwrap();
    event.init_event_with_bubbles_and_cancelable(kind, false, true);
    for name in ["targetTouches", "changedTouches"] {
        Reflect::set(&event, &name.into(), &list).unwrap();
    }
    document.dispatch_event(&event).unwrap();
    event
}

#[wasm_bindgen_test]
fn actual_touch_drag_inertia_cancellation_and_disposal_control_terminal_scroll() {
    // Keep the clock alive until the real terminal and its owned queues drop.
    let clock = Clock::new();
    let fixture = Fixture::new();
    let services = fixture.services();
    let mut text = String::new();
    for line in 0..30 {
        write!(&mut text, "{line}\r\n").unwrap();
    }
    fixture.terminal.shared.native.write(&text).unwrap();
    fixture.viewport().sync(None).unwrap();
    clock.flush();
    clock.flush();
    let screen = services.dom.screen.clone();
    let height = services
        .render_service
        .dimensions()
        .unwrap()
        .css
        .cell
        .height;
    assert!(height > 0.0);
    let delta = (height * 2.0).max(40.0);
    let original = fixture.row();
    assert!(original >= 10, "fixture must retain scrollback for inertia");

    assert!(send(&screen, "touchstart", 1.0, 0.0).default_prevented());
    clock.time(1010.0);
    assert!(send(&screen, "touchmove", 1.0, delta).default_prevented());
    let after_drag = fixture.row();
    assert!(
        after_drag < original,
        "real touch CHANGE must reach Viewport and BufferService"
    );
    clock.time(1020.0);
    send(&screen, "touchend", 1.0, delta);
    clock.time(1030.0);
    clock.flush();
    let after_inertia = fixture.row();
    assert!(
        after_inertia < after_drag,
        "owned animation must continue scrolling the real buffer"
    );

    clock.time(1040.0);
    send(&screen, "touchstart", 2.0, 0.0);
    let cancelled = fixture.row();
    clock.time(1060.0);
    clock.flush();
    clock.time(1080.0);
    clock.flush();
    assert_eq!(
        fixture.row(),
        cancelled,
        "new touch must cancel old inertia"
    );

    // Start fresh inertia, then remove the actual mouse consumer by disposal.
    clock.time(1090.0);
    send(&screen, "touchmove", 2.0, delta);
    clock.time(1100.0);
    send(&screen, "touchend", 2.0, delta);
    fixture.terminal.dispose().unwrap();
    let disposed = fixture.row();
    clock.time(1110.0);
    clock.flush();
    clock.time(1130.0);
    clock.flush();
    clock.time(1150.0);
    clock.flush();
    assert_eq!(
        fixture.row(),
        disposed,
        "disposed consumer must not scroll from captured inertia targets"
    );
    assert!(!send(&screen, "touchstart", 3.0, 0.0).default_prevented());
    assert!(!send(&screen, "touchmove", 3.0, delta).default_prevented());
    assert_eq!(
        fixture.row(),
        disposed,
        "disposed terminal must not consume new gestures"
    );
    services.gesture.dispose().unwrap();
}
