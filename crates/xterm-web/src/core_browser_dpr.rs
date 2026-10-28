// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! ScreenDprMonitor and its source-order, reentrant resize-disposable semantics.
use super::host::{invoke1, number, report_drop_error, DefaultDomListener as ResizeListener};
use js_sys::Function;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::Window;
use xterm_core::{
    event::{Emitter, Event},
    lifecycle::Disposable,
};

pub(super) struct ScreenDprMonitor {
    parent: RefCell<Window>,
    current: Cell<Option<f64>>,
    callback: RefCell<Option<Function>>,
    media: RefCell<Option<JsValue>>,
    resize: RefCell<Option<Rc<ResizeListener>>>,
    // Source can abandon a newly added listener if old-listener disposal throws
    // or reenters. Preserve it while live, but retain an owner for Rust Drop.
    abandoned: RefCell<Vec<Rc<ResizeListener>>>,
    changed: Emitter<f64>,
    disposed: Cell<bool>,
    weak: Weak<Self>,
}
impl ScreenDprMonitor {
    pub(super) fn new(window: &Window) -> Result<Rc<Self>, JsValue> {
        let this = Rc::new_cyclic(|weak| Self {
            parent: RefCell::new(window.clone()),
            current: Cell::new(None),
            callback: RefCell::new(None),
            media: RefCell::new(None),
            resize: RefCell::new(None),
            abandoned: RefCell::new(Vec::new()),
            changed: Emitter::new(),
            disposed: Cell::new(false),
            weak: weak.clone(),
        });
        let weak = Rc::downgrade(&this);
        let callback = Closure::wrap(Box::new(move |_: JsValue| {
            if let Some(this) = weak.upgrade() {
                this.check_and_rearm()?;
            }
            Ok(())
        }) as Box<dyn Fn(JsValue) -> Result<(), JsValue>>);
        let callback = callback.into_js_value().unchecked_into();
        *this.callback.borrow_mut() = Some(callback);
        this.current
            .set(Some(number(this.window().as_ref(), "devicePixelRatio")?));
        this.rearm()?;
        this.replace_resize_listener()?;
        Ok(this)
    }
    fn window(&self) -> Window {
        self.parent.borrow().clone()
    }
    fn callback_value(&self) -> JsValue {
        self.callback
            .borrow()
            .as_ref()
            .map(JsValue::from)
            .unwrap_or(JsValue::UNDEFINED)
    }
    pub(super) fn event(&self) -> Event<f64> {
        self.changed.event()
    }
    pub(super) fn set_window(&self, window: &Window) -> Result<(), JsValue> {
        *self.parent.borrow_mut() = window.clone();
        self.replace_resize_listener()?;
        self.check_and_rearm()
    }
    fn replace_resize_listener(&self) -> Result<(), JsValue> {
        let weak = self.weak.clone();
        let callback = Closure::wrap(Box::new(move |_: JsValue| {
            if let Some(this) = weak.upgrade() {
                this.check_and_rearm()?;
            }
            Ok(())
        }) as Box<dyn Fn(JsValue) -> Result<(), JsValue>>);
        let callback = callback.into_js_value().unchecked_into();
        let next = ResizeListener::new(self.window().as_ref(), "resize", callback);
        if let Err(error) = next.add() {
            self.abandoned.borrow_mut().push(next);
            return Err(error);
        }
        // MutableDisposable checks its closed flag after RHS listener creation.
        if self.disposed.get() {
            self.abandoned.borrow_mut().push(next);
            return Ok(());
        }
        let old = self.resize.borrow().clone();
        if let Some(old) = &old {
            if let Err(error) = old.remove() {
                self.abandoned.borrow_mut().push(next);
                return Err(error);
            }
        }
        let replaced = self.resize.borrow_mut().replace(next);
        if let Some(replaced) = replaced {
            if old.as_ref().is_none_or(|old| !Rc::ptr_eq(old, &replaced)) {
                self.abandoned.borrow_mut().push(replaced);
            }
        }
        Ok(())
    }
    fn check_and_rearm(&self) -> Result<(), JsValue> {
        let value = number(self.window().as_ref(), "devicePixelRatio")?;
        if self.current.get().is_none_or(|old| old != value) {
            let payload = number(self.window().as_ref(), "devicePixelRatio")?;
            self.changed
                .fire(&payload)
                .map_err(|error| crate::js_error::to_js_error(&error))?;
        }
        self.rearm()
    }
    fn rearm(&self) -> Result<(), JsValue> {
        if self.callback.borrow().is_none() {
            return Ok(());
        }
        let old = self.media.borrow().clone();
        if let Some(old) = old.filter(|value| !value.is_null() && !value.is_undefined()) {
            invoke1(&old, "removeListener", || Ok(self.callback_value()))?;
        }
        self.current
            .set(Some(number(self.window().as_ref(), "devicePixelRatio")?));
        let receiver = self.window();
        let media = invoke1(receiver.as_ref(), "matchMedia", || {
            let value = number(self.window().as_ref(), "devicePixelRatio")?;
            Ok(JsValue::from_str(&format!(
                "screen and (resolution: {}dppx)",
                ryu_js::Buffer::new().format(value)
            )))
        })?;
        *self.media.borrow_mut() = Some(media.clone());
        invoke1(&media, "addListener", || Ok(self.callback_value()))?;
        Ok(())
    }
    fn clear_listener(&self) -> Result<(), JsValue> {
        let media = self.media.borrow().clone();
        let Some(media) = media.filter(JsValue::is_truthy) else {
            return Ok(());
        };
        if self.callback.borrow().is_none() {
            return Ok(());
        }
        invoke1(&media, "removeListener", || Ok(self.callback_value()))?;
        self.media.borrow_mut().take();
        self.callback.borrow_mut().take();
        Ok(())
    }
    pub(super) fn dispose(&self) -> Result<(), JsValue> {
        if self.disposed.replace(true) {
            return Ok(());
        }
        let resize = self.resize.borrow().clone();
        if let Some(resize) = resize {
            resize.remove()?;
        }
        self.resize.borrow_mut().take();
        self.changed.dispose();
        self.clear_listener()
    }
}
impl Drop for ScreenDprMonitor {
    fn drop(&mut self) {
        if let Err(error) = self.dispose() {
            report_drop_error(&error);
        }
        if let Some(resize) = self.resize.get_mut().take() {
            resize.cleanup();
        }
        for resize in self.abandoned.get_mut().drain(..) {
            resize.cleanup();
        }
    }
}
