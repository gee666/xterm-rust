// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! SmoothScrollableElement owns DOM composition; Scrollable owns numeric state.
#[path = "element_layout.rs"]
mod layout;
#[path = "element_wheel.rs"]
mod wheel;

use super::{
    abstract_scrollbar::{ScrollbarHost, SharedOptions},
    browser_scheduler::{new_scrollable, scroll_error, BrowserScrollable},
    environment::ScrollEnvironment,
    fast_dom_node::FastDomNode,
    horizontal_scrollbar::HorizontalScrollbar,
    interop,
    options::{normalize_input, read_change, ResolvedOptions},
    state::{DimensionUpdate, Position, PositionUpdate, ScrollEvent, ScrollState},
    vertical_scrollbar::VerticalScrollbar,
    wheel_event::StandardWheelEvent,
    widget,
};
use crate::{dom_listener::DomListener, links::animation_queue::WindowAnimationQueue};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, EventTarget, HtmlElement};
use xterm_core::{
    event::{Emitter, Event as CoreEvent},
    lifecycle::{Disposable, DisposableStore},
    options::OptionValue as V,
};

const HIDE_TIMEOUT_MS: f64 = 500.0;
const SCROLL_WHEEL_SENSITIVITY: f64 = 50.0;

pub struct SmoothScrollableElement {
    options: SharedOptions,
    model: BrowserScrollable,
    owns_model: bool,
    environment: Rc<ScrollEnvironment>,
    vertical: RefCell<Option<VerticalScrollbar>>,
    horizontal: RefCell<Option<HorizontalScrollbar>>,
    node: RefCell<Option<HtmlElement>>,
    shadows: RefCell<Option<[FastDomNode; 3]>>,
    listen_on: RefCell<Option<EventTarget>>,
    wheel_listeners: RefCell<Vec<DomListener>>,
    store: DisposableStore,
    hide_timeout: RefCell<Option<JsValue>>,
    hide_ready: Cell<bool>,
    dragging: Cell<bool>,
    mouse_over: Cell<bool>,
    should_render: Cell<bool>,
    disposed: Cell<bool>,
    scroll_event: Emitter<ScrollEvent>,
}
impl SmoothScrollableElement {
    pub fn new(
        element: HtmlElement,
        options: JsValue,
        model: BrowserScrollable,
        environment: Rc<ScrollEnvironment>,
    ) -> Result<Rc<Self>, JsValue> {
        Self::create(element, options, model, environment, false)
    }
    pub fn new_owned(
        element: HtmlElement,
        options: JsValue,
        environment: Rc<ScrollEnvironment>,
        queue: WindowAnimationQueue,
    ) -> Result<Rc<Self>, JsValue> {
        Self::create(
            element,
            options,
            new_scrollable(queue, true, 0.0),
            environment,
            true,
        )
    }
    fn create(
        element: HtmlElement,
        options: JsValue,
        model: BrowserScrollable,
        environment: Rc<ScrollEnvironment>,
        owns_model: bool,
    ) -> Result<Rc<Self>, JsValue> {
        let raw = normalize_input(&options, owns_model)?;
        let options = Rc::new(RefCell::new(ResolvedOptions::resolve(
            &raw,
            environment.platform.is_mac,
        )?));
        let this = Rc::new(Self {
            options,
            model,
            owns_model,
            environment,
            vertical: RefCell::new(None),
            horizontal: RefCell::new(None),
            node: RefCell::new(None),
            shadows: RefCell::new(None),
            listen_on: RefCell::new(None),
            wheel_listeners: RefCell::new(Vec::new()),
            store: DisposableStore::new(),
            hide_timeout: RefCell::new(None),
            hide_ready: Cell::new(false),
            dragging: Cell::new(false),
            mouse_over: Cell::new(false),
            should_render: Cell::new(true),
            disposed: Cell::new(false),
            scroll_event: Emitter::new(),
        });
        let weak = Rc::downgrade(&this);
        this.store
            .add(this.model.on_scroll().try_subscribe(move |event| {
                if let Some(this) = weak.upgrade() {
                    if !this.disposed.get() {
                        this.handle_scroll(event)
                            .map_err(crate::js_error::callback_error)?;
                        this.scroll_event.fire(event)?;
                    }
                }
                Ok(())
            }));
        let start = Rc::downgrade(&this);
        let end = Rc::downgrade(&this);
        let host = ScrollbarHost {
            drag_start: Rc::new(move || {
                if let Some(this) = start.upgrade() {
                    this.dragging.set(true);
                    this.reveal()?;
                }
                Ok(())
            }),
            drag_end: Rc::new(move || {
                if let Some(this) = end.upgrade() {
                    this.dragging.set(false);
                    this.hide()?;
                }
                Ok(())
            }),
            iframe_cache: Rc::clone(&this.environment.iframe),
        };
        *this.vertical.borrow_mut() = Some(VerticalScrollbar::new(
            this.model.clone(),
            this.options.clone(),
            host.clone(),
            this.environment.platform.is_windows,
        )?);
        *this.horizontal.borrow_mut() = Some(HorizontalScrollbar::new(
            this.model.clone(),
            this.options.clone(),
            host,
            this.environment.platform.is_windows,
        )?);
        this.build_dom(element)?;
        this.set_listening_to_wheel()?;
        let target = this.listen_node()?;
        let over = Rc::downgrade(&this);
        let listener = widget::mouse_listener(
            &target,
            "mouseover",
            &this.environment.main_window,
            this.environment.iframe.clone(),
            Rc::new(move |_| {
                if let Some(this) = over.upgrade() {
                    this.mouse_over.set(true);
                    this.reveal()?;
                }
                Ok(())
            }),
        )?;
        super::pointer_monitor::store_listener(&this.store, listener);
        let leave = Rc::downgrade(&this);
        let listener = widget::mouse_listener(
            &target,
            "mouseleave",
            &this.environment.main_window,
            this.environment.iframe.clone(),
            Rc::new(move |_| {
                if let Some(this) = leave.upgrade() {
                    this.mouse_over.set(false);
                    this.hide()?;
                }
                Ok(())
            }),
        )?;
        super::pointer_monitor::store_listener(&this.store, listener);
        this.hide_ready.set(true);
        Ok(this)
    }
    fn vertical(&self) -> Result<VerticalScrollbar, JsValue> {
        self.vertical
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::TypeError::new("vertical scrollbar is not initialized").into())
    }
    fn horizontal(&self) -> Result<HorizontalScrollbar, JsValue> {
        self.horizontal
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::TypeError::new("horizontal scrollbar is not initialized").into())
    }
    fn listen_node(&self) -> Result<HtmlElement, JsValue> {
        self.listen_on
            .borrow()
            .clone()
            .map(|node| node.unchecked_into())
            .ok_or_else(|| js_sys::TypeError::new("scroll listener node is not initialized").into())
    }
    pub fn get_dom_node(&self) -> Result<HtmlElement, JsValue> {
        self.node
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::TypeError::new("scroll node is not initialized").into())
    }
    pub fn options(&self) -> SharedOptions {
        Rc::clone(&self.options)
    }
    pub fn on_scroll(&self) -> CoreEvent<ScrollEvent> {
        self.scroll_event.event()
    }
    pub fn get_scroll_dimensions(&self) -> ScrollState {
        self.model.scroll_state()
    }
    pub fn get_scroll_position(&self) -> Position {
        self.model.current_scroll_position()
    }
    pub fn set_scroll_dimensions(&self, update: DimensionUpdate) -> Result<(), JsValue> {
        self.model
            .set_scroll_dimensions(update, false)
            .map_err(scroll_error)
    }
    pub fn set_scroll_position(
        &self,
        update: PositionUpdate,
        reuse_animation: bool,
    ) -> Result<(), JsValue> {
        if reuse_animation {
            self.model.set_scroll_position_smooth(update, true)
        } else {
            self.model.set_scroll_position_now(update)
        }
        .map_err(scroll_error)
    }
    pub fn delegate_scroll_from_mouse_wheel_event(&self, event: &Event) -> Result<(), JsValue> {
        self.handle_mouse_wheel(&StandardWheelEvent::new(
            Some(event),
            0.0,
            0.0,
            &self.environment,
        )?)
    }
    fn cancel_hide(&self) -> Result<(), JsValue> {
        let pending = self.hide_timeout.borrow().clone();
        if let Some(handle) = pending {
            interop::method1(
                self.environment.main_window.as_ref(),
                "clearTimeout",
                &handle,
            )?;
            self.hide_timeout.borrow_mut().take();
        }
        Ok(())
    }
    fn schedule_hide(self: &Rc<Self>) -> Result<(), JsValue> {
        if self.mouse_over.get() || self.dragging.get() {
            return Ok(());
        }
        if !self.hide_ready.get() {
            return Err(js_sys::TypeError::new("hide timeout is not initialized").into());
        }
        if self.disposed.get() {
            return Err(
                js_sys::Error::new("Calling cancelAndSet on a disposed TimeoutTimer").into(),
            );
        }
        self.cancel_hide()?;
        let weak = Rc::downgrade(self);
        let callback = Closure::<dyn Fn() -> Result<(), JsValue>>::new(move || {
            if let Some(this) = weak.upgrade() {
                this.hide_timeout.borrow_mut().take();
                this.hide()?;
            }
            Ok(())
        })
        .into_js_value();
        let handle = interop::method2(
            self.environment.main_window.as_ref(),
            "setTimeout",
            &callback,
            &HIDE_TIMEOUT_MS.into(),
        )?;
        *self.hide_timeout.borrow_mut() = Some(handle);
        Ok(())
    }
    fn reveal(self: &Rc<Self>) -> Result<(), JsValue> {
        self.vertical()?.begin_reveal()?;
        self.horizontal()?.begin_reveal()?;
        self.schedule_hide()
    }
    fn hide(&self) -> Result<(), JsValue> {
        if !self.mouse_over.get() && !self.dragging.get() {
            self.vertical()?.begin_hide()?;
            self.horizontal()?.begin_hide()?;
        }
        Ok(())
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        if self.disposed.get() {
            return Ok(());
        }
        self.clear_wheel_listeners()?;
        self.store.dispose();
        if self.owns_model {
            self.model.dispose().map_err(scroll_error)?;
        }
        if let Some(vertical) = self.vertical.borrow().clone() {
            vertical.dispose()?;
        }
        if let Some(horizontal) = self.horizontal.borrow().clone() {
            horizontal.dispose()?;
        }
        self.cancel_hide()?;
        self.scroll_event.dispose();
        self.disposed.set(true);
        Ok(())
    }
}
impl Drop for SmoothScrollableElement {
    fn drop(&mut self) {
        if let Err(error) = self.dispose() {
            crate::write_scheduler::report_unhandled_error(&error);
        }
    }
}
