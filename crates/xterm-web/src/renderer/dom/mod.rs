// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
mod backend;
mod blink;
mod draw;
mod layout;
use super::{
    canvas_width::CanvasWidthMeasurer, cursor_blink::CursorBlinkController, options::RenderOptions,
    row::RowFactory, row_types::Selection, service_api::RedrawRequest,
    text_blink::TextBlinkController, width_cache::WidthCache,
};
use crate::{
    api::RenderDimensions, char_size_browser::CharSizeService,
    character_joiner::CharacterJoinerService, dom::BrowserDom, theme::ThemeService,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, HtmlElement, Window};
use xterm_core::{
    decoration::DecorationService,
    event::Emitter,
    headless::Terminal,
    lifecycle::{Disposable, DisposableStore},
    options::OptionsService,
};

// A unique selector counter is required by upstream's per-terminal CSS isolation.
static NEXT_TERMINAL_ID: AtomicU64 = AtomicU64::new(1);

pub struct DomRenderer {
    terminal: Rc<Terminal>,
    document: Document,
    window: Window,
    element: HtmlElement,
    screen: HtmlElement,
    viewport: HtmlElement,
    pub row_container: HtmlElement,
    selection_container: HtmlElement,
    dimensions_style: HtmlElement,
    theme_style: HtmlElement,
    rows: RefCell<Vec<HtmlElement>>,
    row_blink: RefCell<Vec<bool>>,
    dimensions: Cell<RenderDimensions>,
    spacing: Cell<f64>,
    focused: Cell<bool>,
    selection: RefCell<Selection>,
    last_selection: Cell<Selection>,
    selection_model: Cell<super::selection_model::SelectionRenderModel>,
    theme: Rc<ThemeService>,
    joiners: Rc<CharacterJoinerService>,
    decoration: DecorationService,
    metrics: Rc<CharSizeService>,
    widths: RefCell<WidthCache<CanvasWidthMeasurer>>,
    options: RefCell<RenderOptions>,
    options_service: Rc<OptionsService>,
    listeners: DisposableStore,
    terminal_id: u64,
    disposed: Cell<bool>,
    redraw: Emitter<RedrawRequest>,
    text_blink: RefCell<Option<TextBlinkController>>,
    cursor_blink: RefCell<Option<CursorBlinkController>>,
    dom_listeners: RefCell<Vec<crate::dom_listener::DomListener>>,
}
impl DomRenderer {
    pub fn new(
        terminal: Rc<Terminal>,
        dom: &BrowserDom,
        theme: Rc<ThemeService>,
        joiners: Rc<CharacterJoinerService>,
    ) -> Result<Rc<Self>, JsValue> {
        let buffers = terminal
            .with_state(|state| Rc::clone(&state.buffers))
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let decoration = DecorationService::with_buffers(&buffers)
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        Self::new_with_decorations(terminal, dom, theme, joiners, decoration)
    }
    pub fn new_with_decorations(
        terminal: Rc<Terminal>,
        dom: &BrowserDom,
        theme: Rc<ThemeService>,
        joiners: Rc<CharacterJoinerService>,
        decoration: DecorationService,
    ) -> Result<Rc<Self>, JsValue> {
        Self::create(terminal, dom, theme, joiners, decoration, None)
    }
    pub fn new_with_metrics(
        terminal: Rc<Terminal>,
        dom: &BrowserDom,
        theme: Rc<ThemeService>,
        joiners: Rc<CharacterJoinerService>,
        decoration: DecorationService,
        metrics: Rc<CharSizeService>,
    ) -> Result<Rc<Self>, JsValue> {
        Self::create(terminal, dom, theme, joiners, decoration, Some(metrics))
    }
    fn create(
        terminal: Rc<Terminal>,
        dom: &BrowserDom,
        theme: Rc<ThemeService>,
        joiners: Rc<CharacterJoinerService>,
        decoration: DecorationService,
        metrics: Option<Rc<CharSizeService>>,
    ) -> Result<Rc<Self>, JsValue> {
        let options_service = terminal
            .with_state(|state| Rc::clone(&state.options))
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let options =
            RenderOptions::read(&options_service).map_err(|e| crate::js_error::to_js_error(&e))?;
        let metrics = match metrics {
            Some(metrics) => metrics,
            None => Rc::new(CharSizeService::new(
                &dom.document,
                &dom.helpers,
                options.font.clone(),
            )?),
        };
        metrics.measure()?;
        let mut widths = WidthCache::new(CanvasWidthMeasurer::new(&dom.document)?);
        widths.set_font(options.font.clone()).map_err(width_error)?;
        let row_container = element(&dom.document, "div", "xterm-rows")?;
        row_container
            .style()
            .set_property("line-height", "normal")?;
        row_container.set_attribute("aria-hidden", "true")?;
        let selection_container = element(&dom.document, "div", "xterm-selection")?;
        selection_container.set_attribute("aria-hidden", "true")?;
        let dimensions_style = element(&dom.document, "style", "")?;
        let theme_style = element(&dom.document, "style", "")?;
        let window = dom
            .document
            .default_view()
            .ok_or_else(|| js_sys::Error::new("Terminal document has no owner window"))?;
        let focused = dom
            .document
            .active_element()
            .is_some_and(|active| active.is_same_node(Some(dom.textarea.as_ref())));
        let renderer = Rc::new(Self {
            terminal,
            document: dom.document.clone(),
            window,
            element: dom.element.clone(),
            screen: dom.screen.clone(),
            viewport: dom.viewport.clone(),
            row_container,
            selection_container,
            dimensions_style,
            theme_style,
            rows: RefCell::new(Vec::new()),
            row_blink: RefCell::new(Vec::new()),
            dimensions: Cell::new(RenderDimensions::default()),
            spacing: Cell::new(0.0),
            focused: Cell::new(focused),
            selection: RefCell::new(Selection::default()),
            last_selection: Cell::new(Selection::default()),
            selection_model: Cell::new(super::selection_model::SelectionRenderModel::default()),
            theme,
            joiners,
            decoration,
            metrics,
            widths: RefCell::new(widths),
            options: RefCell::new(options),
            options_service,
            listeners: DisposableStore::new(),
            terminal_id: NEXT_TERMINAL_ID.fetch_add(1, Ordering::Relaxed),
            disposed: Cell::new(false),
            redraw: Emitter::new(),
            text_blink: RefCell::new(None),
            cursor_blink: RefCell::new(None),
            dom_listeners: RefCell::new(Vec::new()),
        });
        let cols = renderer
            .terminal
            .cols()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let rows = renderer
            .terminal
            .rows()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        renderer.refresh_row_elements(rows)?;
        renderer.screen.append_child(&renderer.dimensions_style)?;
        renderer.screen.append_child(&renderer.theme_style)?;
        renderer.update_dimensions(cols, rows)?;
        renderer.inject_css();
        renderer
            .element
            .class_list()
            .add_1(&renderer.owner_class())?;
        renderer.screen.append_child(&renderer.row_container)?;
        renderer
            .screen
            .append_child(&renderer.selection_container)?;
        renderer.set_default_spacing()?;
        renderer.initialize_blink()?;
        renderer.subscribe();
        Ok(renderer)
    }
    fn subscribe(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.listeners.add(
            self.options_service
                .on_option_change()
                .try_subscribe(move |_| {
                    if let Some(renderer) = weak.upgrade() {
                        renderer
                            .options_changed()
                            .map_err(crate::js_error::callback_error)?;
                    }
                    Ok(())
                }),
        );
        let weak = Rc::downgrade(self);
        self.listeners
            .add(self.theme.on_change_colors().subscribe(move |_| {
                if let Some(renderer) = weak.upgrade() {
                    renderer.inject_css();
                }
            }));
        let weak = Rc::downgrade(self);
        self.listeners
            .add(self.metrics.state.on_change().try_subscribe(move |_| {
                if let Some(renderer) = weak.upgrade() {
                    renderer
                        .handle_char_size_changed()
                        .map_err(crate::js_error::callback_error)?;
                }
                Ok(())
            }));
    }
    pub fn dimensions(&self) -> RenderDimensions {
        self.dimensions.get()
    }
    pub fn has_valid_char_size(&self) -> bool {
        self.metrics.state.has_valid_size()
    }
    pub fn decoration_service(&self) -> DecorationService {
        self.decoration.clone()
    }
    pub fn char_size_service(&self) -> Rc<CharSizeService> {
        Rc::clone(&self.metrics)
    }
    pub fn character_joiners(&self) -> Rc<CharacterJoinerService> {
        Rc::clone(&self.joiners)
    }
    fn owner_class(&self) -> String {
        format!("xterm-dom-renderer-owner-{}", self.terminal_id)
    }
    fn options_changed(&self) -> Result<(), JsValue> {
        let options = RenderOptions::read(&self.options_service)
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        self.widths
            .borrow_mut()
            .set_font(options.font.clone())
            .map_err(width_error)?;
        let font_changed = self.options.borrow().font != options.font;
        *self.options.borrow_mut() = options.clone();
        if font_changed {
            self.metrics.set_font(options.font)?;
        }
        let cols = self
            .terminal
            .cols()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let rows = self
            .terminal
            .rows()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        self.update_dimensions(cols, rows)?;
        self.inject_css();
        self.set_default_spacing()?;
        self.update_blink_duration()
    }
    fn inject_css(&self) {
        let colors = self.theme.colors();
        let colors = colors.borrow();
        let options = self.options.borrow();
        self.theme_style
            .set_text_content(Some(&super::theme_css::theme_css(
                self.terminal_id,
                &colors,
                &options.font,
                options.cursor_width,
            )));
    }
    pub fn clear(&self) -> Result<(), JsValue> {
        for row in self.rows.borrow().iter() {
            row.set_text_content(None);
        }
        self.row_blink.borrow_mut().fill(false);
        self.update_text_blink()
    }
    pub fn dispose(&self) {
        if self.disposed.replace(true) {
            return;
        }
        self.listeners.dispose();
        self.redraw.dispose();
        self.dom_listeners.borrow_mut().clear();
        if let Some(blink) = self.text_blink.borrow_mut().take() {
            blink.dispose();
        }
        if let Some(blink) = self.cursor_blink.borrow_mut().take() {
            blink.dispose();
        }
        self.widths.borrow_mut().dispose();
        self.row_container.remove();
        self.selection_container.remove();
        self.dimensions_style.remove();
        self.theme_style.remove();
        let owner = self.owner_class();
        let classes = self
            .element
            .class_name()
            .split_whitespace()
            .filter(|class| *class != owner)
            .collect::<Vec<_>>()
            .join(" ");
        self.element.set_class_name(&classes);
    }
}
impl Drop for DomRenderer {
    fn drop(&mut self) {
        self.dispose();
    }
}
fn element(document: &Document, tag: &str, class: &str) -> Result<HtmlElement, JsValue> {
    let element: HtmlElement = document.create_element(tag)?.dyn_into()?;
    if !class.is_empty() {
        element.set_class_name(class);
    }
    Ok(element)
}
fn width_error(error: super::width_cache::WidthError<JsValue>) -> JsValue {
    match error {
        super::width_cache::WidthError::Measure(error) => error,
        super::width_cache::WidthError::Disposed => {
            js_sys::TypeError::new("width measurement canvas has been disposed").into()
        }
    }
}
fn css_number(value: f64) -> String {
    ryu_js::Buffer::new().format(value).into()
}
