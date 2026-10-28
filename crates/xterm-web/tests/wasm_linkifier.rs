#![cfg(target_arch = "wasm32")]
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{HtmlElement, MouseEvent};
use xterm_web::links::native_osc::{NativeOscLinkProvider, OscLinkHandler};
use xterm_web::links::{
    browser_linkifier::{BrowserLink, BrowserLinkProvider, BrowserLinkifier, LinkReply},
    Link, Position, Range, RangePosition,
};
wasm_bindgen_test_configure!(run_in_browser);
#[derive(Default)]
struct Provider {
    callbacks: RefCell<Vec<LinkReply>>,
    requests: Cell<usize>,
}
impl BrowserLinkProvider for Provider {
    fn provide_links(&self, _y: usize, callback: LinkReply) -> Result<(), JsValue> {
        self.requests.set(self.requests.get() + 1);
        self.callbacks.borrow_mut().push(callback);
        Ok(())
    }
}
fn element() -> HtmlElement {
    let document = web_sys::window().unwrap().document().unwrap();
    let element: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
    element.set_class_name("xterm");
    document.body().unwrap().append_child(&element).unwrap();
    element
}
fn event(element: &HtmlElement, kind: &str) {
    element
        .dispatch_event(&MouseEvent::new(kind).unwrap())
        .unwrap();
}
fn link(log: Rc<RefCell<Vec<String>>>) -> BrowserLink {
    let activate = log.clone();
    let hover = log.clone();
    let leave = log.clone();
    let dispose = log;
    BrowserLink {
        data: Rc::new(Link {
            text: "https://example.com/".encode_utf16().collect(),
            range: Range {
                start: RangePosition { x: 3, y: 1 },
                end: RangePosition { x: 7, y: 1 },
            },
        }),
        decorations: None,
        activate: Rc::new(move |_, _| {
            activate.borrow_mut().push("activate".into());
            Ok(())
        }),
        hover: Some(Rc::new(move |_, _| {
            hover.borrow_mut().push("hover".into());
            Ok(())
        })),
        leave: Some(Rc::new(move |_, _| {
            leave.borrow_mut().push("leave".into());
            Ok(())
        })),
        dispose: Some(Rc::new(move || {
            dispose.borrow_mut().push("dispose".into());
            Ok(())
        })),
    }
}
#[wasm_bindgen_test]
fn dom_hover_activation_decorations_and_cached_leave() {
    let root = element();
    let position = Rc::new(Cell::new(Position { x: 5, y: 1 }));
    let coords = position.clone();
    let log = Rc::new(RefCell::new(Vec::new()));
    let draw = log.clone();
    let controller = BrowserLinkifier::new(
        root.clone(),
        80,
        0,
        Rc::new(move |_| Some(coords.get())),
        Rc::new(move |show, e| {
            assert_eq!((e.x1, e.y1, e.x2, e.y2), (2, 0, 7, 0));
            draw.borrow_mut().push(format!("underline:{show}"));
            Ok(())
        }),
    )
    .unwrap();
    let provider = Rc::new(Provider::default());
    let _registration = controller.register_provider(provider.clone());
    event(&root, "mousemove");
    assert_eq!(provider.requests.get(), 1);
    let reply = provider.callbacks.borrow_mut().remove(0);
    reply(Some(vec![link(log.clone())])).unwrap();
    assert!(root.class_list().contains("xterm-cursor-pointer"));
    assert_eq!(*log.borrow(), vec!["underline:true", "hover"]);
    event(&root, "mousedown");
    event(&root, "mouseup");
    assert_eq!(log.borrow().last().unwrap(), "activate");
    controller.set_current_underline(false).unwrap();
    assert_eq!(log.borrow().last().unwrap(), "underline:false");
    controller.set_current_pointer_cursor(false).unwrap();
    assert!(!root.class_list().contains("xterm-cursor-pointer"));
    position.set(Position { x: 8, y: 1 });
    event(&root, "mousemove");
    assert_eq!(provider.requests.get(), 1);
    assert!(controller.current_link().is_none());
    assert_eq!(log.borrow().last().unwrap(), "leave");
    controller.dispose().unwrap();
    position.set(Position { x: 5, y: 1 });
    event(&root, "mousemove");
    assert_eq!(provider.requests.get(), 1);
    root.remove();
}
#[wasm_bindgen_test]
fn async_priority_mouseout_and_disposal_ignore_late_replies() {
    let root = element();
    let log = Rc::new(RefCell::new(Vec::new()));
    let controller = BrowserLinkifier::new(
        root.clone(),
        80,
        0,
        Rc::new(|_| Some(Position { x: 5, y: 1 })),
        Rc::new(|_, _| Ok(())),
    )
    .unwrap();
    let high = Rc::new(Provider::default());
    let low = Rc::new(Provider::default());
    let _high = controller.register_provider(high.clone());
    let _low = controller.register_provider(low.clone());
    event(&root, "mousemove");
    let lower = low.callbacks.borrow_mut().remove(0);
    let higher = high.callbacks.borrow_mut().remove(0);
    lower(Some(vec![link(log.clone())])).unwrap();
    assert!(controller.current_link().is_none());
    higher(None).unwrap();
    assert!(controller.current_link().is_some());
    event(&root, "mouseleave");
    lower(Some(vec![link(log.clone())])).unwrap();
    assert!(controller.current_link().is_none());
    controller.dispose().unwrap();
    higher(Some(vec![link(log.clone())])).unwrap();
    assert!(controller.current_link().is_none());
    root.remove();
}
#[wasm_bindgen_test]
fn rendered_rows_clear_requery_and_dispose_old_cached_links() {
    let root = element();
    let log = Rc::new(RefCell::new(Vec::new()));
    let controller = BrowserLinkifier::new(
        root.clone(),
        80,
        0,
        Rc::new(|_| Some(Position { x: 5, y: 1 })),
        Rc::new(|_, _| Ok(())),
    )
    .unwrap();
    let provider = Rc::new(Provider::default());
    let _registration = controller.register_provider(provider.clone());
    event(&root, "mousemove");
    provider.callbacks.borrow_mut().remove(0)(Some(vec![link(log.clone())])).unwrap();
    controller.on_render(1, 3).unwrap();
    assert_eq!(provider.requests.get(), 1);
    controller.on_render(0, 0).unwrap();
    assert_eq!(provider.requests.get(), 2);
    assert_eq!(*log.borrow(), vec!["hover", "leave", "dispose"]);
    root.remove();
}

#[wasm_bindgen_test]
fn real_rust_osc8_cells_resolve_wrap_activate_and_expire_on_trim() {
    let native = Rc::new(xterm_core::headless::Terminal::new(5, 3, 0).unwrap());
    native
        .write("\u{1b}]8;id=shared;https://example.com/path\u{1b}\\abcdeFG\u{1b}]8;;\u{1b}\\x")
        .unwrap();
    let id = native
        .with_state(|state| {
            let buffer = state.active_buffer();
            let line = buffer.borrow().lines.get(0).unwrap();
            let id = line.borrow().get_extended(0).unwrap().url_id();
            assert!(state.links.get_link_data(id).is_some());
            id
        })
        .unwrap();
    assert_ne!(id, 0);
    let root = element();
    let activated = Rc::new(Cell::new(0));
    let count = activated.clone();
    let handler = OscLinkHandler {
        allow_non_http_protocols: Rc::new(|| Ok(false)),
        activate: Rc::new(move |_, link| {
            assert_eq!(
                link.text,
                "https://example.com/path"
                    .encode_utf16()
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                link.range,
                Range {
                    start: RangePosition { x: 1, y: 1 },
                    end: RangePosition { x: 2, y: 2 }
                }
            );
            count.set(count.get() + 1);
            Ok(())
        }),
        hover: None,
        leave: None,
    };
    let provider = Rc::new(NativeOscLinkProvider::new(
        Rc::downgrade(&native),
        web_sys::window().unwrap(),
        move || Ok(Some(handler.clone())),
    ));
    let controller = BrowserLinkifier::new(
        root.clone(),
        5,
        0,
        Rc::new(|_| Some(Position { x: 3, y: 1 })),
        Rc::new(|_, _| Ok(())),
    )
    .unwrap();
    let _registration = controller.register_provider(provider);
    event(&root, "mousemove");
    assert!(controller.current_link().is_some());
    event(&root, "mousedown");
    event(&root, "mouseup");
    assert_eq!(activated.get(), 1);
    native.write("\r\n1\r\n2\r\n3\r\n4\r\n5\r\n6").unwrap();
    native
        .with_state(|state| assert!(state.links.get_link_data(id).is_none()))
        .unwrap();
    controller.on_render(0, 2).unwrap();
    assert!(controller.current_link().is_none());
    root.remove();
}

#[wasm_bindgen_test]
fn underline_callback_changes_pointer_before_hover_pointer_step() {
    let root = element();
    let holder: Rc<RefCell<Option<std::rc::Weak<BrowserLinkifier>>>> = Rc::new(RefCell::new(None));
    let weak = holder.clone();
    let controller = Rc::new(
        BrowserLinkifier::new(
            root.clone(),
            80,
            0,
            Rc::new(|_| Some(Position { x: 5, y: 1 })),
            Rc::new(move |show, _| {
                if show {
                    weak.borrow()
                        .as_ref()
                        .unwrap()
                        .upgrade()
                        .unwrap()
                        .set_current_pointer_cursor(false)?;
                }
                Ok(())
            }),
        )
        .unwrap(),
    );
    *holder.borrow_mut() = Some(Rc::downgrade(&controller));
    let provider = Rc::new(Provider::default());
    let _registration = controller.register_provider(provider.clone());
    event(&root, "mousemove");
    provider.callbacks.borrow_mut().remove(0)(Some(vec![link(Rc::new(RefCell::new(Vec::new())))]))
        .unwrap();
    assert!(controller.current_link().is_some());
    assert!(!root.class_list().contains("xterm-cursor-pointer"));
    root.remove();
}

#[wasm_bindgen_test]
fn throwing_leave_preserves_current_until_successful_callback_completion() {
    let root = element();
    let holder: Rc<RefCell<Option<std::rc::Weak<BrowserLinkifier>>>> = Rc::new(RefCell::new(None));
    let controller = Rc::new(
        BrowserLinkifier::new(
            root.clone(),
            80,
            0,
            Rc::new(|_| Some(Position { x: 5, y: 1 })),
            Rc::new(|_, _| Ok(())),
        )
        .unwrap(),
    );
    *holder.borrow_mut() = Some(Rc::downgrade(&controller));
    let fail = Rc::new(Cell::new(true));
    let should_fail = fail.clone();
    let weak = holder.clone();
    let marker: JsValue = js_sys::Object::new().into();
    let thrown = marker.clone();
    let mut provided = link(Rc::new(RefCell::new(Vec::new())));
    provided.leave = Some(Rc::new(move |_, _| {
        assert!(weak
            .borrow()
            .as_ref()
            .unwrap()
            .upgrade()
            .unwrap()
            .current_link()
            .is_some());
        if should_fail.replace(false) {
            Err(thrown.clone())
        } else {
            Ok(())
        }
    }));
    let provider = Rc::new(Provider::default());
    let _registration = controller.register_provider(provider.clone());
    event(&root, "mousemove");
    provider.callbacks.borrow_mut().remove(0)(Some(vec![provided])).unwrap();
    assert_eq!(controller.resize(80, 0).unwrap_err(), marker);
    assert!(controller.current_link().is_some());
    controller.resize(80, 0).unwrap();
    assert!(controller.current_link().is_none());
    root.remove();
}
