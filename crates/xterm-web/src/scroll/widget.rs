// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
// Composition equivalent of browser/scrollable/widget.ts's protected listeners.

use super::interop::event_window;
use super::mouse_event::{IframeCache, StandardMouseEvent};
use super::pointer_monitor::store_listener;
use crate::dom_listener::DomListener;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{EventTarget, HtmlElement, Window};
use xterm_core::lifecycle::DisposableStore;

pub type MouseListener = Rc<dyn Fn(StandardMouseEvent) -> Result<(), JsValue>>;

/// Obtain the current node window at dispatch, not registration: the node may
/// have been adopted. The caller's existing store owns the actual DOM listener.
pub fn mouse_listener(
    node: &HtmlElement,
    event_type: &str,
    fallback: &Window,
    cache: Rc<IframeCache>,
    listener: MouseListener,
) -> Result<DomListener, JsValue> {
    let target: EventTarget = node.clone().into();
    let source = node.clone();
    let fallback = fallback.clone();
    DomListener::new_reentrant_fallible(&target, event_type, false, None, move |event| {
        let window = event_window(source.as_ref(), &fallback)?;
        // Source getWindow passes its result without an instanceof check. This
        // opaque JS wrapper adds no eager error; IframeCache performs the same
        // subsequent raw property/WeakMap operations as the source constructor.
        let window: Window = window.unchecked_into();
        let event = StandardMouseEvent::new(&window, &event, &cache)?;
        listener(event)
    })
}

pub fn on_click(
    store: &DisposableStore,
    node: &HtmlElement,
    fallback: &Window,
    cache: Rc<IframeCache>,
    listener: MouseListener,
) -> Result<(), JsValue> {
    store_listener(
        store,
        mouse_listener(node, "click", fallback, cache, listener)?,
    );
    Ok(())
}

pub fn on_mouseover(
    store: &DisposableStore,
    node: &HtmlElement,
    fallback: &Window,
    cache: Rc<IframeCache>,
    listener: MouseListener,
) -> Result<(), JsValue> {
    store_listener(
        store,
        mouse_listener(node, "mouseover", fallback, cache, listener)?,
    );
    Ok(())
}

pub fn on_mouseleave(
    store: &DisposableStore,
    node: &HtmlElement,
    fallback: &Window,
    cache: Rc<IframeCache>,
    listener: MouseListener,
) -> Result<(), JsValue> {
    store_listener(
        store,
        mouse_listener(node, "mouseleave", fallback, cache, listener)?,
    );
    Ok(())
}
