// Copyright (c) 2024 The xterm.js authors. All rights reserved. Licensed under MIT.
//! Actual DOM Viewport over the provided SmoothScrollableElement.
#[path = "viewport_bindings.rs"]
mod bindings;
#[path = "viewport_options.rs"]
mod options;
#[path = "viewport_styles.rs"]
mod styles;

use super::{SyncCallback, ViewportController, ViewportError, ViewportHost};
use crate::{
    js_error::to_js_error,
    renderer::service::RenderService,
    scroll::{
        browser_scheduler::{self, BrowserScrollable, QueueSource},
        element::SmoothScrollableElement,
        environment::ScrollEnvironment,
        state::{DimensionUpdate, PositionUpdate},
    },
    theme::ThemeService,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, HtmlElement};
use xterm_core::{
    buffer::BufferService, core_service::CoreService, event::Event, lifecycle::DisposableHandle,
    mouse_state::MouseStateService, options::OptionsService,
};

pub struct ViewportServices {
    pub buffers: Rc<BufferService>,
    pub core: Rc<CoreService>,
    pub mouse: Rc<MouseStateService>,
    pub options: Rc<OptionsService>,
    pub theme: Rc<ThemeService>,
    pub render: Rc<RenderService>,
    pub main_document: Document,
    pub queue_source: QueueSource,
    pub environment: Rc<ScrollEnvironment>,
}

struct DomHost {
    services: Rc<ViewportServices>,
    element: RefCell<Option<Rc<SmoothScrollableElement>>>,
}
impl DomHost {
    fn element(&self) -> Result<Rc<SmoothScrollableElement>, JsValue> {
        self.element
            .borrow()
            .clone()
            .ok_or_else(|| error("Viewport element is not initialized"))
    }
}
impl ViewportHost for DomHost {
    type Error = JsValue;
    fn display_row(&self) -> Result<f64, JsValue> {
        let buffer = self.services.buffers.buffer();
        let row = buffer
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .ydisp;
        Ok(row as f64)
    }
    fn cell_height(&self) -> Result<f64, JsValue> {
        Ok(self.services.render.dimensions()?.css.cell.height)
    }
    fn canvas_height(&self) -> Result<f64, JsValue> {
        Ok(self.services.render.dimensions()?.css.canvas.height)
    }
    fn buffer_line_count(&self) -> Result<f64, JsValue> {
        let buffer = self.services.buffers.buffer();
        let count = buffer
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .lines
            .len();
        Ok(count as f64)
    }
    fn synchronized_output(&self) -> Result<bool, JsValue> {
        Ok(self
            .services
            .core
            .dec_private_modes
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .synchronized_output)
    }
    fn scroll_top(&self) -> Result<f64, JsValue> {
        Ok(self.element()?.get_scroll_position().top)
    }
    fn set_dimensions(&self, height: f64, scroll_height: f64) -> Result<(), JsValue> {
        self.element()?.set_scroll_dimensions(DimensionUpdate {
            height: Some(height),
            scroll_height: Some(scroll_height),
            ..DimensionUpdate::default()
        })
    }
    fn set_scroll_top(&self, top: f64, reuse_animation: Option<bool>) -> Result<(), JsValue> {
        self.element()?.set_scroll_position(
            PositionUpdate {
                top: Some(top),
                left: None,
            },
            reuse_animation.unwrap_or(false),
        )
    }
    fn add_refresh_callback(&self, callback: SyncCallback<JsValue>) -> Result<i32, JsValue> {
        self.services
            .render
            .add_refresh_callback(move |_| callback().map_err(viewport_error))
    }
}

#[derive(Clone)]
enum Cleanup {
    Requests(Rc<ViewportController<DomHost>>),
    Model(BrowserScrollable),
    Element(Rc<SmoothScrollableElement>),
    Listener(DisposableHandle),
    Node(HtmlElement),
}
impl Cleanup {
    fn dispose(&self) -> Result<(), JsValue> {
        match self {
            Self::Requests(controller) => {
                controller.dispose();
                Ok(())
            }
            Self::Model(model) => model.dispose().map_err(browser_scheduler::scroll_error),
            Self::Element(element) => element.dispose(),
            Self::Listener(listener) => {
                listener.dispose();
                Ok(())
            }
            Self::Node(node) => {
                let remove: js_sys::Function =
                    js_sys::Reflect::get(node, &"remove".into())?.dyn_into()?;
                remove.call0(node).map(|_| ())
            }
        }
    }
}

pub struct Viewport {
    host: Rc<DomHost>,
    controller: Rc<ViewportController<DomHost>>,
    model: BrowserScrollable,
    root: HtmlElement,
    style: RefCell<Option<HtmlElement>>,
    cleanup: RefCell<Vec<Cleanup>>,
    disposed: Cell<bool>,
}
impl Viewport {
    pub fn new(
        root: HtmlElement,
        screen: HtmlElement,
        services: ViewportServices,
    ) -> Result<Rc<Self>, JsValue> {
        let services = Rc::new(services);
        let duration = services
            .options
            .get("smoothScrollDuration")
            .map_err(|error| to_js_error(&error))?
            .to_number()
            .map_err(|error| to_js_error(&error))?;
        let model = browser_scheduler::new_scrollable_with_queue_source(
            services.queue_source.clone(),
            false,
            duration,
        );
        let host = Rc::new(DomHost {
            services,
            element: RefCell::new(None),
        });
        let controller = ViewportController::new(Rc::clone(&host));
        let viewport = Rc::new(Self {
            host,
            controller: controller.clone(),
            model: model.clone(),
            root,
            style: RefCell::new(None),
            cleanup: RefCell::new(vec![Cleanup::Requests(controller), Cleanup::Model(model)]),
            disposed: Cell::new(false),
        });
        bindings::initialize(&viewport, screen)?;
        Ok(viewport)
    }
    pub fn on_request_scroll_lines(&self) -> Event<f64> {
        self.controller.on_request_scroll_lines()
    }
    pub fn queue_sync(self: &Rc<Self>, row: Option<f64>) -> Result<(), JsValue> {
        self.controller.queue_sync(row).map_err(viewport_error)
    }
    pub fn sync(&self, row: Option<f64>) -> Result<(), JsValue> {
        self.controller.sync(row).map_err(viewport_error)
    }
    pub fn scroll_lines(&self, displacement: f64) -> Result<(), JsValue> {
        self.controller
            .scroll_lines(displacement)
            .map_err(viewport_error)
    }
    pub fn scroll_to_line(&self, line: f64, disable_smooth: bool) -> Result<(), JsValue> {
        self.controller
            .scroll_to_line(line, disable_smooth)
            .map_err(viewport_error)
    }
    pub fn handle_touch_scroll(&self, translation_y: f64) -> Result<(), JsValue> {
        self.controller
            .handle_touch_scroll(translation_y)
            .map_err(viewport_error)
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        if self.disposed.replace(true) {
            return Ok(());
        }
        let mut index = 0;
        loop {
            let resource = self.cleanup.borrow().get(index).cloned();
            let Some(resource) = resource else {
                break;
            };
            resource.dispose()?;
            index += 1;
        }
        self.cleanup.borrow_mut().clear();
        Ok(())
    }
    fn track(&self, cleanup: Cleanup) {
        self.cleanup.borrow_mut().push(cleanup);
    }
    fn listener(&self, handle: DisposableHandle) {
        self.track(Cleanup::Listener(handle));
    }
}
impl Drop for Viewport {
    fn drop(&mut self) {
        if let Err(error) = self.dispose() {
            crate::write_scheduler::report_unhandled_error(&error);
        }
    }
}
fn viewport_error(error: ViewportError<JsValue>) -> JsValue {
    match error {
        ViewportError::Host(error) => error,
        ViewportError::Dispatch(error) => to_js_error(&error),
    }
}
fn error(message: &str) -> JsValue {
    js_sys::Error::new(message).into()
}
