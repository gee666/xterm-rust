//! Construct the actual Viewport from the opened browser services. Native state
//! is borrowed only to clone service owners, never across DOM or event callbacks.
use super::browser_services::BrowserServices;
use crate::{
    core_browser::CoreBrowserService,
    js_error::to_js_error,
    renderer::service::RenderService,
    scroll::{browser_scheduler::QueueSource, environment::ScrollEnvironment},
    viewport::{Viewport, ViewportServices},
};
use std::rc::Rc;
use wasm_bindgen::JsValue;

pub(crate) fn create_viewport(
    services: &Rc<BrowserServices>,
    render: Rc<RenderService>,
    browser: Rc<CoreBrowserService>,
    queue_source: QueueSource,
    environment: Rc<ScrollEnvironment>,
) -> Result<Rc<Viewport>, JsValue> {
    let (buffers, core, mouse, options) = services
        .native
        .with_state(|state| {
            (
                Rc::clone(&state.buffer_service),
                Rc::clone(&state.core),
                Rc::clone(&state.mouse),
                Rc::clone(&state.options),
            )
        })
        .map_err(|error| to_js_error(&error))?;
    Viewport::new(
        services.dom.element.clone(),
        services.dom.screen.clone(),
        ViewportServices {
            buffers,
            core,
            mouse,
            options,
            theme: Rc::clone(&services.theme),
            render,
            main_document: browser.main_document(),
            queue_source,
            environment,
        },
    )
}
