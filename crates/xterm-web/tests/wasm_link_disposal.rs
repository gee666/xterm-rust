#![cfg(target_arch = "wasm32")]
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{HtmlElement, MouseEvent};
use xterm_web::links::{
    browser_linkifier::{
        BrowserLink, BrowserLinkProvider, BrowserLinkifier, LinkReply, NativeBrowserLink,
    },
    Link, Position, Range, RangePosition,
};
wasm_bindgen_test_configure!(run_in_browser);

#[derive(Default)]
struct Provider(RefCell<Vec<LinkReply>>);
impl BrowserLinkProvider for Provider {
    fn provide_links(&self, _: usize, reply: LinkReply) -> Result<(), JsValue> {
        self.0.borrow_mut().push(reply);
        Ok(())
    }
}
fn link(x: u64, dispose: impl Fn() -> Result<(), JsValue> + 'static) -> BrowserLink {
    BrowserLink::Native(NativeBrowserLink {
        data: Rc::new(Link {
            text: format!("link{x}").encode_utf16().collect(),
            range: Range {
                start: RangePosition { x, y: 1 },
                end: RangePosition { x: x + 2, y: 1 },
            },
        }),
        decorations: None,
        activate: Rc::new(|_, _| Ok(())),
        hover: None,
        leave: None,
        dispose: Some(Rc::new(dispose)),
    })
}
fn move_to(root: &HtmlElement, x: i32, y: i32) {
    let create = js_sys::Function::new_with_args(
        "x,y",
        "return new MouseEvent('mousemove',{clientX:x,clientY:y});",
    );
    let event: MouseEvent = create
        .call2(&JsValue::UNDEFINED, &x.into(), &y.into())
        .unwrap()
        .dyn_into()
        .unwrap();
    root.dispatch_event(&event).unwrap();
}
fn run(replace_later_reply: bool) {
    let document = web_sys::window().unwrap().document().unwrap();
    let root: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
    root.set_class_name("xterm");
    document.body().unwrap().append_child(&root).unwrap();
    let controller = BrowserLinkifier::new(
        root.clone(),
        80,
        0,
        Rc::new(|event| {
            Some(Position {
                x: event.client_x() as usize,
                y: event.client_y() as usize,
            })
        }),
        Rc::new(|_, _| Ok(())),
    )
    .unwrap();
    let first = Rc::new(Provider::default());
    let later = Rc::new(Provider::default());
    let _first = controller.register_provider(first.clone());
    let _later = controller.register_provider(later.clone());
    move_to(&root, 3, 1);
    let first_reply = first.0.borrow_mut().remove(0);
    let later_reply = Rc::new(later.0.borrow_mut().remove(0));
    let log = Rc::new(RefCell::new(Vec::new()));
    let replacement_reply = later_reply.clone();
    let disposal_log = log.clone();
    first_reply(Some(vec![link(3, move || {
        disposal_log.borrow_mut().push("first");
        let log = disposal_log.clone();
        later_reply(Some(vec![link(20, move || {
            log.borrow_mut().push("new");
            Ok(())
        })]))
    })]))
    .unwrap();
    if replace_later_reply {
        let log = log.clone();
        replacement_reply(Some(vec![link(10, move || {
            log.borrow_mut().push("old");
            Ok(())
        })]))
        .unwrap();
    }
    move_to(&root, 3, 2);
    assert_eq!(
        *log.borrow(),
        ["first", "new"],
        "Map traversal must visit a newly completed or replaced later provider reply"
    );
    controller.dispose().unwrap();
    root.remove();
}
#[wasm_bindgen_test]
fn disposal_visits_a_provider_reply_completed_by_an_earlier_dispose_callback() {
    run(false);
}
#[wasm_bindgen_test]
fn disposal_visits_the_replacement_not_the_previous_later_provider_reply() {
    run(true);
}

#[wasm_bindgen_test]
fn nested_mousemove_does_not_redirect_an_outer_disposal_to_the_new_cache() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
    root.set_class_name("xterm");
    document.body().unwrap().append_child(&root).unwrap();
    let controller = BrowserLinkifier::new(
        root.clone(),
        80,
        0,
        Rc::new(|event| {
            Some(Position {
                x: 3,
                y: event.client_y() as usize,
            })
        }),
        Rc::new(|_, _| Ok(())),
    )
    .unwrap();
    let provider = Rc::new(Provider::default());
    let _registration = controller.register_provider(provider.clone());
    move_to(&root, 3, 1);
    let reply = provider.0.borrow_mut().remove(0);
    let log = Rc::new(RefCell::new(Vec::new()));
    let first_log = log.clone();
    let tail_log = log.clone();
    let nested = Cell::new(false);
    let nested_root = root.clone();
    reply(Some(vec![
        link(3, move || {
            first_log.borrow_mut().push("first");
            if !nested.replace(true) {
                move_to(&nested_root, 3, 3);
                let log = first_log.clone();
                let reply = provider.0.borrow_mut().remove(0);
                reply(Some(vec![link(20, move || {
                    log.borrow_mut().push("new-cache");
                    Ok(())
                })]))?;
            }
            Ok(())
        }),
        link(10, move || {
            tail_log.borrow_mut().push("tail");
            Ok(())
        }),
    ]))
    .unwrap();
    move_to(&root, 3, 2);
    assert_eq!(*log.borrow(), ["first", "first", "tail", "tail"]);
    controller.dispose().unwrap();
    root.remove();
}
