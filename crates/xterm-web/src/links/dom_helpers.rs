// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use wasm_bindgen::JsValue;
use web_sys::{HtmlElement, Node, UiEvent, Window};

/// Prefer an owner document's window, then an event's view, then the caller's
/// global window. The explicit fallback permits iframe and multi-window tests.
pub fn get_window(node: Option<&Node>, event: Option<&UiEvent>, fallback: &Window) -> Window {
    node.and_then(Node::owner_document)
        .and_then(|document| document.default_view())
        .or_else(|| event.and_then(UiEvent::view))
        .unwrap_or_else(|| fallback.clone())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PagePosition {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

pub fn page_position(node: &HtmlElement, fallback: &Window) -> Result<PagePosition, JsValue> {
    let bounds = node.get_bounding_client_rect();
    let window = get_window(Some(node.as_ref()), None, fallback);
    Ok(PagePosition {
        left: bounds.left() + window.scroll_x()?,
        top: bounds.top() + window.scroll_y()?,
        width: bounds.width(),
        height: bounds.height(),
    })
}

pub mod event_type {
    pub const CLICK: &str = "click";
    pub const MOUSE_DOWN: &str = "mousedown";
    pub const MOUSE_OVER: &str = "mouseover";
    pub const MOUSE_LEAVE: &str = "mouseleave";
    pub const KEY_DOWN: &str = "keydown";
    pub const KEY_UP: &str = "keyup";
    pub const INPUT: &str = "input";
    pub const BLUR: &str = "blur";
    pub const FOCUS: &str = "focus";
    pub const CHANGE: &str = "change";
    pub const POINTER_DOWN: &str = "pointerdown";
    pub const POINTER_MOVE: &str = "pointermove";
    pub const POINTER_UP: &str = "pointerup";
    pub const MOUSE_WHEEL: &str = "wheel";
    pub const WHEEL: &str = "wheel";
}
