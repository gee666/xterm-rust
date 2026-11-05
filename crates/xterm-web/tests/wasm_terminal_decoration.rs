#![cfg(target_arch = "wasm32")]
//! Real Terminal ownership, independent of the controlled component fixture.
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_core::{circular_list::ListChange, decoration::DecorationOptions};
use xterm_web::terminal::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

async fn frame() {
    let window = web_sys::window().unwrap();
    let next = js_sys::Promise::new(&mut |resolve, _| {
        window.request_animation_frame(&resolve).unwrap();
    });
    JsFuture::from(next).await.unwrap();
}

#[wasm_bindgen_test(async)]
async fn pre_open_cache_sync_and_live_overlay_survive_terminal_transitions() {
    let document = web_sys::window().unwrap().document().unwrap();
    let parent = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
    let terminal = Terminal::new(js_sys::Object::new().into()).unwrap();
    let buffers = terminal
        .shared_native()
        .with_state(|state| Rc::clone(&state.buffers))
        .unwrap();
    let marker = buffers.normal().borrow_mut().add_marker(0);
    let service = terminal.decoration_service();
    let decoration = service
        .register_decoration(Rc::new(RefCell::new(DecorationOptions::new(marker))))
        .unwrap()
        .unwrap();
    buffers
        .normal()
        .borrow()
        .lines
        .on_insert_emitter
        .fire(&ListChange {
            index: 0,
            amount: 1,
        })
        .unwrap();
    assert!(service.line_cache().has_pending_sync());
    JsFuture::from(js_sys::Promise::resolve(&JsValue::UNDEFINED))
        .await
        .unwrap();
    assert!(!service.line_cache().has_pending_sync());
    assert!(service
        .get_decorations_at_cell(0.0, 1, None)
        .next()
        .unwrap()
        .ptr_eq(&decoration));
    assert!(decoration.element().is_none());
    let observations = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&observations);
    let rendered = decoration.clone();
    let on_render = Closure::wrap(Box::new(move |_: JsValue| {
        output.borrow_mut().push(rendered.element().is_some());
    }) as Box<dyn FnMut(JsValue)>);
    let _subscription = terminal
        .on_render()
        .call1(&JsValue::UNDEFINED, on_render.as_ref())
        .unwrap();

    terminal.open(parent.clone().into()).unwrap();
    terminal.refresh(0.0, 23.0).unwrap();
    frame().await;
    let node = decoration
        .element()
        .unwrap()
        .downcast_ref::<web_sys::HtmlElement>()
        .unwrap()
        .clone();
    assert!(node.is_connected());
    assert_eq!(
        observations.borrow().first(),
        Some(&false),
        "public render fires before decoration rendering"
    );
    assert_eq!(
        parent
            .query_selector_all(".xterm-decoration-container")
            .unwrap()
            .length(),
        1
    );
    buffers.activate_alt_buffer(None).unwrap();
    terminal.refresh(0.0, 23.0).unwrap();
    frame().await;
    assert_eq!(node.style().get_property_value("display").unwrap(), "none");
    buffers.activate_normal_buffer().unwrap();
    terminal.resize(90.0, 30.0).unwrap();
    terminal.refresh(0.0, 29.0).unwrap();
    frame().await;
    assert_eq!(node.style().get_property_value("display").unwrap(), "block");
    assert!(node.is_connected());
    terminal.reset().unwrap();
    assert!(!node.is_connected());
    assert!(decoration.is_disposed());
    assert!(service.decorations().unwrap().next().is_none());

    let marker = buffers.normal().borrow_mut().add_marker(0);
    let pending = service
        .register_decoration(Rc::new(RefCell::new(DecorationOptions::new(marker))))
        .unwrap()
        .unwrap();
    terminal.dispose().unwrap();
    frame().await;
    assert!(
        pending.element().is_none(),
        "whole-terminal disposal cancels queued overlay work"
    );
    assert_eq!(parent.child_element_count(), 0);
    parent.remove();
}

#[wasm_bindgen_test(async)]
async fn pre_open_reset_and_disposal_clear_queued_cache_mutations() {
    let terminal = Terminal::new(js_sys::Object::new().into()).unwrap();
    let buffers = terminal
        .shared_native()
        .with_state(|state| Rc::clone(&state.buffers))
        .unwrap();
    let service = terminal.decoration_service();
    let register = || {
        let marker = buffers.normal().borrow_mut().add_marker(0);
        service
            .register_decoration(Rc::new(RefCell::new(DecorationOptions::new(marker))))
            .unwrap()
            .unwrap()
    };
    let insert = || {
        buffers
            .normal()
            .borrow()
            .lines
            .on_insert_emitter
            .fire(&ListChange {
                index: 0,
                amount: 1,
            })
            .unwrap();
    };
    let original = register();
    insert();
    assert!(service.line_cache().has_pending_sync());
    terminal.reset().unwrap();
    assert!(original.is_disposed());
    let fresh = register();
    JsFuture::from(js_sys::Promise::resolve(&JsValue::UNDEFINED))
        .await
        .unwrap();
    assert!(!service.line_cache().has_pending_sync());
    assert!(
        service
            .get_decorations_at_cell(0.0, 0, None)
            .next()
            .unwrap()
            .ptr_eq(&fresh),
        "old insert must not shift a post-reset decoration"
    );
    assert!(service
        .get_decorations_at_cell(0.0, 1, None)
        .next()
        .is_none());
    insert();
    assert!(service.line_cache().has_pending_sync());
    terminal.dispose().unwrap();
    assert!(!service.line_cache().has_pending_sync());
    assert!(fresh.is_disposed());
    JsFuture::from(js_sys::Promise::resolve(&JsValue::UNDEFINED))
        .await
        .unwrap();
    assert!(service.decorations().unwrap().next().is_none());
}
