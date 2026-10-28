// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under MIT. Port of scrollable/abstractScrollbar.ts.

use super::{
    arrow::ScrollbarArrow,
    browser_scheduler::BrowserScrollable,
    browser_visibility::{new_visibility, BrowserVisibility},
    fast_dom_node::FastDomNode,
    horizontal_scrollbar,
    iframe::IframeCache,
    options::ResolvedOptions,
    pointer_monitor::{store_listener, GlobalPointerMoveMonitor},
    scrollbar_state::ScrollbarState,
    state::{PositionUpdate, ScrollEvent},
    vertical_scrollbar,
};
use crate::dom_listener::DomListener;
use std::{
    cell::{Cell, OnceCell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Event, HtmlElement};
use xterm_core::{
    lifecycle::{Disposable, DisposableStore},
    options::OptionValue as V,
};

#[path = "scrollbar_events.rs"]
mod events;
#[path = "scrollbar_layout.rs"]
mod layout;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}
impl Axis {
    fn class_name(self) -> &'static str {
        match self {
            Self::Horizontal => "xterm-horizontal",
            Self::Vertical => "xterm-vertical",
        }
    }
    pub fn write_position(self, target: &mut PositionUpdate, position: f64) {
        match self {
            Self::Horizontal => target.left = Some(position),
            Self::Vertical => target.top = Some(position),
        }
    }
}
pub type SharedOptions = Rc<RefCell<ResolvedOptions>>;
pub type DragCallback = Rc<dyn Fn() -> Result<(), JsValue>>;
#[derive(Clone)]
pub struct ScrollbarHost {
    pub drag_start: DragCallback,
    pub drag_end: DragCallback,
    pub iframe_cache: Rc<IframeCache>,
}
pub(super) struct Inner {
    pub(super) axis: Axis,
    pub(super) host: ScrollbarHost,
    pub(super) model: BrowserScrollable,
    pub(super) state: RefCell<ScrollbarState>,
    pub(super) visibility: BrowserVisibility,
    pub(super) node: FastDomNode,
    pub(super) slider: OnceCell<FastDomNode>,
    pub(super) arrows: RefCell<Vec<Rc<ScrollbarArrow>>>,
    pub(super) arrow_delta: RefCell<V>,
    pub(super) monitor: GlobalPointerMoveMonitor,
    pub(super) root_listeners: DisposableStore,
    pub(super) slider_listeners: DisposableStore,
    pub(super) lazy_render: bool,
    pub(super) scroll_by_page: Cell<bool>,
    pub(super) should_render: Cell<bool>,
    pub(super) disposed: Cell<bool>,
    pub(super) is_windows: bool,
}

/// Oriented wrappers keep source constructor names and share all drag/state code.
#[derive(Clone)]
pub struct Scrollbar {
    pub(super) inner: Rc<Inner>,
}
impl Scrollbar {
    pub fn new(
        axis: Axis,
        model: BrowserScrollable,
        options: SharedOptions,
        host: ScrollbarHost,
        is_windows: bool,
    ) -> Result<Self, JsValue> {
        let initial = layout::prepare(axis, &model, &options)?;
        let window = web_sys::window()
            .ok_or_else(|| js_sys::Error::new("scrollbar requires a browser window"))?;
        let visibility = new_visibility(
            window.clone(),
            initial.visibility,
            format!("xterm-visible xterm-scrollbar {}", axis.class_name()),
            format!("xterm-invisible xterm-scrollbar {}", axis.class_name()),
        )?;
        visibility.set_is_needed(initial.state.is_needed())?;
        let document = window
            .document()
            .ok_or_else(|| js_sys::Error::new("scrollbar requires a document"))?;
        let node = FastDomNode::new(document.create_element("div")?.dyn_into::<HtmlElement>()?);
        node.set_attribute("role", "presentation")?;
        node.set_attribute("aria-hidden", "true")?;
        visibility.set_dom_node(node.clone())?;
        node.set_position("absolute")?;
        let inner = Rc::new(Inner {
            axis,
            host,
            model,
            state: RefCell::new(initial.state),
            visibility,
            node,
            slider: OnceCell::new(),
            arrows: RefCell::new(Vec::new()),
            arrow_delta: RefCell::new(0.0.into()),
            monitor: GlobalPointerMoveMonitor::new(),
            root_listeners: DisposableStore::new(),
            slider_listeners: DisposableStore::new(),
            lazy_render: initial.lazy_render,
            scroll_by_page: Cell::new(initial.scroll_by_page),
            should_render: Cell::new(true),
            disposed: Cell::new(false),
            is_windows,
        });
        let weak = Rc::downgrade(&inner);
        let listener = DomListener::new_reentrant_fallible(
            inner.node.dom_node.as_ref(),
            "pointerdown",
            false,
            None,
            move |event| {
                if let Some(inner) = weak.upgrade() {
                    let target = js_sys::Reflect::get(event.as_ref(), &"target".into())?;
                    let node: &JsValue = inner.node.dom_node.as_ref();
                    if target == *node {
                        inner.handle_pointer_down(&event)?;
                    }
                }
                Ok(())
            },
        )?;
        store_listener(&inner.root_listeners, listener);
        match axis {
            Axis::Horizontal => horizontal_scrollbar::initialize(&inner, &options)?,
            Axis::Vertical => vertical_scrollbar::initialize(&inner, &options, initial.has_arrows)?,
        }
        Ok(Self { inner })
    }
    pub fn dom_node(&self) -> HtmlElement {
        self.inner.node.dom_node.clone()
    }
    pub fn slider_node(&self) -> Result<HtmlElement, JsValue> {
        Ok(self.inner.slider()?.dom_node.clone())
    }
    pub fn begin_reveal(&self) -> Result<(), JsValue> {
        self.inner.visibility.set_should_be_visible(true)
    }
    pub fn begin_hide(&self) -> Result<(), JsValue> {
        self.inner.visibility.set_should_be_visible(false)
    }
    pub fn render(&self) -> Result<(), JsValue> {
        self.inner.render()
    }
    pub fn is_needed(&self) -> bool {
        self.inner.state.borrow().is_needed()
    }
    pub fn delegate_pointer_down(&self, event: &Event) -> Result<(), JsValue> {
        self.inner.delegate_pointer_down(event)
    }
    pub fn write_scroll_position(&self, target: &mut PositionUpdate, position: f64) {
        self.inner.axis.write_position(target, position)
    }
    pub fn update_scrollbar_size(&self, size: &V) -> Result<(), JsValue> {
        self.inner.update_scrollbar_size(size)
    }
    pub fn update_options(&self, options: &SharedOptions) -> Result<(), JsValue> {
        match self.inner.axis {
            Axis::Horizontal => horizontal_scrollbar::update_options(&self.inner, options),
            Axis::Vertical => vertical_scrollbar::update_options(&self.inner, options),
        }
    }
    pub fn handle_scroll(&self, event: &ScrollEvent) -> Result<bool, JsValue> {
        self.inner.handle_scroll(event)
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.inner.dispose()
    }
}
impl Inner {
    pub(super) fn slider(&self) -> Result<&FastDomNode, JsValue> {
        self.slider
            .get()
            .ok_or_else(|| js_sys::TypeError::new("scrollbar slider has not been created").into())
    }
    pub(super) fn dispose(&self) -> Result<(), JsValue> {
        if self.disposed.replace(true) {
            return Ok(());
        }
        self.visibility.dispose()?;
        self.monitor.dispose();
        self.root_listeners.dispose();
        let arrows = self.arrows.borrow().clone();
        for arrow in arrows {
            arrow.dispose()?;
        }
        self.slider_listeners.dispose();
        Ok(())
    }
}
impl Drop for Inner {
    fn drop(&mut self) {
        if let Err(error) = self.dispose() {
            web_sys::console::error_1(&error);
        }
    }
}
