// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Live buffer decorations. Geometry and lifecycle run in Rust against real DOM nodes.
use crate::{api::RenderDimensions, js_error};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, HtmlElement};
use xterm_core::{
    buffer::BufferSet,
    decoration::{Decoration, DecorationService},
    event::Event,
    headless::RenderEvent,
    lifecycle::{Disposable, DisposableStore},
};
#[path = "buffer_decoration_microtask.rs"]
mod microtask;
#[path = "buffer_decoration_style.rs"]
mod style;
pub use microtask::bind_microtasks;

pub type DecorationRefresh = Rc<dyn Fn(f64) -> Result<(), JsValue>>;
pub type AddDecorationRefresh = Rc<dyn Fn(DecorationRefresh) -> Result<i32, JsValue>>;
/// The same live service capabilities used by the upstream renderer. Tests can
/// supply controlled metrics/scheduling without substituting the DOM itself.
pub struct BufferDecorationHost {
    pub document: Document,
    pub screen: HtmlElement,
    pub buffers: Rc<BufferSet>,
    pub decorations: Rc<DecorationService>,
    pub dimensions: Rc<dyn Fn() -> Result<RenderDimensions, JsValue>>,
    pub add_refresh_callback: AddDecorationRefresh,
    pub viewport_changed: Event<RenderEvent>,
    pub dimensions_changed: Event<RenderDimensions>,
    pub dpr_changed: Event<f64>,
}
#[derive(Clone)]
struct Entry {
    decoration: Decoration,
    element: HtmlElement,
    handle: Rc<dyn std::any::Any>,
}
pub struct BufferDecorationRenderer {
    host: BufferDecorationHost,
    container: HtmlElement,
    entries: RefCell<Vec<Entry>>,
    listeners: DisposableStore,
    frame_pending: Cell<bool>,
    alt_active: Cell<bool>,
    dimensions_changed: Cell<bool>,
    disposed: Cell<bool>,
}
impl BufferDecorationRenderer {
    pub fn new(host: BufferDecorationHost) -> Result<Rc<Self>, JsValue> {
        // Upstream creates the container in global document, then adopts it into
        // the screen's document. Individual decoration nodes use mainDocument.
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| js_sys::Error::new("Decoration container requires document"))?;
        let container: HtmlElement = document.create_element("div")?.dyn_into()?;
        container.class_list().add_1("xterm-decoration-container")?;
        host.screen.append_child(&container)?;
        let renderer = Rc::new(Self {
            host,
            container,
            entries: RefCell::new(Vec::new()),
            listeners: DisposableStore::new(),
            frame_pending: Cell::new(false),
            alt_active: Cell::new(false),
            dimensions_changed: Cell::new(false),
            disposed: Cell::new(false),
        });
        renderer.subscribe();
        Ok(renderer)
    }
    fn subscribe(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.listeners
            .add(self.host.viewport_changed.try_subscribe(move |_| {
                if let Some(renderer) = weak.upgrade() {
                    renderer
                        .refresh_decorations()
                        .map_err(js_error::callback_error)?;
                }
                Ok(())
            }));
        let weak = Rc::downgrade(self);
        self.listeners
            .add(self.host.dimensions_changed.try_subscribe(move |_| {
                if let Some(renderer) = weak.upgrade() {
                    renderer.dimensions_changed.set(true);
                    renderer.queue_refresh().map_err(js_error::callback_error)?;
                }
                Ok(())
            }));
        let weak = Rc::downgrade(self);
        self.listeners
            .add(self.host.dpr_changed.try_subscribe(move |_| {
                if let Some(renderer) = weak.upgrade() {
                    renderer.queue_refresh().map_err(js_error::callback_error)?;
                }
                Ok(())
            }));
        let weak = Rc::downgrade(self);
        self.listeners.add(
            self.host
                .buffers
                .on_buffer_activate()
                .subscribe(move |event| {
                    if let Some(renderer) = weak.upgrade() {
                        renderer.alt_active.set(Rc::ptr_eq(
                            &event.active_buffer,
                            &renderer.host.buffers.alt(),
                        ));
                    }
                }),
        );
        let weak = Rc::downgrade(self);
        self.listeners.add(
            self.host
                .decorations
                .on_decoration_registered()
                .try_subscribe(move |_| {
                    if let Some(renderer) = weak.upgrade() {
                        renderer.queue_refresh().map_err(js_error::callback_error)?;
                    }
                    Ok(())
                }),
        );
        let weak = Rc::downgrade(self);
        self.listeners
            .add(
                self.host
                    .decorations
                    .on_decoration_removed()
                    .try_subscribe(move |decoration| {
                        if let Some(renderer) = weak.upgrade() {
                            renderer
                                .remove_decoration(decoration)
                                .map_err(js_error::callback_error)?;
                        }
                        Ok(())
                    }),
            );
    }
    fn queue_refresh(self: &Rc<Self>) -> Result<(), JsValue> {
        if self.frame_pending.get() {
            return Ok(());
        }
        let renderer = Rc::clone(self);
        (self.host.add_refresh_callback)(Rc::new(move |_| {
            renderer.refresh_decorations()?;
            // Source clears its sentinel after callbacks, not before them.
            renderer.frame_pending.set(false);
            Ok(())
        }))?;
        self.frame_pending.set(true);
        Ok(())
    }
    pub fn container(&self) -> HtmlElement {
        self.container.clone()
    }
    pub fn has_pending_refresh(&self) -> bool {
        self.frame_pending.get()
    }
    pub fn refresh_decorations(self: &Rc<Self>) -> Result<(), JsValue> {
        let decorations = self
            .host
            .decorations
            .decorations()
            .map_err(|e| js_error::to_js_error(&e))?;
        for decoration in decorations {
            self.refresh_style(&decoration)?;
            if self.dimensions_changed.get() {
                self.refresh_x(&decoration, None)?;
            }
        }
        self.dimensions_changed.set(false);
        Ok(())
    }
    fn remove_decoration(&self, decoration: &Decoration) -> Result<(), JsValue> {
        let removed = {
            let mut entries = self.entries.borrow_mut();
            entries
                .iter()
                .position(|entry| entry.decoration.ptr_eq(decoration))
                .map(|index| entries.remove(index))
        };
        if let Some(entry) = removed {
            entry.element.remove();
        }
        // Deliberate source reentry: onDecorationRemoved fires before the
        // decoration's outer disposal traversal has completed.
        decoration
            .dispose()
            .map_err(|error| js_error::to_js_error(&error))
    }
    pub fn dispose(&self) {
        if self.disposed.replace(true) {
            return;
        }
        self.listeners.dispose();
        self.container.remove();
        self.entries.borrow_mut().clear();
    }
}
impl Drop for BufferDecorationRenderer {
    fn drop(&mut self) {
        self.dispose();
    }
}
