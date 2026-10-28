// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. Live AccessibilityManager DOM and event ownership.
use super::{Announcements, CharacterAction, LocalizableStrings};
use crate::{dom_listener::DomListener, links::browser_debounce::BrowserTimeDebouncer, utf16};
use std::{
    cell::{Cell, OnceCell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{Document, HtmlElement, Window};
use xterm_core::lifecycle::DisposableHandle;

#[derive(Clone, Copy, Debug, Default)]
pub struct Dimensions {
    pub cell_width: f64,
    pub cell_height: f64,
    pub canvas_width: f64,
    pub font_size: f64,
}
#[derive(Debug, Clone)]
pub struct Row {
    pub text: Vec<u16>,
    pub columns: Vec<usize>,
}
#[derive(Clone, Debug)]
pub enum AccessibilityEvent {
    Resize(usize),
    Render { start: f64, end: f64 },
    Scroll,
    Character(Vec<u16>),
    LineFeed,
    Tab(usize),
    Key(Vec<u16>),
    Blur,
    Dimensions,
}
pub type Listener = Rc<dyn Fn(AccessibilityEvent) -> Result<(), JsValue>>;

/// One captured buffer identity for a render batch. Reads stay live within
/// that buffer, but a terminal reset must not redirect later rows to a new one.
pub trait AccessibilityBuffer {
    fn display_offset(&self) -> Result<usize, JsValue>;
    fn line_count(&self) -> Result<usize, JsValue>;
    fn row(&self, absolute: usize) -> Result<Option<Row>, JsValue>;
}

/// The real terminal supplies live snapshots and shares its existing selection
/// service. Host callbacks must not retain a terminal-state borrow while calling
/// subscribers; DOM focus and selection can synchronously reenter the terminal.
pub trait AccessibilityHost {
    fn rows(&self) -> Result<usize, JsValue>;
    fn cols(&self) -> Result<usize, JsValue>;
    fn render_buffer(self: Rc<Self>) -> Result<Rc<dyn AccessibilityBuffer>, JsValue>;
    fn buffer_length(&self) -> Result<usize, JsValue>;
    fn dimensions(&self) -> Result<Dimensions, JsValue>;
    fn scroll_lines(&self, amount: i32) -> Result<(), JsValue>;
    fn clear_selection(&self) -> Result<(), JsValue>;
    fn select(&self, column: f64, row: f64, length: f64) -> Result<(), JsValue>;
    fn subscribe(&self, listener: Listener) -> Result<Vec<DisposableHandle>, JsValue>;
}

pub(super) struct TreeRow {
    pub element: HtmlElement,
}
pub(super) struct Inner {
    pub document: Document,
    pub host: Rc<dyn AccessibilityHost>,
    pub strings: LocalizableStrings,
    pub container: HtmlElement,
    pub row_container: HtmlElement,
    pub live_region: HtmlElement,
    pub rows: RefCell<Vec<Rc<TreeRow>>>,
    pub column_map: js_sys::WeakMap,
    pub announcements: RefCell<Announcements>,
    pub boundaries: OnceCell<(js_sys::Function, js_sys::Function)>,
    timer: RefCell<Option<Rc<BrowserTimeDebouncer>>>,
    subscriptions: RefCell<Vec<DisposableHandle>>,
    selection_listener: RefCell<Option<DomListener>>,
    disposed: Cell<bool>,
}

/// An accessibility tree attached to the actual terminal DOM. Cloneable handles
/// share one owner. DOM/event/timer closures hold weak references, not cycles.
#[derive(Clone)]
pub struct AccessibilityManager {
    pub(super) inner: Rc<Inner>,
}
impl AccessibilityManager {
    pub fn new(
        window: Window,
        document: Document,
        root: Option<&HtmlElement>,
        host: Rc<dyn AccessibilityHost>,
        strings: LocalizableStrings,
    ) -> Result<Self, JsValue> {
        let container = element(&document, "xterm-accessibility")?;
        let row_container = element(&document, "xterm-accessibility-tree")?;
        row_container.set_attribute("role", "list")?;
        container.append_child(&row_container)?;
        let live_region = element(&document, "live-region")?;
        live_region.set_attribute("aria-live", "assertive")?;
        container.append_child(&live_region)?;
        let inner = Rc::new(Inner {
            document,
            host,
            strings,
            container,
            row_container,
            live_region,
            rows: RefCell::default(),
            column_map: js_sys::WeakMap::new(),
            announcements: RefCell::default(),
            boundaries: OnceCell::new(),
            timer: RefCell::new(None),
            subscriptions: RefCell::default(),
            selection_listener: RefCell::new(None),
            disposed: Cell::new(false),
        });
        let count = inner.host.rows()?;
        if count == 0 {
            return Err(
                js_sys::Error::new("accessibility requires at least one terminal row").into(),
            );
        }
        for _ in 0..count {
            let row = inner.create_row()?;
            inner.rows.borrow_mut().push(Rc::clone(&row));
            inner.row_container.append_child(&row.element)?;
        }
        inner.initialize_boundaries()?;
        inner.attach_boundaries()?;
        let weak = Rc::downgrade(&inner);
        let timer = BrowserTimeDebouncer::try_new(
            window,
            crate::links::time_debounce::DEFAULT_DEBOUNCE_MS,
            move |range| {
                if let Some(inner) = weak.upgrade() {
                    inner.render_rows(range.start, range.end)
                } else {
                    Ok(())
                }
            },
        )?;
        inner.timer.replace(Some(Rc::new(timer)));
        let root = root.ok_or_else(|| {
            js_sys::Error::new("Cannot enable accessibility before Terminal.open")
        })?;
        root.insert_before(&inner.container, root.first_child().as_ref())?;
        let weak = Rc::downgrade(&inner);
        let subscriptions = inner.host.subscribe(Rc::new(move |event| {
            if let Some(inner) = weak.upgrade() {
                inner.handle_event(event)
            } else {
                Ok(())
            }
        }))?;
        inner.subscriptions.replace(subscriptions);
        let weak = Rc::downgrade(&inner);
        let listener = DomListener::new_reentrant_fallible(
            inner.document.as_ref(),
            "selectionchange",
            false,
            None,
            move |_| {
                if let Some(inner) = weak.upgrade() {
                    inner.selection_change()
                } else {
                    Ok(())
                }
            },
        )?;
        inner.selection_listener.replace(Some(listener));
        inner.refresh_dimensions()?;
        inner.refresh(None, None)?;
        Ok(Self { inner })
    }
    pub fn container(&self) -> HtmlElement {
        self.inner.container.clone()
    }
    pub fn handle_event(&self, event: AccessibilityEvent) -> Result<(), JsValue> {
        self.inner.handle_event(event)
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.inner.dispose()
    }
}
impl Inner {
    pub fn refresh(&self, start: Option<f64>, end: Option<f64>) -> Result<(), JsValue> {
        let timer = self
            .timer
            .borrow()
            .as_ref()
            .cloned()
            .ok_or_else(|| js_sys::Error::new("accessibility timer is not initialized"))?;
        timer.refresh(start, end, self.host.rows()? as f64)
    }
    fn character(&self, value: &[u16]) -> Result<(), JsValue> {
        let action = self
            .announcements
            .borrow_mut()
            .character(value)
            .map_err(js_error)?;
        if action == CharacterAction::Overflow {
            set_value(&self.live_region, &self.strings.too_much_output_value())?;
        }
        Ok(())
    }
    pub(super) fn announce(&self) -> Result<(), JsValue> {
        if self.announcements.borrow().pending().is_empty() {
            return Ok(());
        }
        let live = js_sys::Reflect::get(&self.live_region, &"textContent".into())?;
        if live == self.strings.too_much_output_value() {
            self.clear_live()?;
        }
        // Read the DOM getter before snapshotting pending data, matching +=.
        // Getters/setters may reenter, so no Rust borrow spans a host operation.
        let live = js_sys::Reflect::get(&self.live_region, &"textContent".into())?;
        let pending = self
            .announcements
            .borrow()
            .pending_snapshot()
            .map_err(js_error)?;
        let value = accessibility_string_add(&live, &utf16::to_js_string(&pending))?;
        set_value(&self.live_region, &value)?;
        self.announcements.borrow_mut().clear_pending();
        Ok(())
    }
    fn clear_live(&self) -> Result<(), JsValue> {
        set_text(&self.live_region, &[])?;
        self.announcements.borrow_mut().clear_live_region();
        Ok(())
    }
    fn handle_event(self: &Rc<Self>, event: AccessibilityEvent) -> Result<(), JsValue> {
        match event {
            AccessibilityEvent::Resize(rows) => self.resize_rows(rows),
            AccessibilityEvent::Render { start, end } => self.refresh(Some(start), Some(end)),
            AccessibilityEvent::Scroll => self.refresh(None, None),
            AccessibilityEvent::Character(value) => self.character(&value),
            AccessibilityEvent::LineFeed => self.character(&[10]),
            AccessibilityEvent::Tab(count) => {
                for _ in 0..count {
                    self.character(&[32])?;
                }
                Ok(())
            }
            AccessibilityEvent::Key(value) => {
                self.clear_live()?;
                self.announcements
                    .borrow_mut()
                    .key(&value)
                    .map_err(js_error)
            }
            AccessibilityEvent::Blur => self.clear_live(),
            AccessibilityEvent::Dimensions => self.refresh_dimensions(),
        }
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        if self.disposed.replace(true) {
            return Ok(());
        }
        let timer = self.timer.borrow().as_ref().cloned();
        if let Some(timer) = timer {
            timer.dispose()?;
        }
        let subscriptions = std::mem::take(&mut *self.subscriptions.borrow_mut());
        for subscription in subscriptions {
            subscription.dispose();
        }
        let selection_listener = self.selection_listener.borrow_mut().take();
        if let Some(mut listener) = selection_listener {
            listener.dispose()?;
        }
        self.detach_boundaries()?;
        self.container.remove();
        self.rows.borrow_mut().clear();
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
pub(super) fn element(document: &Document, class: &str) -> Result<HtmlElement, JsValue> {
    // The source assumes the known HTML tag's interface, not the module realm's
    // HTMLElement instanceof. A checked cast would reject valid iframe nodes.
    let element: HtmlElement = document.create_element("div")?.unchecked_into();
    element.set_class_name(class);
    Ok(element)
}
pub(super) fn js_error(error: impl std::fmt::Display) -> JsValue {
    js_sys::Error::new(&error.to_string()).into()
}
pub(super) fn set_text(element: &HtmlElement, text: &[u16]) -> Result<(), JsValue> {
    set_value(element, &utf16::to_js_string(text))
}
pub(super) fn set_value(element: &HtmlElement, value: &JsValue) -> Result<(), JsValue> {
    if !js_sys::Reflect::set(element, &"textContent".into(), value)? {
        return Err(js_sys::TypeError::new("Cannot assign textContent").into());
    }
    Ok(())
}
// This shim supplies only JavaScript's property-value + string coercion. All
// announcement state and transitions remain in Rust; thrown values stay intact.
#[wasm_bindgen(
    inline_js = "export function accessibility_string_add(left,right){return left+right;}"
)]
extern "C" {
    #[wasm_bindgen(catch)]
    fn accessibility_string_add(
        left: &JsValue,
        right: &js_sys::JsString,
    ) -> Result<JsValue, JsValue>;
}
