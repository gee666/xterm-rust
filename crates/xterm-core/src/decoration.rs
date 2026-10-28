// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
//! Host-independent decorations. Element handles are opaque to the terminal core.
use crate::{
    buffer::marker::Marker,
    color::{self, Color},
    event::{EmitError, Emitter, Event},
    lifecycle::{Disposable, DisposableHandle},
    sorted_list::{SortedList, SortedListError},
};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};
#[path = "decoration_cache.rs"]
mod cache;
pub use cache::{DecorationCellIterator, DecorationLineCache};
#[path = "decoration_service.rs"]
mod service;
pub use service::DecorationService;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecorationLayer {
    Bottom,
    Top,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverviewPosition {
    Full,
    Left,
    Center,
    Right,
}
#[derive(Clone, Debug, Default)]
pub struct OverviewRulerOptions {
    pub color: String,
    pub position: Option<OverviewPosition>,
}

/// Options remain shared and writable, like the upstream options object.
#[derive(Clone)]
pub struct DecorationOptions {
    pub marker: Marker,
    pub x: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<u32>,
    pub layer: Option<DecorationLayer>,
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
    pub overview_ruler_options: Option<OverviewRulerOptions>,
}
impl DecorationOptions {
    pub fn new(marker: Marker) -> Self {
        Self {
            marker,
            x: None,
            width: None,
            height: None,
            layer: None,
            background_color: None,
            foreground_color: None,
            overview_ruler_options: None,
        }
    }
}

#[derive(Debug)]
pub enum DecorationError {
    Dispatch(EmitError),
    Storage(SortedListError),
}
impl std::fmt::Display for DecorationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dispatch(e) => e.fmt(f),
            Self::Storage(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for DecorationError {}
impl From<EmitError> for DecorationError {
    fn from(e: EmitError) -> Self {
        Self::Dispatch(e)
    }
}
impl From<SortedListError> for DecorationError {
    fn from(e: SortedListError) -> Self {
        Self::Storage(e)
    }
}

type Element = Rc<dyn Any>;
struct DecorationState {
    marker: Marker,
    options: Rc<RefCell<DecorationOptions>>,
    indexed_start: Cell<i64>,
    disposed: Cell<bool>,
    element: RefCell<Option<Element>>,
    background: RefCell<Option<Option<Color>>>,
    foreground: RefCell<Option<Option<Color>>>,
    on_render: Emitter<Element>,
    on_dispose: Emitter<()>,
    marker_listener: RefCell<Option<DisposableHandle>>,
    service_listener: RefCell<Option<DisposableHandle>>,
}

impl Drop for DecorationState {
    fn drop(&mut self) {
        if let Some(listener) = self.marker_listener.get_mut().take() {
            listener.dispose();
        }
        if let Some(listener) = self.service_listener.get_mut().take() {
            listener.dispose();
        }
    }
}

/// Clones preserve decoration identity. Disposal remains explicit and fallible.
#[derive(Clone)]
pub struct Decoration {
    state: Rc<DecorationState>,
}
impl Decoration {
    fn new(options: Rc<RefCell<DecorationOptions>>) -> Self {
        let marker = options.borrow().marker.clone();
        if let Some(overview) = &mut options.borrow_mut().overview_ruler_options {
            if overview.position.is_none() {
                overview.position = Some(OverviewPosition::Full);
            }
        }
        Self {
            state: Rc::new(DecorationState {
                indexed_start: Cell::new(marker.line()),
                marker,
                options,
                disposed: Cell::new(false),
                element: RefCell::new(None),
                background: RefCell::new(None),
                foreground: RefCell::new(None),
                on_render: Emitter::new(),
                on_dispose: Emitter::new(),
                marker_listener: RefCell::new(None),
                service_listener: RefCell::new(None),
            }),
        }
    }
    pub fn marker(&self) -> Marker {
        self.state.marker.clone()
    }
    pub fn options(&self) -> Rc<RefCell<DecorationOptions>> {
        Rc::clone(&self.state.options)
    }
    pub fn is_disposed(&self) -> bool {
        self.state.disposed.get()
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }
    pub fn on_dispose(&self) -> Event<()> {
        self.state.on_dispose.event()
    }
    pub fn on_render(&self) -> Event<Element> {
        self.state.on_render.event()
    }
    pub fn element(&self) -> Option<Element> {
        self.state.element.borrow().clone()
    }
    /// Hosts set the element independently of firing onRender, as upstream renderers do.
    pub fn set_element(&self, element: Option<Element>) {
        *self.state.element.borrow_mut() = element;
    }
    pub fn fire_render(&self, element: Element) -> Result<(), EmitError> {
        self.state.on_render.fire(&element)
    }
    pub fn background_color(
        &self,
        canvas: impl FnOnce(&str) -> Option<[u8; 4]>,
    ) -> Result<Option<Color>, color::css::UnsupportedCss> {
        self.cached_color(&self.state.background, false, canvas)
    }
    pub fn foreground_color(
        &self,
        canvas: impl FnOnce(&str) -> Option<[u8; 4]>,
    ) -> Result<Option<Color>, color::css::UnsupportedCss> {
        self.cached_color(&self.state.foreground, true, canvas)
    }
    fn cached_color(
        &self,
        cache: &RefCell<Option<Option<Color>>>,
        foreground: bool,
        canvas: impl FnOnce(&str) -> Option<[u8; 4]>,
    ) -> Result<Option<Color>, color::css::UnsupportedCss> {
        if let Some(value) = cache.borrow().as_ref() {
            return Ok(value.clone());
        }
        let css = {
            let options = self.state.options.borrow();
            if foreground {
                options.foreground_color.clone()
            } else {
                options.background_color.clone()
            }
        };
        let parsed = match css.filter(|s| !s.is_empty()) {
            Some(css) => Some(color::css::to_color_with_canvas(&css, canvas)?),
            None => None,
        };
        *cache.borrow_mut() = Some(parsed.clone());
        Ok(parsed)
    }
    pub fn dispose(&self) -> Result<(), EmitError> {
        // The source fires before DisposableStore marks the decoration disposed.
        self.state.on_dispose.fire(&())?;
        if !self.state.disposed.replace(true) {
            self.state.on_render.dispose();
            self.state.on_dispose.dispose();
        }
        Ok(())
    }
    fn height(&self) -> i64 {
        i64::from(self.state.options.borrow().height.unwrap_or(1))
    }
    fn matches(&self, x: f64, layer: Option<DecorationLayer>) -> bool {
        let options = self.state.options.borrow();
        let xmin = options.x.unwrap_or(0.0);
        x >= xmin
            && x < xmin + options.width.unwrap_or(1.0)
            && layer.is_none_or(|layer| options.layer.unwrap_or(DecorationLayer::Bottom) == layer)
    }
}

fn line_key(d: &Decoration) -> f64 {
    d.marker().line() as f64
}
type DecorationList = SortedList<Decoration, fn(&Decoration) -> f64>;
