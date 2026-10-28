// Copyright (c) 2024 The xterm.js authors. All rights reserved. Licensed under MIT.
//! Viewport queue/synchronization logic. The browser host uses the actual
//! scrollable element and live services, never a separate terminal state.

use std::{cell::Cell, error::Error, fmt, rc::Rc};
use xterm_core::{
    event::{EmitError, Emitter, Event},
    lifecycle::Disposable,
};

#[derive(Debug)]
pub enum ViewportError<E> {
    Host(E),
    Dispatch(EmitError),
}
impl<E: fmt::Display> fmt::Display for ViewportError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host(error) => error.fmt(f),
            Self::Dispatch(error) => error.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for ViewportError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Host(error) => Some(error),
            Self::Dispatch(error) => Some(error),
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[path = "viewport_browser.rs"]
mod browser;
#[cfg(target_arch = "wasm32")]
pub use browser::{Viewport, ViewportServices};

pub type SyncCallback<E> = Rc<dyn Fn() -> Result<(), ViewportError<E>>>;
/// Reads and writes are separate to retain source getter/callback checkpoints.
/// The host must release its own data borrows before invoking a callback.
pub trait ViewportHost: 'static {
    type Error: 'static;
    fn display_row(&self) -> Result<f64, Self::Error>;
    fn cell_height(&self) -> Result<f64, Self::Error>;
    fn canvas_height(&self) -> Result<f64, Self::Error>;
    fn buffer_line_count(&self) -> Result<f64, Self::Error>;
    fn synchronized_output(&self) -> Result<bool, Self::Error>;
    fn scroll_top(&self) -> Result<f64, Self::Error>;
    fn set_dimensions(&self, height: f64, scroll_height: f64) -> Result<(), Self::Error>;
    fn set_scroll_top(&self, top: f64, reuse_animation: Option<bool>) -> Result<(), Self::Error>;
    fn add_refresh_callback(&self, callback: SyncCallback<Self::Error>)
        -> Result<i32, Self::Error>;
}

/// Shared controller. Its flags deliberately remain source partial state when
/// a host call or scroll listener fails; no rollback/finally behavior is added.
pub struct ViewportController<H: ViewportHost> {
    host: Rc<H>,
    queued: Cell<Option<i32>>,
    latest: Cell<Option<f64>>,
    syncing: Cell<bool>,
    handling: Cell<bool>,
    suppress: Cell<bool>,
    needs_render: Cell<bool>,
    request_scroll: Emitter<f64>,
}
impl<H: ViewportHost> ViewportController<H> {
    pub fn new(host: Rc<H>) -> Rc<Self> {
        Rc::new(Self {
            host,
            queued: Cell::new(None),
            latest: Cell::new(None),
            syncing: Cell::new(false),
            handling: Cell::new(false),
            suppress: Cell::new(false),
            needs_render: Cell::new(false),
            request_scroll: Emitter::new(),
        })
    }
    pub fn on_request_scroll_lines(&self) -> Event<f64> {
        self.request_scroll.event()
    }
    pub fn latest_display_row(&self) -> Option<f64> {
        self.latest.get()
    }
    pub fn queued_refresh(&self) -> Option<i32> {
        self.queued.get()
    }
    pub fn needs_sync_on_render(&self) -> bool {
        self.needs_render.get()
    }

    pub fn queue_sync(self: &Rc<Self>, row: Option<f64>) -> Result<(), ViewportError<H::Error>> {
        if let Some(row) = row {
            self.latest.set(Some(row));
        }
        if self.queued.get().is_some() {
            return Ok(());
        }
        let weak = Rc::downgrade(self);
        let id = self
            .host
            .add_refresh_callback(Rc::new(move || {
                if let Some(viewport) = weak.upgrade() {
                    viewport.queued.set(None);
                    viewport.sync(viewport.latest.get())?;
                }
                Ok(())
            }))
            .map_err(ViewportError::Host)?;
        // A reentrant scheduler can invoke the callback before returning its ID.
        // Source assignment happens afterward, retaining that ID in this case.
        self.queued.set(Some(id));
        Ok(())
    }
    pub fn buffer_activated(self: &Rc<Self>) -> Result<(), ViewportError<H::Error>> {
        self.latest.set(None);
        self.queue_sync(None)
    }
    pub fn sync(&self, row: Option<f64>) -> Result<(), ViewportError<H::Error>> {
        // Default arguments execute before the body/early-return in JS.
        let row = match row {
            Some(row) => row,
            None => self.host.display_row().map_err(ViewportError::Host)?,
        };
        if self.syncing.get() {
            return Ok(());
        }
        if self
            .host
            .synchronized_output()
            .map_err(ViewportError::Host)?
        {
            self.needs_render.set(true);
            return Ok(());
        }
        self.syncing.set(true);
        self.suppress.set(true);
        let height = self.host.canvas_height().map_err(ViewportError::Host)?;
        let cell = self.host.cell_height().map_err(ViewportError::Host)?;
        let count = self.host.buffer_line_count().map_err(ViewportError::Host)?;
        self.host
            .set_dimensions(height, cell * count)
            .map_err(ViewportError::Host)?;
        self.suppress.set(false);
        if !self.latest.get().is_some_and(|latest| latest == row) {
            let cell = self.host.cell_height().map_err(ViewportError::Host)?;
            self.host
                .set_scroll_top(row * cell, None)
                .map_err(ViewportError::Host)?;
        }
        self.syncing.set(false);
        Ok(())
    }
    pub fn on_render(&self) -> Result<(), ViewportError<H::Error>> {
        if self.needs_render.replace(false) {
            self.sync(None)?;
        }
        Ok(())
    }
    pub fn scroll_lines(&self, displacement: f64) -> Result<(), ViewportError<H::Error>> {
        let top = self.host.scroll_top().map_err(ViewportError::Host)?;
        let height = self.host.cell_height().map_err(ViewportError::Host)?;
        self.host
            .set_scroll_top(top + displacement * height, Some(true))
            .map_err(ViewportError::Host)
    }
    pub fn scroll_to_line(
        &self,
        line: f64,
        disable_smooth: bool,
    ) -> Result<(), ViewportError<H::Error>> {
        if disable_smooth {
            self.latest.set(Some(line));
        }
        let height = self.host.cell_height().map_err(ViewportError::Host)?;
        self.host
            .set_scroll_top(line * height, Some(!disable_smooth))
            .map_err(ViewportError::Host)
    }
    pub fn handle_touch_scroll(&self, translation: f64) -> Result<(), ViewportError<H::Error>> {
        let top = self.host.scroll_top().map_err(ViewportError::Host)?;
        self.host
            .set_scroll_top(top - translation, None)
            .map_err(ViewportError::Host)
    }
    pub fn handle_scroll(&self, top: f64) -> Result<(), ViewportError<H::Error>> {
        if self.handling.get() || self.suppress.get() {
            return Ok(());
        }
        self.handling.set(true);
        let height = self.host.cell_height().map_err(ViewportError::Host)?;
        let row = js_round(top / height);
        let current = self.host.display_row().map_err(ViewportError::Host)?;
        let difference = row - current;
        if difference != 0.0 {
            self.latest.set(Some(row));
            self.request_scroll
                .fire(&difference)
                .map_err(ViewportError::Dispatch)?;
        }
        self.handling.set(false);
        Ok(())
    }
    pub fn dispose(&self) {
        self.request_scroll.dispose();
    }
}

fn js_round(value: f64) -> f64 {
    if !value.is_finite() || value == 0.0 {
        return value;
    }
    let lower = value.floor();
    let rounded = if value - lower < 0.5 {
        lower
    } else {
        lower + 1.0
    };
    if rounded == 0.0 && value.is_sign_negative() {
        -0.0
    } else {
        rounded
    }
}
