// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
mod backend;
mod blink;
mod browser;
mod create;
mod draw;
mod layout;
mod styles;
use super::{
    canvas_width::{CanvasWidthMeasurer, SourceCanvasCache},
    cursor_blink::CursorBlinkController,
    options::RenderOptions,
    row::RowFactory,
    row_types::Selection,
    service_api::{BrowserService, RedrawRequest},
    text_blink::TextBlinkController,
};
use crate::{
    api::RenderDimensions, char_size_browser::CharSizeService,
    character_joiner::CharacterJoinerService, dom::BrowserDom, theme::ThemeService,
};
use std::{
    cell::{Cell, OnceCell, RefCell},
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
    browser: Option<Rc<dyn BrowserService>>,
    owns_metrics: bool,
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
    widths: OnceCell<SourceCanvasCache>,
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
            .add(self.theme.on_change_colors().try_subscribe(move |_| {
                if let Some(renderer) = weak.upgrade() {
                    renderer
                        .inject_css()
                        .map_err(crate::js_error::callback_error)?;
                }
                Ok(())
            }));
        if self.owns_metrics {
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
    }
    fn widths(&self) -> Result<&SourceCanvasCache, JsValue> {
        self.widths
            .get()
            .ok_or_else(|| js_sys::TypeError::new("width cache is not initialized").into())
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
        *self.options.borrow_mut() = options;
        let cols = self
            .terminal
            .cols()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let rows = self
            .terminal
            .rows()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        self.update_dimensions(cols, rows)?;
        self.inject_css()?;
        self.widths()?
            .set_font_from_options(&self.options_service)
            .map_err(width_error)?;
        self.set_default_spacing()?;
        self.update_blink_duration()
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
        if let Some(widths) = self.widths.get() {
            widths.dispose();
        }
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
    // Callers use known HTML tags. A realm-dependent instanceof would reject
    // valid nodes created by a same-origin iframe document, unlike the TS cast.
    let element: HtmlElement = document.create_element(tag)?.unchecked_into();
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
