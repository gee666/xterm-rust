// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
#[path = "overview_ruler_draw.rs"]
mod draw;
use super::{
    color_zone_store::ColorZoneStore, service::RenderService, service_api::BrowserService,
};
use crate::theme::ThemeService;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement};
use xterm_core::{
    buffer::BufferService,
    decoration::DecorationService,
    lifecycle::{Disposable, DisposableStore},
    options::{OptionValue, OptionsService},
};
type Frame = Closure<dyn Fn(f64) -> Result<(), JsValue>>;
/// Live terminal services shared with the main renderer, not copied ruler state.
#[derive(Clone)]
pub struct OverviewServices {
    pub buffer: Rc<BufferService>,
    pub decorations: DecorationService,
    pub render: Rc<RenderService>,
    pub options: Rc<OptionsService>,
    pub theme: Rc<ThemeService>,
    pub browser: Rc<dyn BrowserService>,
}
/// Decoration overview canvas. Its owner must dispose it before browser services.
pub struct OverviewRulerRenderer {
    canvas: HtmlCanvasElement,
    context: RefCell<Option<CanvasRenderingContext2d>>,
    services: OverviewServices,
    zones: RefCell<ColorZoneStore>,
    frame: Cell<Option<i32>>,
    callback: RefCell<Option<Frame>>,
    update_dimensions: Cell<bool>,
    last_length: Cell<usize>,
    listeners: DisposableStore,
    disposed: Cell<bool>,
}
impl OverviewRulerRenderer {
    /// Insert the canvas before the viewport and subscribe to live terminal changes.
    pub fn new(
        viewport: &HtmlElement,
        document: &Document,
        services: OverviewServices,
    ) -> Result<Rc<Self>, JsValue> {
        // Known HTML tag and requested 2D interface are valid across iframe realms.
        let canvas: HtmlCanvasElement = document.create_element("canvas")?.unchecked_into();
        canvas.set_class_name("xterm-decoration-overview-ruler");
        let renderer = Rc::new(Self {
            canvas,
            context: RefCell::new(None),
            services,
            zones: RefCell::new(ColorZoneStore::new()),
            frame: Cell::new(None),
            callback: RefCell::new(None),
            update_dimensions: Cell::new(true),
            last_length: Cell::new(0),
            listeners: DisposableStore::new(),
            disposed: Cell::new(false),
        });
        renderer.refresh_canvas_dimensions()?;
        if let Some(parent) = viewport.parent_element() {
            parent.insert_before(renderer.canvas.as_ref(), Some(viewport.as_ref()))?;
        }
        let context = renderer
            .canvas
            .get_context("2d")?
            .ok_or_else(|| js_sys::Error::new("Ctx cannot be null"))?
            .unchecked_into();
        *renderer.context.borrow_mut() = Some(context);
        renderer.subscribe();
        renderer.queue_refresh(true)?;
        Ok(renderer)
    }
    /// Return the same canvas used for drawing and removed on disposal.
    pub fn canvas(&self) -> HtmlCanvasElement {
        self.canvas.clone()
    }
    fn ctx(&self) -> Result<CanvasRenderingContext2d, JsValue> {
        self.context
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::Error::new("Ctx cannot be null").into())
    }
    fn subscribe(self: &Rc<Self>) {
        for event in [
            self.services.decorations.on_decoration_registered(),
            self.services.decorations.on_decoration_removed(),
        ] {
            let weak = Rc::downgrade(self);
            self.listeners.add(event.try_subscribe(move |_| {
                if let Some(renderer) = weak.upgrade() {
                    renderer
                        .queue_refresh(false)
                        .map_err(crate::js_error::callback_error)?;
                }
                Ok(())
            }));
        }
        let weak = Rc::downgrade(self);
        self.listeners.add(
            self.services
                .render
                .on_rendered_viewport_change()
                .try_subscribe(move |_| {
                    if let Some(renderer) = weak.upgrade() {
                        renderer
                            .queue_refresh(false)
                            .map_err(crate::js_error::callback_error)?;
                    }
                    Ok(())
                }),
        );
        let weak = Rc::downgrade(self);
        self.listeners.add(
            self.services
                .buffer
                .buffers
                .on_buffer_activate()
                .try_subscribe(move |_| {
                    if let Some(renderer) = weak.upgrade() {
                        let active = renderer.services.buffer.buffer();
                        let alt = renderer.services.buffer.buffers.alt();
                        renderer
                            .canvas
                            .style()
                            .set_property(
                                "display",
                                if Rc::ptr_eq(&active, &alt) {
                                    "none"
                                } else {
                                    "block"
                                },
                            )
                            .map_err(crate::js_error::callback_error)?;
                    }
                    Ok(())
                }),
        );
        let weak = Rc::downgrade(self);
        self.listeners
            .add(self.services.buffer.on_scroll().try_subscribe(move |_| {
                if let Some(renderer) = weak.upgrade() {
                    if renderer.last_length.get()
                        != renderer
                            .normal_length()
                            .map_err(crate::js_error::callback_error)?
                    {
                        renderer
                            .refresh_heights()
                            .map_err(crate::js_error::callback_error)?;
                        renderer
                            .refresh_padding()
                            .map_err(crate::js_error::callback_error)?;
                    }
                }
                Ok(())
            }));
        let weak = Rc::downgrade(self);
        self.listeners.add(
            self.services
                .render
                .on_dimensions_change()
                .try_subscribe(move |_| {
                    if let Some(renderer) = weak.upgrade() {
                        renderer
                            .queue_refresh(true)
                            .map_err(crate::js_error::callback_error)?;
                    }
                    Ok(())
                }),
        );
        let weak = Rc::downgrade(self);
        self.listeners.add(
            self.services
                .browser
                .on_dpr_change()
                .try_subscribe(move |_| {
                    if let Some(renderer) = weak.upgrade() {
                        renderer
                            .queue_refresh(true)
                            .map_err(crate::js_error::callback_error)?;
                    }
                    Ok(())
                }),
        );
        let weak = Rc::downgrade(self);
        self.listeners.add(
            self.services
                .options
                .on_option_change()
                .try_subscribe(move |key| {
                    if *key == "scrollbar" {
                        if let Some(renderer) = weak.upgrade() {
                            renderer
                                .queue_refresh(true)
                                .map_err(crate::js_error::callback_error)?;
                        }
                    }
                    Ok(())
                }),
        );
        let weak = Rc::downgrade(self);
        self.listeners.add(
            self.services
                .theme
                .on_change_colors()
                .try_subscribe(move |_| {
                    if let Some(renderer) = weak.upgrade() {
                        renderer
                            .queue_refresh(false)
                            .map_err(crate::js_error::callback_error)?;
                    }
                    Ok(())
                }),
        );
    }
    fn scrollbar(&self) -> Result<OptionValue, JsValue> {
        self.services
            .options
            .get("scrollbar")
            .map_err(|e| crate::js_error::to_js_error(&e))
    }
    fn width(&self) -> Result<OptionValue, JsValue> {
        let scrollbar = self.scrollbar()?;
        let show = optional(&scrollbar, "showScrollbar")?;
        if !matches!(show, OptionValue::Undefined | OptionValue::Null) && !show.is_truthy() {
            return Ok(0.0.into());
        }
        let width = optional(&scrollbar, "width")?;
        Ok(
            if matches!(width, OptionValue::Undefined | OptionValue::Null) {
                0.0.into()
            } else {
                width
            },
        )
    }
    /// Coalesce redraws on the terminal owner window's next animation frame.
    pub fn queue_refresh(self: &Rc<Self>, dimensions: bool) -> Result<(), JsValue> {
        if self.disposed.get() {
            return Ok(());
        }
        self.update_dimensions
            .set(dimensions || self.update_dimensions.get());
        if self.frame.get().is_some() {
            return Ok(());
        }
        let weak = Rc::downgrade(self);
        let callback: Frame = Closure::wrap(Box::new(move |_: f64| {
            if let Some(renderer) = weak.upgrade() {
                if !renderer.disposed.get() {
                    renderer.refresh_decorations()?;
                }
                renderer.frame.set(None);
            }
            Ok(())
        }));
        let handle = self
            .services
            .browser
            .window()
            .request_animation_frame(callback.as_ref().unchecked_ref())?;
        *self.callback.borrow_mut() = Some(callback);
        self.frame.set(Some(handle));
        Ok(())
    }
    fn active_length(&self) -> Result<usize, JsValue> {
        let buffer = self.services.buffer.buffer();
        let length = buffer
            .try_borrow()
            .map_err(|e| js_sys::Error::new(&e.to_string()))?
            .lines
            .len();
        Ok(length)
    }
    fn normal_length(&self) -> Result<usize, JsValue> {
        let buffer = self.services.buffer.buffers.normal();
        let length = buffer
            .try_borrow()
            .map_err(|e| js_sys::Error::new(&e.to_string()))?
            .lines
            .len();
        Ok(length)
    }
    /// Remove the canvas, detach listeners and cancel any pending redraw once.
    pub fn dispose(&self) -> Result<(), JsValue> {
        if self.disposed.replace(true) {
            return Ok(());
        }
        self.canvas.remove();
        self.listeners.dispose();
        if let Some(handle) = self.frame.get() {
            self.services
                .browser
                .window()
                .cancel_animation_frame(handle)?;
            self.frame.set(None);
        }
        self.callback.borrow_mut().take();
        Ok(())
    }
}
impl Drop for OverviewRulerRenderer {
    fn drop(&mut self) {
        if let Err(error) = self.dispose() {
            web_sys::console::error_1(&error);
        }
    }
}
fn optional(value: &OptionValue, key: &str) -> Result<OptionValue, JsValue> {
    if matches!(value, OptionValue::Null | OptionValue::Undefined) {
        Ok(OptionValue::Undefined)
    } else {
        value
            .get_property(key)
            .map_err(|e| crate::js_error::to_js_error(&e))
    }
}
fn number(value: &OptionValue) -> Result<f64, JsValue> {
    value
        .to_number()
        .map_err(|e| crate::js_error::to_js_error(&e))
}
fn uint32(value: f64) -> u32 {
    if !value.is_finite() || value == 0.0 {
        0
    } else {
        value.trunc().rem_euclid(4294967296.0) as u32
    }
}
fn round(value: f64) -> f64 {
    let lower = value.floor();
    if value - lower < 0.5 {
        lower
    } else {
        value.ceil()
    }
}
