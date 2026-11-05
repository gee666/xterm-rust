use js_sys::{Function, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
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
fn render_rechecks_short_circuit_errors_and_reentrant_current_link() {
    for mode in [
        "plain",
        "second start outside",
        "second end outside",
        "throw second start",
        "throw second end",
        "first start outside",
        "first end outside",
        "zero start",
        "clear first start",
        "clear second start",
        "clear first end",
        "clear second end",
    ] {
        let document = web_sys::window().unwrap().document().unwrap();
        let element: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
        element.set_class_name("xterm");
        document.body().unwrap().append_child(&element).unwrap();
        let controller = BrowserLinkifier::new(
            element.clone(),
            20,
            0,
            Rc::new(|_| Some(Position { x: 2, y: 2 })),
            Rc::new(|_, _| Ok(())),
        )
        .unwrap();
        let setup = Function::new_with_args("mode, element", include_str!("render_recheck.js"))
            .call2(&JsValue::UNDEFINED, &mode.into(), &element)
            .unwrap();
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
        let arm: Function = Reflect::get(&setup, &"arm".into())
            .unwrap()
            .dyn_into()
            .unwrap();
        arm.call0(&setup).unwrap();
        let result = controller.on_render(if mode == "zero start" { 0.0 } else { 1.0 }, 1.0);
        let verify: Function = Reflect::get(&setup, &"verify".into())
            .unwrap()
            .dyn_into()
            .unwrap();
        verify
            .call1(&setup, &result.err().unwrap_or(JsValue::UNDEFINED))
            .unwrap();
        assert_eq!(
            provider.0.borrow().is_some(),
            [
                "plain",
                "zero start",
                "second start outside",
                "second end outside",
                "clear first end"
            ]
            .contains(&mode),
            "{mode}"
        );
        controller.dispose().unwrap();
        element.remove();
    }
}
