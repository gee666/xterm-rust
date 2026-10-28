use super::*;
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
        Self::create(terminal, dom, theme, joiners, decoration, None, None)
    }
    pub fn new_with_metrics(
        terminal: Rc<Terminal>,
        dom: &BrowserDom,
        theme: Rc<ThemeService>,
        joiners: Rc<CharacterJoinerService>,
        decoration: DecorationService,
        metrics: Rc<CharSizeService>,
    ) -> Result<Rc<Self>, JsValue> {
        Self::create(
            terminal,
            dom,
            theme,
            joiners,
            decoration,
            Some(metrics),
            None,
        )
    }
    pub fn new_with_browser(
        terminal: Rc<Terminal>,
        dom: &BrowserDom,
        theme: Rc<ThemeService>,
        joiners: Rc<CharacterJoinerService>,
        decoration: DecorationService,
        metrics: Rc<CharSizeService>,
        browser: Rc<dyn BrowserService>,
    ) -> Result<Rc<Self>, JsValue> {
        Self::create(
            terminal,
            dom,
            theme,
            joiners,
            decoration,
            Some(metrics),
            Some(browser),
        )
    }
    fn create(
        terminal: Rc<Terminal>,
        dom: &BrowserDom,
        theme: Rc<ThemeService>,
        joiners: Rc<CharacterJoinerService>,
        decoration: DecorationService,
        metrics: Option<Rc<CharSizeService>>,
        browser: Option<Rc<dyn BrowserService>>,
    ) -> Result<Rc<Self>, JsValue> {
        let options_service = terminal
            .with_state(|state| Rc::clone(&state.options))
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let options =
            RenderOptions::read(&options_service).map_err(|e| crate::js_error::to_js_error(&e))?;
        let owns_metrics = metrics.is_none();
        let metrics = match metrics {
            Some(metrics) => metrics,
            None => CharSizeService::with_options(
                &dom.document,
                &dom.helpers,
                Rc::clone(&options_service),
            )?,
        };
        if owns_metrics {
            metrics.measure()?;
        }
        let row_container = element(&dom.document, "div", "xterm-rows")?;
        row_container
            .style()
            .set_property("line-height", "normal")?;
        row_container.set_attribute("aria-hidden", "true")?;
        let selection_container = element(&dom.document, "div", "xterm-selection")?;
        selection_container.set_attribute("aria-hidden", "true")?;
        let dimensions_style = element(&dom.document, "style", "")?;
        let theme_style = element(&dom.document, "style", "")?;
        let window = match &browser {
            Some(browser) => browser.window(),
            None => dom
                .document
                .default_view()
                .ok_or_else(|| js_sys::Error::new("Terminal document has no owner window"))?,
        };
        let renderer = Rc::new(Self {
            terminal,
            document: dom.document.clone(),
            window,
            browser,
            owns_metrics,
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
            focused: Cell::new(false),
            selection: RefCell::new(Selection::default()),
            last_selection: Cell::new(Selection::default()),
            selection_model: Cell::new(
                crate::renderer::selection_model::SelectionRenderModel::default(),
            ),
            theme,
            joiners,
            decoration,
            metrics,
            widths: OnceCell::new(),
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
        renderer.subscribe();
        renderer.inject_css()?;
        renderer
            .element
            .class_list()
            .add_1(&renderer.owner_class())?;
        renderer.screen.append_child(&renderer.row_container)?;
        renderer
            .screen
            .append_child(&renderer.selection_container)?;
        let focused = match &renderer.browser {
            Some(browser) => browser.is_focused()?,
            None => dom
                .document
                .active_element()
                .is_some_and(|active| active.is_same_node(Some(dom.textarea.as_ref()))),
        };
        renderer.focused.set(focused);
        renderer.initialize_blink()?;
        // Source creates the canvases only after injecting CSS and installing
        // its listeners. Font conversion can call back into those listeners.
        renderer
            .widths
            .set(SourceCanvasCache::new(CanvasWidthMeasurer::new(
                &dom.document,
            )?))
            .map_err(|_| js_sys::Error::new("width cache already initialized"))?;
        renderer
            .widths()?
            .set_font_from_options(&renderer.options_service)
            .map_err(width_error)?;
        renderer.set_default_spacing()?;
        Ok(renderer)
    }
}
