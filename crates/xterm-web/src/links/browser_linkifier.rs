// Copyright (c) 2019-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{
    Decorations, HoverState, Link, LinkProviderService, Position, ProviderRegistration,
    ProviderReplies, UnderlineEvent,
};
use crate::dom_listener::DomListener;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlElement, MouseEvent};

#[path = "browser_linkifier_events.rs"]
mod events;

pub type LinkCallback = Rc<dyn Fn(&MouseEvent, &Link) -> Result<(), JsValue>>;
pub type LinkDisposal = Rc<dyn Fn() -> Result<(), JsValue>>;
pub type LinkReply = Box<dyn Fn(Option<Vec<BrowserLink>>) -> Result<(), JsValue>>;
pub type MouseCoordinates = Rc<dyn Fn(&MouseEvent) -> Option<Position>>;
pub type UnderlineCallback = Rc<dyn Fn(bool, UnderlineEvent) -> Result<(), JsValue>>;

#[derive(Clone)]
pub struct BrowserLink {
    pub data: Rc<Link>,
    pub decorations: Option<Decorations>,
    pub activate: LinkCallback,
    pub hover: Option<LinkCallback>,
    pub leave: Option<LinkCallback>,
    pub dispose: Option<LinkDisposal>,
}
pub trait BrowserLinkProvider {
    fn provide_links(&self, y: usize, callback: LinkReply) -> Result<(), JsValue>;
}

pub(super) struct State {
    element: HtmlElement,
    providers: LinkProviderService<dyn BrowserLinkProvider>,
    replies: RefCell<ProviderReplies>,
    handlers: RefCell<HashMap<*const Link, Rc<BrowserLink>>>,
    hover: RefCell<HoverState>,
    coordinates: MouseCoordinates,
    underline: UnderlineCallback,
    last_event: RefCell<Option<MouseEvent>>,
    last_cell: Cell<Option<Position>>,
    active_line: Cell<Option<usize>>,
    mouse_out: Cell<bool>,
    resized: Cell<bool>,
    disposed: Cell<bool>,
    cols: Cell<usize>,
}

/// Live DOM link handling. The coordinate callback must read current renderer
/// dimensions and add ydisp to produce one-based buffer coordinates. Renderer
/// changes call `on_render`; terminal resize calls `resize`. Event listeners and
/// late provider replies cannot retain this controller after disposal/drop.
pub struct BrowserLinkifier {
    state: Rc<State>,
    listeners: Vec<DomListener>,
}
impl BrowserLinkifier {
    pub fn new(
        element: HtmlElement,
        cols: usize,
        scroll_offset: usize,
        coordinates: MouseCoordinates,
        underline: UnderlineCallback,
    ) -> Result<Self, JsValue> {
        let state = Rc::new(State {
            element: element.clone(),
            providers: LinkProviderService::default(),
            replies: RefCell::new(ProviderReplies::default()),
            handlers: RefCell::new(HashMap::new()),
            hover: RefCell::new(HoverState::new(cols, scroll_offset)),
            coordinates,
            underline,
            last_event: RefCell::new(None),
            last_cell: Cell::new(None),
            active_line: Cell::new(None),
            mouse_out: Cell::new(true),
            resized: Cell::new(false),
            disposed: Cell::new(false),
            cols: Cell::new(cols),
        });
        let mut listeners = Vec::new();
        for event_type in ["mousemove", "mouseleave", "mousedown", "mouseup"] {
            let weak = Rc::downgrade(&state);
            listeners.push(DomListener::new(
                element.as_ref(),
                event_type,
                false,
                move |event| {
                    let Some(state) = weak.upgrade().filter(|state| !state.disposed.get()) else {
                        return;
                    };
                    let result = match event_type {
                        "mousemove" => event
                            .dyn_into::<MouseEvent>()
                            .map_err(|_| JsValue::from_str("mousemove was not a MouseEvent"))
                            .and_then(|e| state.mouse_move(e)),
                        "mouseleave" => {
                            state.mouse_out.set(true);
                            state.clear_current()
                        }
                        "mousedown" => {
                            state.hover.borrow_mut().mouse_down();
                            Ok(())
                        }
                        "mouseup" => event
                            .dyn_into::<MouseEvent>()
                            .map_err(|_| JsValue::from_str("mouseup was not a MouseEvent"))
                            .and_then(|e| {
                                let effect = state.hover.borrow().mouse_up((state.coordinates)(&e));
                                state.apply(effect.into_iter().collect(), Some(&e))
                            }),
                        _ => unreachable!("fixed event type list"),
                    };
                    // DOM EventListener exceptions are reported, not swallowed. Native
                    // caller methods below instead propagate recoverable exceptions.
                    if let Err(error) = result {
                        web_sys::console::error_1(&error);
                    }
                },
            )?);
        }
        Ok(Self { state, listeners })
    }
    pub fn register_provider(
        &self,
        provider: Rc<dyn BrowserLinkProvider>,
    ) -> ProviderRegistration<dyn BrowserLinkProvider> {
        self.state.providers.register(provider)
    }
    pub fn current_link(&self) -> Option<Rc<Link>> {
        self.state.hover.borrow().current().cloned()
    }
    pub fn resize(&self, cols: usize, scroll_offset: usize) -> Result<(), JsValue> {
        self.state.cols.set(cols);
        self.state
            .hover
            .borrow_mut()
            .set_geometry(cols, scroll_offset);
        self.state.clear_current()?;
        self.state.resized.set(true);
        Ok(())
    }
    pub fn set_scroll_offset(&self, scroll_offset: usize) {
        self.state
            .hover
            .borrow_mut()
            .set_geometry(self.state.cols.get(), scroll_offset);
    }
    pub fn on_render(&self, start: usize, end: usize) -> Result<(), JsValue> {
        let invalidated = self.state.hover.borrow().invalidated_by_render(start, end);
        if invalidated {
            self.state.clear_current()?;
            let event = self.state.last_event.borrow().clone();
            if let Some(position) = event.as_ref().and_then(|e| (self.state.coordinates)(e)) {
                self.state.ask(position, false)?;
            }
        }
        Ok(())
    }
    pub fn set_current_underline(&self, value: bool) -> Result<(), JsValue> {
        let effect = self.state.hover.borrow_mut().set_underline(value);
        self.state.apply(effect.into_iter().collect(), None)
    }
    pub fn set_current_pointer_cursor(&self, value: bool) -> Result<(), JsValue> {
        let effect = self.state.hover.borrow_mut().set_pointer_cursor(value);
        self.state.apply(effect.into_iter().collect(), None)
    }
    pub fn dispose(&mut self) -> Result<(), JsValue> {
        self.state.disposed.set(true);
        for listener in &mut self.listeners {
            listener.dispose()?;
        }
        self.state.last_event.borrow_mut().take();
        self.state.replies.borrow_mut().clear();
        self.state.handlers.borrow_mut().clear();
        self.state.providers.dispose();
        Ok(())
    }
}
impl Drop for BrowserLinkifier {
    fn drop(&mut self) {
        if let Err(error) = self.dispose() {
            web_sys::console::error_1(&error);
        }
    }
}
use xterm_core::lifecycle::Disposable;
