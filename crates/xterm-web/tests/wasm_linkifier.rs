#![cfg(target_arch = "wasm32")]
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{HtmlElement, MouseEvent};
use xterm_web::links::{
    browser_linkifier::{BrowserLink, BrowserLinkProvider, BrowserLinkifier, LinkReply},
    Link, Position, Range,
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
                start: Position { x: 3, y: 1 },
                end: Position { x: 7, y: 1 },
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
    let mut controller = BrowserLinkifier::new(
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
    let mut controller = BrowserLinkifier::new(
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
