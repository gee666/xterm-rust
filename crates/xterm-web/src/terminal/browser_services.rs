//! Browser component ownership. The shared RenderService owns the active backend;
//! input and addons always read its current dimensions after renderer replacement.

use super::Shared;
use crate::{
    accessibility::{AccessibilityManager, LocalizableStrings}, api::RenderDimensions,
    canvas_css::CanvasColorParser, char_size_browser::CharSizeService,
    character_joiner::CharacterJoinerService, core_browser::CoreBrowserService, dom::BrowserDom,
    input::dom::InputBindings, links::{browser_linkifier::BrowserLinkifier, debounce::RenderRange},
    renderer::{dom::DomRenderer, service::RenderService, service_api::{BrowserService, Renderer}},
    selection::{dom::SelectionDom, service::SelectionService}, theme::{Theme, ThemeService},
    write_scheduler::BrowserWriteScheduler,
};
use js_sys::Function;
use std::{cell::{Cell, RefCell}, rc::{Rc, Weak}};
use wasm_bindgen::JsValue;
use xterm_core::{decoration::DecorationService, event::Emitter, headless::Terminal,
    input::write_buffer::WriteBuffer, lifecycle::DisposableHandle, platform::Platform};

#[derive(Default)]
pub struct BrowserEvents {
    pub key: Emitter<JsValue>,
    pub render: Rc<Emitter<RenderRange>>,
    pub rendered_viewport: Rc<Emitter<RenderRange>>,
    pub selection: Emitter<()>,
    pub dimensions: Rc<Emitter<RenderDimensions>>,
    pub blur: Emitter<()>,
    pub dpr_change: Emitter<f64>,
}

pub struct BrowserServices {
    pub native: Rc<Terminal>,
    pub writer: WriteBuffer<BrowserWriteScheduler>,
    pub dom: Rc<BrowserDom>,
    pub core_browser: Rc<CoreBrowserService>,
    pub metrics: Rc<CharSizeService>,
    pub render_service: Rc<RenderService>,
    pub selection: Rc<SelectionService>,
    pub decorations: Rc<DecorationService>,
    pub theme: Rc<ThemeService>,
    pub joiners: Rc<CharacterJoinerService>,
    pub events: Rc<BrowserEvents>,
    pub platform: Platform,
    pub custom_key: Rc<RefCell<Option<Function>>>,
    pub custom_wheel: Rc<RefCell<Option<Function>>>,
    pub links: RefCell<Option<Rc<BrowserLinkifier>>>,
    pub input: RefCell<Option<Rc<InputBindings>>>,
    pub selection_dom: RefCell<Option<Rc<SelectionDom>>>,
    pub gesture: Rc<crate::input::touch::GestureContext>,
    pub strings: LocalizableStrings,
    pub accessibility: RefCell<Option<AccessibilityManager>>,
    pub listeners: RefCell<Vec<DisposableHandle>>,
    pub dom_listeners: RefCell<Vec<crate::dom_listener::DomListener>>,
    pub disposed: Cell<bool>,
}

#[derive(Clone)]
pub struct WeakBrowserServices(pub Weak<BrowserServices>);
impl WeakBrowserServices {
    pub fn upgrade(&self) -> Result<Rc<BrowserServices>, JsValue> {
        self.0.upgrade().ok_or_else(|| js_sys::Error::new("Browser services have been disposed").into())
    }
}

impl BrowserServices {
    pub(crate) fn new(shared: &Shared, dom: Rc<BrowserDom>) -> Result<Rc<Self>, JsValue> {
        let window = dom.element.owner_document().and_then(|document| document.default_view())
            .or_else(web_sys::window).ok_or_else(|| js_sys::Error::new("Terminal has no owner window"))?;
        let core_browser = CoreBrowserService::new(&dom.textarea, &window, &dom.document)?;
        let options = shared.native.with_state(|state| Rc::clone(&state.options))
            .map_err(|error| crate::js_error::to_js_error(&error))?;
        let metrics = CharSizeService::with_options(&dom.document, &dom.helpers, Rc::clone(&options))?;
        let parser = Rc::new(CanvasColorParser::new(&dom.document)?);
        let value = options.get("theme").map_err(|error| crate::js_error::to_js_error(&error))?;
        let theme_value = Theme::from_option_value(&value).map_err(|error| crate::js_error::to_js_error(&error))?;
        let theme = Rc::new(ThemeService::new_utf16(&theme_value, parser.utf16_theme_parser()));
        let joiners = Rc::new(CharacterJoinerService::new(|error| web_sys::console::error_1(&error.to_string().into())));
        let browser: Rc<dyn BrowserService> = core_browser.clone();
        let render_service = RenderService::new(browser, dom.screen.clone(), Rc::clone(&shared.native),
            Rc::clone(&theme), Rc::clone(&metrics), shared.decorations.as_ref().clone())?;
        let runtime = Rc::clone(&shared.runtime.borrow());
        let gesture = runtime.gesture(&dom.document)?;
        let services = Rc::new(Self {
            native: Rc::clone(&shared.native), writer: shared.writer.clone(), dom, core_browser, metrics, render_service,
            selection: Rc::clone(&shared.selection), decorations: Rc::clone(&shared.decorations), theme, joiners,
            events: Rc::clone(&shared.browser_events), platform: shared.platform,
            custom_key: Rc::clone(&shared.custom_key), custom_wheel: Rc::clone(&shared.custom_wheel),
            links: RefCell::new(None), input: RefCell::new(None), selection_dom: RefCell::new(None),
            gesture, strings: runtime.strings.clone(), accessibility: RefCell::new(None),
            listeners: RefCell::new(Vec::new()), dom_listeners: RefCell::new(Vec::new()), disposed: Cell::new(false),
        });
        // Source forwards public render first; decoration listeners are added
        // afterward. A public listener exception must stop that later work.
        services.listeners.borrow_mut().push(services.render_service.on_rendered_viewport_change()
            .forward(Rc::clone(&services.events.render)));
        services.listeners.borrow_mut().push(services.render_service.on_rendered_viewport_change()
            .forward(Rc::clone(&services.events.rendered_viewport)));
        services.listeners.borrow_mut().push(services.render_service.on_dimensions_change()
            .forward(Rc::clone(&services.events.dimensions)));
        Ok(services)
    }

    pub fn install_default_renderer(&self) -> Result<(), JsValue> {
        if self.render_service.has_renderer() { return Ok(()); }
        let browser: Rc<dyn BrowserService> = self.core_browser.clone();
        let renderer: Rc<dyn Renderer> = DomRenderer::new_with_browser(Rc::clone(&self.native), &self.dom,
            Rc::clone(&self.theme), Rc::clone(&self.joiners), self.decorations.as_ref().clone(),
            Rc::clone(&self.metrics), browser)?;
        self.render_service.set_renderer(renderer)
    }
    pub fn refresh(&self, start: f64, end: f64) -> Result<(), JsValue> {
        self.render_service.refresh_rows(start, end, false, false)
    }
    pub fn render_rows(&self, start: f64, end: f64) -> Result<(), JsValue> {
        self.render_service.refresh_rows(start, end, true, false)
    }
    pub fn add_refresh_callback(&self, callback: impl Fn(f64) -> Result<(), JsValue> + 'static) -> Result<i32, JsValue> {
        self.render_service.add_refresh_callback(callback)
    }
    pub fn mouse_geometry(&self) -> Result<crate::input::mouse_coords::MouseGeometry, JsValue> {
        let screen = &self.dom.screen;
        let rectangle = screen.get_bounding_client_rect();
        let window = self.core_browser.window();
        let style = window.get_computed_style(screen)?.ok_or_else(|| js_sys::Error::new("Screen computed style unavailable"))?;
        let dimensions = self.render_service.dimensions()?;
        Ok(crate::input::mouse_coords::MouseGeometry {
            rect_left: rectangle.left(), rect_top: rectangle.top(),
            padding_left: crate::input::mouse_coords::parse_css_padding(&style.get_property_value("padding-left")?),
            padding_top: crate::input::mouse_coords::parse_css_padding(&style.get_property_value("padding-top")?),
            valid_char_size: self.metrics.state.has_valid_size(), cell_width: dimensions.css.cell.width,
            cell_height: dimensions.css.cell.height, canvas_width: dimensions.css.canvas.width,
            canvas_height: dimensions.css.canvas.height,
        })
    }
    pub fn report_error(&self, error: JsValue) { crate::write_scheduler::report_unhandled_error(&error); }
}
