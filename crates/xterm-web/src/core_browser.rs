// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Live CoreBrowserService. Focus, owning window and main document are distinct.
#[path = "core_browser_dpr.rs"]
mod dpr;
#[path = "core_browser_host.rs"]
mod host;
use crate::renderer::service_api::BrowserService;
use dpr::ScreenDprMonitor;
use host::DefaultDomListener;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Document, HtmlTextAreaElement, Window};
use xterm_core::{
    event::{Emitter, Event},
    lifecycle::{Disposable, DisposableHandle},
};

pub struct CoreBrowserService {
    textarea: HtmlTextAreaElement,
    window: RefCell<Window>,
    main_document: Document,
    focused: Cell<bool>,
    cached: Rc<Cell<Option<bool>>>,
    monitor: Rc<ScreenDprMonitor>,
    dpr_changed: Emitter<f64>,
    window_changed: Emitter<Window>,
    subscriptions: RefCell<Vec<DisposableHandle>>,
    focus: RefCell<Option<Rc<DefaultDomListener>>>,
    blur: RefCell<Option<Rc<DefaultDomListener>>>,
    disposed: Cell<bool>,
}
impl CoreBrowserService {
    pub fn new(
        textarea: &HtmlTextAreaElement,
        window: &Window,
        main_document: &Document,
    ) -> Result<Rc<Self>, JsValue> {
        let monitor = ScreenDprMonitor::new(window)?;
        let this = Rc::new(Self {
            textarea: textarea.clone(),
            window: RefCell::new(window.clone()),
            main_document: main_document.clone(),
            focused: Cell::new(false),
            cached: Rc::new(Cell::new(None)),
            monitor,
            dpr_changed: Emitter::new(),
            window_changed: Emitter::new(),
            subscriptions: RefCell::new(Vec::new()),
            focus: RefCell::new(None),
            blur: RefCell::new(None),
            disposed: Cell::new(false),
        });
        let weak = Rc::downgrade(&this.monitor);
        let window_subscription = this.window_changed.event().try_subscribe(move |window| {
            if let Some(monitor) = weak.upgrade() {
                monitor
                    .set_window(window)
                    .map_err(crate::js_error::callback_error)?;
            }
            Ok(())
        });
        this.subscriptions.borrow_mut().push(window_subscription);
        let weak = Rc::downgrade(&this);
        let dpr_subscription = this.monitor.event().try_subscribe(move |value| {
            if let Some(this) = weak.upgrade() {
                this.dpr_changed.fire(value)?;
            }
            Ok(())
        });
        this.subscriptions.borrow_mut().push(dpr_subscription);
        let weak = Rc::downgrade(&this);
        let focus = Closure::wrap(Box::new(move |_: JsValue| {
            if let Some(this) = weak.upgrade() {
                this.focused.set(true);
            }
        }) as Box<dyn Fn(JsValue)>);
        let focus = DefaultDomListener::new(
            textarea.as_ref(),
            "focus",
            focus.into_js_value().unchecked_into(),
        );
        *this.focus.borrow_mut() = Some(Rc::clone(&focus));
        focus.add()?;
        let weak = Rc::downgrade(&this);
        let blur = Closure::wrap(Box::new(move |_: JsValue| {
            if let Some(this) = weak.upgrade() {
                this.focused.set(false);
            }
        }) as Box<dyn Fn(JsValue)>);
        let blur = DefaultDomListener::new(
            textarea.as_ref(),
            "blur",
            blur.into_js_value().unchecked_into(),
        );
        *this.blur.borrow_mut() = Some(Rc::clone(&blur));
        blur.add()?;
        Ok(this)
    }
    pub fn window(&self) -> Window {
        self.window.borrow().clone()
    }
    pub fn main_document(&self) -> Document {
        self.main_document.clone()
    }
    pub fn set_window(&self, value: &Window) -> Result<(), JsValue> {
        if self.window() == *value {
            return Ok(());
        }
        *self.window.borrow_mut() = value.clone();
        self.window_changed
            .fire(value)
            .map_err(|error| crate::js_error::to_js_error(&error))
    }
    pub fn dpr(&self) -> Result<f64, JsValue> {
        host::number(self.window().as_ref(), "devicePixelRatio")
    }
    pub fn is_focused(&self) -> Result<bool, JsValue> {
        if let Some(value) = self.cached.get() {
            return Ok(value);
        }
        let value = if self.focused.get() {
            let owner = js_sys::Reflect::get(&self.textarea, &JsValue::from_str("ownerDocument"))?;
            host::invoke0(&owner, "hasFocus")?
                .as_bool()
                .ok_or_else(|| js_sys::TypeError::new("Document.hasFocus must return a boolean"))?
        } else {
            false
        };
        self.cached.set(Some(value));
        let cache = Rc::clone(&self.cached);
        let callback = Closure::once_into_js(move |cancelled: bool| {
            if !cancelled {
                cache.set(None);
            }
        });
        // Accepted jobs still clear the cache if the enqueue hook then throws.
        // Failed-before-enqueue jobs release their one-shot capture next turn.
        host::queue_cache_reset(&callback)?;
        Ok(value)
    }
    pub fn on_dpr_change(&self) -> Event<f64> {
        self.dpr_changed.event()
    }
    pub fn on_window_change(&self) -> Event<Window> {
        self.window_changed.event()
    }
    pub fn is_disposed(&self) -> bool {
        self.disposed.get()
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        if self.disposed.replace(true) {
            return Ok(());
        }
        // Source registers public emitters before the monitor, forwarding and
        // textarea listeners. Preserve that ordering and stop at the first error.
        self.dpr_changed.dispose();
        self.window_changed.dispose();
        self.monitor.dispose()?;
        let subscriptions = std::mem::take(&mut *self.subscriptions.borrow_mut());
        for subscription in subscriptions {
            subscription.dispose();
        }
        Self::remove_listener(&self.focus)?;
        Self::remove_listener(&self.blur)
    }
    fn remove_listener(slot: &RefCell<Option<Rc<DefaultDomListener>>>) -> Result<(), JsValue> {
        let listener = slot.borrow().clone();
        if let Some(listener) = listener {
            listener.remove()?;
        }
        slot.borrow_mut().take();
        Ok(())
    }
}
impl BrowserService for CoreBrowserService {
    fn window(&self) -> Window {
        CoreBrowserService::window(self)
    }
    fn dpr(&self) -> Result<f64, JsValue> {
        CoreBrowserService::dpr(self)
    }
    fn is_focused(&self) -> Result<bool, JsValue> {
        CoreBrowserService::is_focused(self)
    }
    fn on_dpr_change(&self) -> Event<f64> {
        CoreBrowserService::on_dpr_change(self)
    }
    fn on_window_change(&self) -> Event<Window> {
        CoreBrowserService::on_window_change(self)
    }
}
impl Drop for CoreBrowserService {
    fn drop(&mut self) {
        if let Err(error) = self.dispose() {
            host::report_drop_error(&error);
        }
        if let Some(listener) = self.focus.get_mut().take() {
            listener.cleanup();
        }
        if let Some(listener) = self.blur.get_mut().take() {
            listener.cleanup();
        }
    }
}
