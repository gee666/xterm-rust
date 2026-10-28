// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! DOM ownership for SelectionService. Core state remains in the Rust buffer.
#[path = "dom_events.rs"]
mod events;
#[path = "dom_schedule.rs"]
mod schedule;
#[path = "dom_subscriptions.rs"]
mod subscriptions;

use super::{
    model::{Dimensions, Position},
    pointer::{self, Modifiers, SelectionMouseOptions},
    service::{RedrawRequest, SelectionPlatform, SelectionService},
};
use crate::{dom::BrowserDom, dom_listener::DomListener, input::mouse_coords::MouseGeometry};
use std::{
    cell::{Cell, RefCell},
    num::NonZeroU32,
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Document, HtmlElement, MouseEvent, Window};
use xterm_core::{buffer::BufferService, event::Event, lifecycle::DisposableHandle};

type Geometry = dyn Fn() -> Result<MouseGeometry, JsValue>;
type FrameCallback = Closure<dyn FnMut(f64)>;
#[derive(Clone, Debug)]
pub struct SelectionDomOptions {
    pub platform: SelectionPlatform,
    pub mouse: SelectionMouseOptions,
    pub word_separator: Vec<u16>,
    pub alt_click_moves_cursor: bool,
    pub application_cursor_keys: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct LinkSelection {
    pub start: Position,
    pub length: i64,
}
/// The coordinator implements this with live terminal/linkifier/render services.
/// It must not strongly own SelectionDom, or that would form an ownership cycle.
pub trait SelectionHost {
    fn options(&self) -> Result<SelectionDomOptions, JsValue>;
    fn current_link(&self) -> Result<Option<LinkSelection>, JsValue>;
    fn on_user_input(&self) -> Result<Event<()>, JsValue>;
    fn scroll_lines(&self, amount: i64, suppress_scroll_event: bool) -> Result<(), JsValue>;
    fn data(&self, data: &[u16], user_input: bool) -> Result<(), JsValue>;
    fn move_cursor(
        &self,
        viewport_position: Position,
        application_cursor_keys: bool,
    ) -> Result<(), JsValue>;
    fn redraw(&self, event: RedrawRequest) -> Result<(), JsValue>;
    fn selection_changed(&self) -> Result<(), JsValue>;
    fn linux_mouse_selection(&self, text: &[u16]) -> Result<(), JsValue>;
    fn report_error(&self, error: JsValue);
}
struct State {
    element: HtmlElement,
    screen: HtmlElement,
    document: Document,
    window: Window,
    buffers: Rc<BufferService>,
    selection: Rc<SelectionService>,
    geometry: Rc<Geometry>,
    host: Rc<dyn SelectionHost>,
    subscriptions: RefCell<Vec<DisposableHandle>>,
    trim_subscription: RefCell<Option<DisposableHandle>>,
    listeners: RefCell<Vec<DomListener>>,
    drag_listeners: RefCell<Vec<DomListener>>,
    raf_callback: RefCell<Option<FrameCallback>>,
    raf_id: Cell<Option<i32>>,
    drag_callback: RefCell<Option<Closure<dyn FnMut()>>>,
    drag_id: Cell<Option<i32>>,
    drag_amount: Cell<f64>,
    down_timestamp: Cell<f64>,
    disposed: Cell<bool>,
}
pub struct SelectionDom {
    state: Rc<State>,
}
impl SelectionDom {
    pub fn new(
        dom: &BrowserDom,
        buffers: Rc<BufferService>,
        selection: Rc<SelectionService>,
        geometry: Rc<Geometry>,
        host: Rc<dyn SelectionHost>,
    ) -> Result<Self, JsValue> {
        let window = dom
            .document
            .default_view()
            .ok_or_else(|| js_sys::Error::new("Selection requires a document window"))?;
        let component = Self {
            state: Rc::new(State {
                element: dom.element.clone(),
                screen: dom.screen.clone(),
                document: dom.document.clone(),
                window,
                buffers,
                selection,
                geometry,
                host,
                subscriptions: RefCell::new(Vec::new()),
                trim_subscription: RefCell::new(None),
                listeners: RefCell::new(Vec::new()),
                drag_listeners: RefCell::new(Vec::new()),
                raf_callback: RefCell::new(None),
                raf_id: Cell::new(None),
                drag_callback: RefCell::new(None),
                drag_id: Cell::new(None),
                drag_amount: Cell::new(0.0),
                down_timestamp: Cell::new(0.0),
                disposed: Cell::new(false),
            }),
        };
        subscriptions::bind(&component.state)?;
        events::bind(&component.state)?;
        Ok(component)
    }
    pub fn service(&self) -> Rc<SelectionService> {
        Rc::clone(&self.state.selection)
    }
    pub fn right_click_select(&self, event: &MouseEvent) -> Result<(), JsValue> {
        events::right_click(&self.state, event)
    }
    pub fn should_force_selection(&self, event: &MouseEvent) -> Result<bool, JsValue> {
        let options = self.state.host.options()?;
        Ok(pointer::should_force_selection(
            options.platform,
            options.mouse,
            modifiers(event),
        ))
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.state.dispose()
    }
}
impl Drop for SelectionDom {
    fn drop(&mut self) {
        if let Err(error) = self.state.dispose() {
            self.state.host.report_error(error);
        }
    }
}
impl State {
    fn dimensions(&self) -> Result<Dimensions, JsValue> {
        let active = self.buffers.buffer();
        let scrollback = active.try_borrow().map_err(error)?.ybase;
        self.coordinate_dimensions(u32::try_from(scrollback).map_err(error)?)
    }
    fn coordinate_dimensions(&self, scrollback: u32) -> Result<Dimensions, JsValue> {
        let columns = u32::try_from(self.buffers.cols()).map_err(error)?;
        let rows = u32::try_from(self.buffers.rows()).map_err(error)?;
        Ok(Dimensions {
            columns: NonZeroU32::new(columns).ok_or_else(|| error("zero selection columns"))?,
            rows: NonZeroU32::new(rows).ok_or_else(|| error("zero selection rows"))?,
            scrollback,
        })
    }
    fn buffer_coords(
        &self,
        event: &MouseEvent,
        selection: bool,
        root: bool,
    ) -> Result<Option<Position>, JsValue> {
        let mut geometry = (self.geometry)()?;
        let element = if root { &self.element } else { &self.screen };
        let rect = element.get_bounding_client_rect();
        geometry.rect_left = rect.left();
        geometry.rect_top = rect.top();
        let Some([column, row]) = geometry.coords(
            f64::from(event.client_x()),
            f64::from(event.client_y()),
            f64::from(u32::try_from(self.buffers.cols()).map_err(error)?),
            f64::from(u32::try_from(self.buffers.rows()).map_err(error)?),
            selection,
        ) else {
            return Ok(None);
        };
        let active = self.buffers.buffer();
        let display = if root {
            0
        } else {
            active.try_borrow().map_err(error)?.ydisp
        };
        let column = exact_u32(column - 1.0)?;
        let row = i64::from(exact_u32(row - 1.0)?)
            .checked_add(i64::try_from(display).map_err(error)?)
            .ok_or_else(|| error("selection row overflow"))?;
        Ok(Some(Position::new(i64::from(column), row)))
    }
    fn stop_drag(&self) -> Result<(), JsValue> {
        if let Some(timer) = self.drag_id.take() {
            self.window.clear_interval_with_handle(timer);
        }
        let listeners = std::mem::take(&mut *self.drag_listeners.borrow_mut());
        for mut listener in listeners {
            listener.dispose()?;
        }
        // Keep the interval closure until a later turn/disposal; mouseup may be
        // raised synchronously from a scroll callback while it is executing.
        Ok(())
    }
    fn dispose(&self) -> Result<(), JsValue> {
        if self.disposed.replace(true) {
            return Ok(());
        }
        self.stop_drag()?;
        if let Some(frame) = self.raf_id.take() {
            self.window.cancel_animation_frame(frame)?;
        }
        for mut listener in std::mem::take(&mut *self.listeners.borrow_mut()) {
            listener.dispose()?;
        }
        for subscription in std::mem::take(&mut *self.subscriptions.borrow_mut()) {
            subscription.dispose();
        }
        if let Some(subscription) = self.trim_subscription.borrow_mut().take() {
            subscription.dispose();
        }
        self.selection.dispose().map_err(error)?;
        Ok(())
    }
}
fn modifiers(event: &MouseEvent) -> Modifiers {
    Modifiers {
        alt: event.alt_key(),
        shift: event.shift_key(),
    }
}
fn exact_u32(value: f64) -> Result<u32, JsValue> {
    if !value.is_finite() || value.fract() != 0.0 || value < 0.0 || value > f64::from(u32::MAX) {
        return Err(error("invalid selection coordinates"));
    }
    Ok(value as u32)
}
fn error(value: impl std::fmt::Display) -> JsValue {
    js_sys::Error::new(&value.to_string()).into()
}
fn selection_error(error: super::service::SelectionError) -> JsValue {
    crate::js_error::to_js_error(&error)
}
