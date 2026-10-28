//! Rust-backed browser Terminal. The public bindings share the native engine,
//! options and emitters; browser components own only DOM state and host jobs.

pub(crate) mod events;
#[cfg(feature = "acceptance-hooks")]
mod acceptance_snapshot;
#[cfg(all(test, target_arch = "wasm32"))]
mod a11y_event_tests;
mod event_functions;
mod operations;
mod browser_open;
mod browser_bindings;
mod browser_events;
mod browser_lifecycle;
mod viewport_services;
mod viewport_host;
pub mod accessibility_host;
mod protocols;
pub mod link_services;
mod input_services;
mod selection_services;
pub mod selection_api;
pub mod browser_services;
pub mod runtime;
pub mod addon_manager;
pub mod options;
pub mod input_host;
pub mod buffer_api;
pub mod buffer_cell;
pub mod parser_api;

use crate::{dom::BrowserDom, js_error::to_js_error, write_scheduler::BrowserWriteScheduler};
use event_functions::EventFunctions;
use js_sys::Function;
use std::{cell::{Cell, RefCell}, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_core::{headless, input::write_buffer::{WriteAction, WriteBuffer}};

pub(crate) struct Shared {
    pub native: Rc<headless::Terminal>,
    pub writer: WriteBuffer<BrowserWriteScheduler>,
    pub options: options::OptionsBinding,
    pub dom: RefCell<Option<Rc<BrowserDom>>>,
    pub events: EventFunctions,
    pub buffer: JsValue,
    pub parser: JsValue,
    pub selection: Rc<crate::selection::service::SelectionService>,
    pub decorations: Rc<xterm_core::decoration::DecorationService>,
    pub platform: xterm_core::platform::Platform,
    pub addons: addon_manager::BrowserAddonManager,
    pub runtime: RefCell<Rc<runtime::RuntimeContext>>,
    pub browser_events: Rc<browser_services::BrowserEvents>,
    pub will_open: xterm_core::event::Emitter<web_sys::HtmlElement>,
    pub services: RefCell<Option<Rc<browser_services::BrowserServices>>>,
    pub link_providers: crate::links::browser_linkifier::BrowserProviders,
    pub listeners: Vec<xterm_core::lifecycle::DisposableHandle>,
    pub custom_key: Rc<RefCell<Option<Function>>>,
    pub custom_wheel: Rc<RefCell<Option<Function>>>,
    pub public_disposed: Cell<bool>,
    pub core_disposed: Cell<bool>,
}

/// Browser API over the Rust engine. No upstream JavaScript terminal is loaded.
#[wasm_bindgen]
pub struct Terminal { pub(crate) shared: Rc<Shared> }

#[wasm_bindgen]
impl Terminal {
    #[wasm_bindgen(constructor)]
    pub fn new(initial_options: JsValue) -> Result<Terminal, JsValue> {
        let window = web_sys::window().ok_or_else(|| js_sys::Error::new("Browser Terminal requires a window"))?;
        let navigator = window.navigator();
        let user_agent = navigator.user_agent()?;
        let platform = navigator.platform()?;
        let platform = xterm_core::platform::Platform::detect(false, Some(
            xterm_core::platform::Navigator { user_agent: &user_agent, platform: &platform }
        )).map_err(|error| to_js_error(&error))?;
        let service = options::constructor_options(&initial_options, platform.is_mac)?;
        let native = Rc::new(headless::Terminal::with_options(service).map_err(|error| to_js_error(&error))?);
        native.with_state(|state| state.logger.set_console_sink(crate::logger::console_sink()))
            .map_err(|error| to_js_error(&error))?;
        let options = options::OptionsBinding::new(native.options().clone())?;
        let parse = Rc::clone(&native);
        let awaiting = Cell::new(false);
        let writer = WriteBuffer::new_owned(BrowserWriteScheduler::new(&window)?, move |_, data, resume| {
            let resume = if awaiting.get() { resume } else { None };
            let source = data.into_parse_source()?;
            let result = parse.parse_write_source(source, resume);
            for cleanup in parse.take_cleanup().map_err(xterm_core::input::write_buffer::WriteError::external)? {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(error) = cleanup.resolve().await {
                        crate::write_scheduler::report_unhandled_error(&crate::js_error::to_js_error(&error));
                    }
                });
            }
            awaiting.set(matches!(&result, Ok(WriteAction::Pending(_))));
            result
        });
        let browser_events = Rc::new(browser_services::BrowserEvents::default());
        let events = EventFunctions::new(&native, &browser_events);
        let input_writer = writer.downgrade();
        let core = native.with_state(|state| Rc::clone(&state.core)).map_err(|error| to_js_error(&error))?;
        let user_input = core.on_user_input().subscribe(move |_| {
            if let Some(writer) = input_writer.upgrade() { writer.handle_user_input(); }
        });
        let weak_writer = writer.downgrade();
        let disposal = native.on_dispose_request().try_subscribe(move |_| {
            if let Some(writer) = weak_writer.upgrade() { writer.try_dispose().map_err(xterm_core::event::EmitError::callback)?; }
            Ok(())
        });
        let weak_native = Rc::downgrade(&native);
        let parsed = writer.on_write_parsed().try_subscribe(move |_| {
            if let Some(native) = weak_native.upgrade() { native.notify_write_parsed()?; }
            Ok(())
        });
        let buffer = buffer_api::WebBufferNamespace::new(Rc::clone(&native))?.into();
        let parser = parser_api::WebParserApi::new(Rc::clone(&native)).into();
        let link_providers = link_services::create_registry(Rc::clone(&native), &window)?;
        let selection = Rc::new(crate::selection::service::SelectionService::new(
            crate::selection::service::SelectionPlatform { windows: platform.is_windows, linux: platform.is_linux, mac: platform.is_mac }));
        let decorations = Rc::new(native.with_state(|state| xterm_core::decoration::DecorationService::with_buffers(&state.buffers))
            .map_err(|error| to_js_error(&error))?.map_err(|error| to_js_error(&error))?);
        Ok(Self { shared: Rc::new(Shared { native, writer, options, events, buffer, parser, platform, selection, decorations, addons: addon_manager::BrowserAddonManager::new(), listeners: vec![disposal, parsed, user_input], custom_key: Rc::new(RefCell::new(None)), custom_wheel: Rc::new(RefCell::new(None)), public_disposed: Cell::new(false), core_disposed: Cell::new(false), link_providers, browser_events, will_open: xterm_core::event::Emitter::new(), runtime: RefCell::new(Rc::new(runtime::RuntimeContext::default())), services: RefCell::new(None), dom: RefCell::new(None) }) })
    }

    #[wasm_bindgen(js_name = _setRuntime)]
    pub fn set_runtime(&self, runtime: &runtime::BrowserRuntime) -> Result<(), JsValue> {
        if self.shared.dom.borrow().is_some() { return Err(js_sys::Error::new("Runtime must be selected before open").into()); }
        *self.shared.runtime.borrow_mut() = Rc::clone(&runtime.context);
        Ok(())
    }

    #[wasm_bindgen(getter)]
    pub fn cols(&self) -> Result<f64, JsValue> { self.shared.native.cols().map(|value| value as f64).map_err(|error| to_js_error(&error)) }
    #[wasm_bindgen(getter)]
    pub fn rows(&self) -> Result<f64, JsValue> { self.shared.native.rows().map(|value| value as f64).map_err(|error| to_js_error(&error)) }
    #[wasm_bindgen(getter)]
    pub fn options(&self) -> js_sys::Object { self.shared.options.object() }
    #[wasm_bindgen(setter)]
    pub fn set_options(&self, value: JsValue) -> Result<(), JsValue> { self.shared.options.assign(&value) }
    #[wasm_bindgen(getter)]
    pub fn buffer(&self) -> JsValue { self.shared.buffer.clone() }
    #[wasm_bindgen(getter)]
    pub fn parser(&self) -> JsValue { self.shared.parser.clone() }
    #[wasm_bindgen(getter)]
    pub fn element(&self) -> Option<web_sys::HtmlElement> { self.shared.dom.borrow().as_ref().map(|dom| dom.element.clone()) }
    #[wasm_bindgen(getter, js_name = screenElement)]
    pub fn screen_element(&self) -> Option<web_sys::HtmlElement> { self.shared.dom.borrow().as_ref().map(|dom| dom.screen.clone()) }
    #[wasm_bindgen(getter)]
    pub fn textarea(&self) -> Option<web_sys::HtmlTextAreaElement> { self.shared.dom.borrow().as_ref().map(|dom| dom.textarea.clone()) }
}

pub(crate) fn byte_view_payload(value: js_sys::Uint8Array) -> Rc<dyn xterm_core::input::write_buffer::WritePayload> {
    crate::byte_view::ByteView::from_view(value)
}

impl Terminal {
    pub fn on_will_open(&self) -> xterm_core::event::Event<web_sys::HtmlElement> { self.shared.will_open.event() }
    pub fn shared_native(&self) -> Rc<headless::Terminal> { Rc::clone(&self.shared.native) }
    pub fn selection_service(&self) -> Rc<crate::selection::service::SelectionService> { Rc::clone(&self.shared.selection) }
    pub fn decoration_service(&self) -> Rc<xterm_core::decoration::DecorationService> { Rc::clone(&self.shared.decorations) }
}

macro_rules! event_getters {
    ($($method:ident => $field:ident, $js:ident;)*) => {
        #[wasm_bindgen]
        impl Terminal {
            $(#[wasm_bindgen(getter, js_name = $js)]
            pub fn $method(&self) -> Function { self.shared.events.$field.clone() })*
        }
    }
}
event_getters! {
    on_bell => bell, onBell;
    on_binary => binary, onBinary;
    on_cursor_move => cursor, onCursorMove;
    on_data => data, onData;
    on_line_feed => line_feed, onLineFeed;
    on_resize => resize, onResize;
    on_scroll => scroll, onScroll;
    on_title_change => title, onTitleChange;
    on_write_parsed => parsed, onWriteParsed;
    on_key => key, onKey;
    on_render => render, onRender;
    on_selection_change => selection, onSelectionChange;
    on_dimensions_change => dimensions, onDimensionsChange;
}
