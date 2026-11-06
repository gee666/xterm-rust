// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
//! Host-independent decorations. Element handles are opaque to the terminal core.
use crate::{
    buffer::{marker::Marker, marker_line::MarkerLine},
    color::{self, Color},
    event::{EmitError, Emitter, Event},
    lifecycle::{Disposable, DisposableHandle},
    sorted_list::{KeyReader, SortedKey, SortedList, SortedListError},
};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};
#[path = "decoration_cache.rs"]
mod cache;
#[path = "decoration_span.rs"]
mod span;
pub use cache::{
    DecorationBucket, DecorationBucketIterator, DecorationCellIterator, DecorationLineCache,
};
pub use span::NonAdvancingDecorationSpan;
#[path = "decoration_properties.rs"]
mod properties;
pub use properties::nullish;
#[path = "decoration_service.rs"]
mod service;
pub use service::{DecorationInitializer, DecorationService};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecorationLayer {
    Bottom,
    Top,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecorationAnchor {
    Left,
    Right,
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
    pub anchor: Option<DecorationAnchor>,
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
            anchor: None,
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
    NonAdvancingSpan(NonAdvancingDecorationSpan),
    Host(crate::options::OptionError),
    Line(crate::buffer::marker_line::MarkerLineError),
    Color(color::css::UnsupportedCss),
}
impl std::fmt::Display for DecorationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dispatch(e) => e.fmt(f),
            Self::Storage(e) => e.fmt(f),
            Self::NonAdvancingSpan(e) => e.fmt(f),
            Self::Host(e) => e.fmt(f),
            Self::Line(e) => e.fmt(f),
            Self::Color(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for DecorationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dispatch(error) => Some(error),
            Self::Storage(error) => Some(error),
            Self::NonAdvancingSpan(error) => Some(error),
            Self::Host(error) => Some(error),
            Self::Line(error) => Some(error),
            Self::Color(error) => Some(error),
        }
    }
}
impl From<NonAdvancingDecorationSpan> for DecorationError {
    fn from(error: NonAdvancingDecorationSpan) -> Self {
        Self::NonAdvancingSpan(error)
    }
}
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

impl From<crate::options::OptionError> for DecorationError {
    fn from(error: crate::options::OptionError) -> Self {
        Self::Host(error)
    }
}
impl From<crate::buffer::marker_line::MarkerLineError> for DecorationError {
    fn from(error: crate::buffer::marker_line::MarkerLineError) -> Self {
        Self::Line(error)
    }
}
impl From<color::css::UnsupportedCss> for DecorationError {
    fn from(error: color::css::UnsupportedCss) -> Self {
        Self::Color(error)
    }
}

type Element = Rc<dyn Any>;
/// Host ownership cleanup, run only after successful native disposal.
pub type DecorationHostCleanup = Rc<dyn Fn() -> Result<(), EmitError>>;
/// Cached IColor object identity is retained across repeated getter calls.
pub type DecorationColor = Rc<RefCell<Color>>;
struct DecorationState {
    marker: Marker,
    options: Rc<RefCell<DecorationOptions>>,
    source: Option<crate::options::OptionValue>,
    indexed_start: RefCell<MarkerLine>,
    disposed: Cell<bool>,
    element: RefCell<Option<Element>>,
    background: RefCell<Option<Option<DecorationColor>>>,
    foreground: RefCell<Option<Option<DecorationColor>>>,
    on_render: Emitter<Element>,
    on_dispose: Emitter<()>,
    marker_listener: RefCell<Option<DisposableHandle>>,
    service_listener: RefCell<Option<DisposableHandle>>,
    host_cleanup: RefCell<Option<DecorationHostCleanup>>,
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
    fn new(
        options: Rc<RefCell<DecorationOptions>>,
        source: Option<crate::options::OptionValue>,
    ) -> Self {
        let marker = options.borrow().marker.clone();
        if let Some(overview) = &mut options.borrow_mut().overview_ruler_options {
            if overview.position.is_none() {
                overview.position = Some(OverviewPosition::Full);
            }
        }
        Self {
            state: Rc::new(DecorationState {
                indexed_start: RefCell::new(marker.line_value()),
                marker,
                options,
                source,
                disposed: Cell::new(false),
                element: RefCell::new(None),
                background: RefCell::new(None),
                foreground: RefCell::new(None),
                on_render: Emitter::new(),
                on_dispose: Emitter::new(),
                marker_listener: RefCell::new(None),
                service_listener: RefCell::new(None),
                host_cleanup: RefCell::new(None),
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
    ) -> Result<Option<DecorationColor>, DecorationError> {
        self.cached_color(&self.state.background, false, canvas)
    }
    pub fn foreground_color(
        &self,
        canvas: impl FnOnce(&str) -> Option<[u8; 4]>,
    ) -> Result<Option<DecorationColor>, DecorationError> {
        self.cached_color(&self.state.foreground, true, canvas)
    }
    fn cached_color(
        &self,
        cache: &RefCell<Option<Option<DecorationColor>>>,
        foreground: bool,
        canvas: impl FnOnce(&str) -> Option<[u8; 4]>,
    ) -> Result<Option<DecorationColor>, DecorationError> {
        if let Some(value) = cache.borrow().as_ref() {
            return Ok(value.clone());
        }
        let key = if foreground {
            "foregroundColor"
        } else {
            "backgroundColor"
        };
        let css = self.property(key)?;
        let css = if css.is_truthy() {
            let value = self.property(key)?.to_utf16()?;
            Some(String::from_utf16(&value).map_err(|_| color::css::UnsupportedCss)?)
        } else {
            None
        };
        let parsed = match css {
            Some(css) => Some(Rc::new(RefCell::new(color::css::to_color_with_canvas(
                &css, canvas,
            )?))),
            None => None,
        };
        *cache.borrow_mut() = Some(parsed.clone());
        Ok(parsed)
    }
    /// Release a host's listener/ownership table only after a successful source
    /// disposal. The callback must not strongly own this decoration.
    pub fn set_host_cleanup(&self, cleanup: DecorationHostCleanup) -> Result<(), EmitError> {
        if self.is_disposed() {
            return cleanup();
        }
        *self.state.host_cleanup.borrow_mut() = Some(cleanup);
        Ok(())
    }
    pub fn dispose(&self) -> Result<(), EmitError> {
        // The source fires before DisposableStore marks the decoration disposed.
        self.state.on_dispose.fire(&())?;
        if !self.state.disposed.replace(true) {
            self.state.on_render.dispose();
            self.state.on_dispose.dispose();
            let cleanup = self.state.host_cleanup.borrow_mut().take();
            if let Some(cleanup) = cleanup {
                cleanup()?;
            }
        }
        Ok(())
    }
    fn indexed_start(&self) -> MarkerLine {
        self.state.indexed_start.borrow().clone()
    }
    fn set_indexed_start(&self, start: MarkerLine) {
        *self.state.indexed_start.borrow_mut() = start;
    }
}

struct DecorationLineKey;
impl KeyReader<Decoration> for DecorationLineKey {
    fn read_key(&self, decoration: Option<&Decoration>) -> Result<SortedKey, SortedListError> {
        Ok(decoration.map_or(SortedKey::Undefined, |decoration| {
            SortedKey::from(decoration.marker().line_value())
        }))
    }
}
type DecorationList = SortedList<Decoration, DecorationLineKey>;
