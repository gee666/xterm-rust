// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
#[path = "service_init.rs"]
mod initialize;
#[path = "service_lifecycle.rs"]
mod lifecycle;
#[path = "service_observer.rs"]
mod observer;
#[path = "service_sync.rs"]
mod synchronized;
use super::{
    row_types::Selection,
    service_api::{BrowserService, Renderer},
};
use crate::{
    api::RenderDimensions,
    char_size_browser::CharSizeService,
    links::{browser_debounce::BrowserRenderDebouncer, debounce::RenderRange},
    task_queue::BrowserTaskQueue,
    theme::ThemeService,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use synchronized::SynchronizedOutput;
use wasm_bindgen::{closure::Closure, JsValue};
use web_sys::IntersectionObserver;
use xterm_core::{
    buffer::BufferService,
    core_service::CoreService,
    decoration::DecorationService,
    event::{Emitter, Event},
    headless::Terminal,
    lifecycle::{Disposable, DisposableStore},
    options::OptionsService,
};
type ObserverCallback = Closure<dyn Fn(js_sys::Array) -> Result<(), JsValue>>;

/// Source RenderService orchestration. Backend replacement, clocks and observer
/// callbacks share this state; no RefCell guard crosses renderer/user callbacks.
pub struct RenderService {
    browser: Rc<dyn BrowserService>,
    core: Rc<CoreService>,
    buffer: Rc<BufferService>,
    options: Rc<OptionsService>,
    metrics: Rc<CharSizeService>,
    renderer: RefCell<Option<Rc<dyn Renderer>>>,
    rows: Cell<usize>,
    paused: Cell<bool>,
    needs_full: Cell<bool>,
    redraw_only: Cell<bool>,
    selection_dirty: Cell<bool>,
    selection: Cell<Selection>,
    debouncer: RefCell<Option<Rc<BrowserRenderDebouncer>>>,
    paused_resize: BrowserTaskQueue,
    synchronized: RefCell<Option<SynchronizedOutput>>,
    observer: RefCell<Option<IntersectionObserver>>,
    observer_callback: RefCell<Option<ObserverCallback>>,
    listeners: DisposableStore,
    disposed: Cell<bool>,
    dimensions_change: Emitter<RenderDimensions>,
    viewport_change: Emitter<RenderRange>,
    render: Emitter<RenderRange>,
    refresh_request: Emitter<RenderRange>,
}
impl RenderService {
    pub fn on_dimensions_change(&self) -> Event<RenderDimensions> {
        self.dimensions_change.event()
    }
    pub fn on_rendered_viewport_change(&self) -> Event<RenderRange> {
        self.viewport_change.event()
    }
    pub fn on_render(&self) -> Event<RenderRange> {
        self.render.event()
    }
    /// The supplied source declares this event but never fires it.
    pub fn on_refresh_request(&self) -> Event<RenderRange> {
        self.refresh_request.event()
    }
    pub fn current_renderer(&self) -> Option<Rc<dyn Renderer>> {
        if self.disposed.get() {
            None
        } else {
            self.renderer.borrow().clone()
        }
    }
    pub fn has_renderer(&self) -> bool {
        self.current_renderer().is_some()
    }
    pub fn dimensions(&self) -> Result<RenderDimensions, JsValue> {
        Ok(self
            .current_renderer()
            .ok_or_else(no_renderer)?
            .dimensions())
    }
    pub fn has_valid_char_size(&self) -> bool {
        self.metrics.state.has_valid_size()
    }
    pub fn set_renderer(self: &Rc<Self>, renderer: Rc<dyn Renderer>) -> Result<(), JsValue> {
        if !self.disposed.get() {
            let old = self.current_renderer();
            if !old.as_ref().is_some_and(|old| Rc::ptr_eq(old, &renderer)) {
                if let Some(old) = old {
                    old.dispose()?;
                }
                *self.renderer.borrow_mut() = Some(renderer);
            }
            // MutableDisposable skips equal assignment, but source still wires
            // another redraw listener and requests a full selection refresh.
            if let Some(renderer) = self.current_renderer() {
                let weak = Rc::downgrade(self);
                // The renderer owns this event's lifetime, as in source. Dropping
                // a native subscription handle does not unregister its callback.
                let _subscription = renderer.on_request_redraw().try_subscribe(move |event| {
                    if let Some(service) = weak.upgrade() {
                        service
                            .refresh_rows(event.start, event.end, event.sync, true)
                            .map_err(crate::js_error::callback_error)?;
                    }
                    Ok(())
                });
                self.selection_dirty.set(true);
                self.full_refresh()?;
            }
        }
        Ok(())
    }
    pub fn refresh_rows(
        &self,
        mut start: f64,
        mut end: f64,
        sync: bool,
        redraw_only: bool,
    ) -> Result<(), JsValue> {
        if self.paused.get() {
            self.needs_full.set(true);
            return Ok(());
        }
        let synchronized = self.synchronizer()?;
        if self.core.dec_private_modes.borrow().synchronized_output {
            return synchronized.buffer_rows(start, end);
        }
        if let Some(buffered) = synchronized.flush() {
            start = js_min(start, buffered.start);
            end = js_max(end, buffered.end);
        }
        if !redraw_only {
            self.redraw_only.set(false);
        }
        if sync {
            self.render_rows(start, end)
        } else {
            self.debounce()?
                .refresh(Some(start), Some(end), self.rows.get() as f64)
        }
    }
    fn render_rows(&self, mut start: f64, mut end: f64) -> Result<(), JsValue> {
        let Some(renderer) = self.current_renderer() else {
            return Ok(());
        };
        if self.core.dec_private_modes.borrow().synchronized_output {
            return self.synchronizer()?.buffer_rows(start, end);
        }
        let last = self.rows.get() as f64 - 1.0;
        start = js_min(start, last);
        end = js_min(end, last);
        renderer.render_rows(start, end)?;
        if self.selection_dirty.get() {
            let selection = self.selection.get();
            self.current_renderer()
                .ok_or_else(no_renderer)?
                .handle_selection_changed(selection.start, selection.end, selection.column)?;
            self.selection_dirty.set(false);
        }
        let event = RenderRange { start, end };
        if !self.redraw_only.get() {
            self.viewport_change
                .fire(&event)
                .map_err(|e| crate::js_error::to_js_error(&e))?;
        }
        self.render
            .fire(&event)
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        self.redraw_only.set(true);
        Ok(())
    }
    pub fn full_refresh(&self) -> Result<(), JsValue> {
        if self.paused.get() {
            self.needs_full.set(true);
            Ok(())
        } else {
            self.refresh_rows(0.0, self.rows.get() as f64 - 1.0, false, false)
        }
    }
    pub fn resize(&self, _cols: usize, rows: usize) -> Result<(), JsValue> {
        self.rows.set(rows);
        self.fire_canvas_resize()
    }
    fn options_changed(&self) -> Result<(), JsValue> {
        if self.has_renderer() {
            self.full_refresh()?;
            self.fire_canvas_resize()?;
        }
        Ok(())
    }
    fn fire_canvas_resize(&self) -> Result<(), JsValue> {
        let Some(renderer) = self.current_renderer() else {
            return Ok(());
        };
        let dimensions = renderer.dimensions();
        // Source initializes its canvas cache to zero and never updates it.
        // Repeated nonzero dimensions therefore still emit; do not fix that quirk.
        if dimensions.css.canvas.width == 0.0 && dimensions.css.canvas.height == 0.0 {
            return Ok(());
        }
        self.dimensions_change
            .fire(&dimensions)
            .map_err(|e| crate::js_error::to_js_error(&e))
    }
    pub fn add_refresh_callback(
        &self,
        callback: impl Fn(f64) -> Result<(), JsValue> + 'static,
    ) -> Result<i32, JsValue> {
        self.debounce()?.add_try_refresh_callback(callback)
    }
    pub fn clear_texture_atlas(&self) -> Result<(), JsValue> {
        if let Some(renderer) = self.current_renderer() {
            if let Some(atlas) = renderer.texture_atlas() {
                atlas.clear_texture_atlas()?;
            }
            self.full_refresh()?;
        }
        Ok(())
    }
    pub fn handle_device_pixel_ratio_change(&self) -> Result<(), JsValue> {
        self.metrics.measure()?;
        if let Some(renderer) = self.current_renderer() {
            renderer.handle_device_pixel_ratio_change()?;
            self.full_refresh()?;
        }
        Ok(())
    }
    pub fn handle_resize(self: &Rc<Self>, cols: usize, rows: usize) -> Result<(), JsValue> {
        let Some(renderer) = self.current_renderer() else {
            return Ok(());
        };
        if self.paused.get() {
            self.paused_resize.clear();
            let weak = Rc::downgrade(self);
            self.paused_resize.enqueue_try(move |_| {
                if let Some(service) = weak.upgrade() {
                    if let Some(renderer) = service.current_renderer() {
                        renderer.handle_resize(cols, rows)?;
                    }
                }
                Ok(false)
            })?;
        } else {
            renderer.handle_resize(cols, rows)?;
        }
        self.full_refresh()
    }
    pub fn handle_char_size_changed(&self) -> Result<(), JsValue> {
        if let Some(renderer) = self.current_renderer() {
            renderer.handle_char_size_changed()?;
        }
        Ok(())
    }
    pub fn handle_blur(&self) -> Result<(), JsValue> {
        if let Some(renderer) = self.current_renderer() {
            renderer.handle_blur()?;
        }
        Ok(())
    }
    pub fn handle_focus(&self) -> Result<(), JsValue> {
        if let Some(renderer) = self.current_renderer() {
            renderer.handle_focus()?;
        }
        Ok(())
    }
    pub fn handle_selection_changed(
        &self,
        start: Option<[i64; 2]>,
        end: Option<[i64; 2]>,
        column: bool,
    ) -> Result<(), JsValue> {
        self.selection.set(Selection { start, end, column });
        if let Some(renderer) = self.current_renderer() {
            renderer.handle_selection_changed(start, end, column)?;
        }
        Ok(())
    }
    pub fn handle_cursor_move(&self) -> Result<(), JsValue> {
        if let Some(renderer) = self.current_renderer() {
            renderer.handle_cursor_move()?;
        }
        Ok(())
    }
    pub fn clear(&self) -> Result<(), JsValue> {
        if let Some(renderer) = self.current_renderer() {
            renderer.clear()?;
        }
        Ok(())
    }
    fn debounce(&self) -> Result<Rc<BrowserRenderDebouncer>, JsValue> {
        self.debouncer
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::Error::new("render debouncer is unavailable").into())
    }
    fn synchronizer(&self) -> Result<SynchronizedOutput, JsValue> {
        self.synchronized
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::Error::new("synchronized output handler is unavailable").into())
    }
}
fn no_renderer() -> JsValue {
    js_sys::TypeError::new("render service has no active renderer").into()
}
fn js_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 && (a.is_sign_negative() || b.is_sign_negative()) {
        -0.0
    } else {
        a.min(b)
    }
}
fn js_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 && (a.is_sign_positive() || b.is_sign_positive()) {
        0.0
    } else {
        a.max(b)
    }
}
