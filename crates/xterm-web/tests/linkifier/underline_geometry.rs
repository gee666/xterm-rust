use js_sys::{Function, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{HtmlElement, MouseEvent};
use xterm_web::links::{
    browser_linkifier::{BrowserLink, BrowserLinkProvider, BrowserLinkifier, LinkReply},
    Position,
};

#[derive(Default)]
struct Provider(RefCell<Option<LinkReply>>);
impl BrowserLinkProvider for Provider {
    fn provide_links(&self, _: usize, reply: LinkReply) -> Result<(), JsValue> {
        *self.0.borrow_mut() = Some(reply);
        Ok(())
    }
}

#[wasm_bindgen_test]
fn underline_event_snapshots_offset_after_range_and_columns_after_arithmetic() {
    for mode in [
        "plain",
        "range",
        "start.x",
        "start.y",
        "end.x",
        "end.y",
        "coerce start.x",
        "coerce start.y",
        "coerce end.y",
        "throw range",
        "throw end.y",
    ] {
        for show in [true, false] {
            let document = web_sys::window().unwrap().document().unwrap();
            let element: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
            element.set_class_name("xterm");
            document.body().unwrap().append_child(&element).unwrap();
            let events = Rc::new(RefCell::new(Vec::new()));
            let output = events.clone();
            let controller = Rc::new(
                BrowserLinkifier::new(
                    element.clone(),
                    20,
                    2,
                    Rc::new(|_| Some(Position { x: 2, y: 5 })),
                    Rc::new(move |show, event| {
                        output.borrow_mut().push((show, event));
                        Ok(())
                    }),
                )
                .unwrap(),
            );
            let weak = Rc::downgrade(&controller);
            let mutation = Closure::<dyn Fn() -> Result<(), JsValue>>::new(move || {
                weak.upgrade().unwrap().resize(40, 3)
            });
            let setup = Function::new_with_args("mode, mutate, show", r#"
            const marker = {}, reads = [];
            let armed = false, changed = false;
            const read = (key, value) => {
              if (!armed) return value;
              reads.push(key);
              if (mode === `throw ${key}`) throw marker;
              const change = () => { if (!changed) { changed = true; mutate(); } };
              if (mode === key) change();
              if (mode === `coerce ${key}`) return { valueOf() { reads.push(`coerce ${key}`); change(); return value; } };
              return value;
            };
            const range = { start: { get x() { return read('start.x', 2); }, get y() { return read('start.y', 5); } },
              end: { get x() { return read('end.x', 8); }, get y() { return read('end.y', 6); } } };
            const link = { text: mode, get range() { read('range', 0); return range; },
              decorations: { underline: !show, pointerCursor: false }, activate() {} };
            return { link, run() {
              armed = true;
              try { link.decorations.underline = show; } catch (error) {
                if (error !== marker) throw error;
                return true;
              } finally { armed = false; }
              return false;
            } };
        "#).call3(&JsValue::UNDEFINED, &mode.into(), mutation.as_ref(), &show.into()).unwrap();
            let provider = Rc::new(Provider::default());
            let _registration = controller.register_provider(provider.clone());
            element
                .dispatch_event(&MouseEvent::new("mousemove").unwrap())
                .unwrap();
            let reply = provider.0.borrow_mut().take().unwrap();
            reply(Some(vec![BrowserLink::JavaScript(
                Reflect::get(&setup, &"link".into()).unwrap(),
            )]))
            .unwrap();
            events.borrow_mut().clear();
            let run: Function = Reflect::get(&setup, &"run".into())
                .unwrap()
                .dyn_into()
                .unwrap();
            let caught = run.call0(&setup).unwrap().as_bool().unwrap();
            assert_eq!(caught, mode.starts_with("throw"), "{mode}");
            let events = events.borrow();
            if caught {
                assert!(events.is_empty(), "{mode}");
            } else {
                let (actual_show, event) = events.last().unwrap();
                assert_eq!(*actual_show, show, "{mode}");
                assert_eq!((event.x1, event.x2), (1, 8), "{mode}");
                assert_eq!(
                    (event.y1, event.y2),
                    if mode == "range" { (1, 2) } else { (2, 3) },
                    "{mode}"
                );
                assert_eq!(event.cols, if mode == "plain" { 20 } else { 40 }, "{mode}");
            }
            drop(events);
            controller.dispose().unwrap();
            element.remove();
        }
    }
}
