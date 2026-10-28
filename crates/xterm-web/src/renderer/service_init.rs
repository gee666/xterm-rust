use super::*;
use web_sys::HtmlElement;
impl RenderService {
    pub fn new(
        browser: Rc<dyn BrowserService>,
        screen: HtmlElement,
        terminal: Rc<Terminal>,
        theme: Rc<ThemeService>,
        metrics: Rc<CharSizeService>,
        decorations: DecorationService,
    ) -> Result<Rc<Self>, JsValue> {
        let (options, buffer, core, logger) = terminal
            .with_state(|state| {
                (
                    Rc::clone(&state.options),
                    Rc::clone(&state.buffer_service),
                    Rc::clone(&state.core),
                    Rc::clone(&state.logger),
                )
            })
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let rows = buffer.rows();
        // Common TaskQueue intentionally uses module-global scheduling, whereas
        // RAF/synchronized-output/observers use CoreBrowserService's live owner.
        let paused_resize = BrowserTaskQueue::source_idle_with_logger(logger)?;
        let service = Rc::new(Self {
            browser,
            core,
            buffer,
            options,
            metrics,
            renderer: RefCell::new(None),
            rows: Cell::new(rows),
            paused: Cell::new(false),
            needs_full: Cell::new(false),
            redraw_only: Cell::new(true),
            selection_dirty: Cell::new(false),
            selection: Cell::new(Selection::default()),
            debouncer: RefCell::new(None),
            paused_resize,
            synchronized: RefCell::new(None),
            observer: RefCell::new(None),
            observer_callback: RefCell::new(None),
            listeners: DisposableStore::new(),
            disposed: Cell::new(false),
            dimensions_change: Emitter::new(),
            viewport_change: Emitter::new(),
            render: Emitter::new(),
            refresh_request: Emitter::new(),
        });
        let browser = Rc::clone(&service.browser);
        let window: Rc<dyn Fn() -> web_sys::Window> = Rc::new(move || browser.window());
        let weak = Rc::downgrade(&service);
        let debouncer =
            BrowserRenderDebouncer::try_new_with_window_source(Rc::clone(&window), move |range| {
                if let Some(service) = weak.upgrade() {
                    service.render_rows(range.start, range.end)?;
                }
                Ok(())
            });
        *service.debouncer.borrow_mut() = Some(Rc::new(debouncer));
        let weak = Rc::downgrade(&service);
        let synchronized = SynchronizedOutput::new(window, Rc::clone(&service.core), move || {
            if let Some(service) = weak.upgrade() {
                service.full_refresh()?;
            }
            Ok(())
        });
        *service.synchronized.borrow_mut() = Some(synchronized);
        service.subscribe_sources(theme, decorations, screen.clone());
        service.register_observer(&service.browser.window(), &screen)?;
        Ok(service)
    }
    fn subscribe_sources(
        self: &Rc<Self>,
        theme: Rc<ThemeService>,
        decorations: DecorationService,
        screen: HtmlElement,
    ) {
        let weak = Rc::downgrade(self);
        self.listeners
            .add(self.browser.on_dpr_change().try_subscribe(move |_| {
                if let Some(service) = weak.upgrade() {
                    service
                        .handle_device_pixel_ratio_change()
                        .map_err(crate::js_error::callback_error)?;
                }
                Ok(())
            }));
        let weak = Rc::downgrade(self);
        self.listeners
            .add(self.buffer.on_resize().try_subscribe(move |_| {
                if let Some(service) = weak.upgrade() {
                    service
                        .full_refresh()
                        .map_err(crate::js_error::callback_error)?;
                }
                Ok(())
            }));
        let weak = Rc::downgrade(self);
        self.listeners.add(
            self.buffer
                .buffers
                .on_buffer_activate()
                .try_subscribe(move |_| {
                    if let Some(service) = weak.upgrade() {
                        service.clear().map_err(crate::js_error::callback_error)?;
                    }
                    Ok(())
                }),
        );
        let weak = Rc::downgrade(self);
        self.listeners
            .add(self.options.on_option_change().try_subscribe(move |_| {
                if let Some(service) = weak.upgrade() {
                    service
                        .options_changed()
                        .map_err(crate::js_error::callback_error)?;
                }
                Ok(())
            }));
        let weak = Rc::downgrade(self);
        self.listeners
            .add(self.metrics.state.on_change().try_subscribe(move |_| {
                if let Some(service) = weak.upgrade() {
                    service
                        .handle_char_size_changed()
                        .map_err(crate::js_error::callback_error)?;
                }
                Ok(())
            }));
        for event in [
            decorations.on_decoration_registered(),
            decorations.on_decoration_removed(),
        ] {
            let weak = Rc::downgrade(self);
            self.listeners.add(event.try_subscribe(move |_| {
                if let Some(service) = weak.upgrade() {
                    service
                        .full_refresh()
                        .map_err(crate::js_error::callback_error)?;
                }
                Ok(())
            }));
        }
        let weak = Rc::downgrade(self);
        self.listeners
            .add(self.options.on_option_change().try_subscribe(move |key| {
                if [
                    "drawBoldTextInBrightColors",
                    "letterSpacing",
                    "lineHeight",
                    "fontFamily",
                    "fontSize",
                    "fontWeight",
                    "fontWeightBold",
                    "minimumContrastRatio",
                    "rescaleOverlappingGlyphs",
                ]
                .contains(key)
                {
                    if let Some(service) = weak.upgrade() {
                        service.clear().map_err(crate::js_error::callback_error)?;
                        service
                            .handle_resize(service.buffer.cols(), service.buffer.rows())
                            .map_err(crate::js_error::callback_error)?;
                        service
                            .full_refresh()
                            .map_err(crate::js_error::callback_error)?;
                    }
                }
                Ok(())
            }));
        let weak = Rc::downgrade(self);
        self.listeners
            .add(self.options.on_option_change().try_subscribe(move |key| {
                if matches!(*key, "cursorBlink" | "cursorStyle") {
                    if let Some(service) = weak.upgrade() {
                        let y = service.buffer.buffer().borrow().y as f64;
                        service
                            .refresh_rows(y, y, false, true)
                            .map_err(crate::js_error::callback_error)?;
                    }
                }
                Ok(())
            }));
        let weak = Rc::downgrade(self);
        self.listeners
            .add(theme.on_change_colors().try_subscribe(move |_| {
                if let Some(service) = weak.upgrade() {
                    service
                        .full_refresh()
                        .map_err(crate::js_error::callback_error)?;
                }
                Ok(())
            }));
        let weak = Rc::downgrade(self);
        self.listeners.add(
            self.browser
                .on_window_change()
                .try_subscribe(move |window| {
                    if let Some(service) = weak.upgrade() {
                        service
                            .register_observer(window, &screen)
                            .map_err(crate::js_error::callback_error)?;
                    }
                    Ok(())
                }),
        );
    }
}
