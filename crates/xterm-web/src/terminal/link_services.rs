//! Browser link composition over the live Rust terminal and renderer.
use super::browser_services::BrowserServices;
use crate::{
    input::mouse_coords::{parse_css_padding, MouseGeometry},
    js_error::to_js_error,
    links::{
        browser_linkifier::{BrowserLinkifier, BrowserProviders},
        native_osc::NativeOscLinkProvider,
        LinkProviderService, Position,
    },
};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::{MouseEvent, Window};
use xterm_core::headless;

pub fn create_registry(
    native: Rc<headless::Terminal>,
    window: &Window,
) -> Result<BrowserProviders, JsValue> {
    let providers: BrowserProviders = Rc::new(LinkProviderService::default());
    let weak = Rc::downgrade(&native);
    let settings = weak.clone();
    let provider =
        NativeOscLinkProvider::with_javascript_handler(weak, window.clone(), move || {
            let native = settings
                .upgrade()
                .ok_or_else(|| js_sys::Error::new("Terminal has been disposed"))?;
            let value = native
                .options()
                .get("linkHandler")
                .map_err(|error| to_js_error(&error))?;
            super::options::into_js(&value)
        });
    // Registration disposal is explicit; the service retains the first, highest
    // priority OSC provider until the owning terminal disposes the registry.
    providers.register(Rc::new(provider));
    Ok(providers)
}

pub fn install(
    providers: BrowserProviders,
    services: &Rc<BrowserServices>,
) -> Result<Rc<BrowserLinkifier>, JsValue> {
    let (cols, _, offset) = snapshot(services)?;
    let coordinates = Rc::downgrade(services);
    let underline = Rc::downgrade(services);
    let linkifier = BrowserLinkifier::with_fallible_providers(
        services.dom.screen.clone(),
        cols,
        offset,
        Rc::new(move |event| {
            let Some(services) = coordinates.upgrade() else {
                return Ok(None);
            };
            coords(&services, event)
        }),
        Rc::new(move |show, event| {
            if let Some(services) = underline.upgrade() {
                if let Some(renderer) = services.render_service.current_renderer() {
                    if let Some(underline) = renderer.link_underline() {
                        underline.handle_link_underline(show, event)?;
                    }
                }
            }
            Ok(())
        }),
        providers,
    )?;
    let linkifier = Rc::new(linkifier);
    let weak = Rc::downgrade(&linkifier);
    services.listeners.borrow_mut().push(
        services
            .render_service
            .on_rendered_viewport_change()
            .try_subscribe(move |range| {
                if let Some(linkifier) = weak.upgrade() {
                    linkifier
                        .on_render(range.start, range.end)
                        .map_err(crate::js_error::callback_error)?;
                }
                Ok(())
            }),
    );
    Ok(linkifier)
}
fn snapshot(services: &BrowserServices) -> Result<(usize, usize, usize), JsValue> {
    services
        .native
        .with_state(|state| {
            let buffer = state.active_buffer();
            let buffer = buffer.try_borrow().map_err(|error| to_js_error(&error))?;
            Ok((state.cols(), state.rows(), buffer.ydisp))
        })
        .map_err(|error| to_js_error(&error))?
}
fn coords(services: &BrowserServices, event: &MouseEvent) -> Result<Option<Position>, JsValue> {
    let element = &services.dom.screen;
    let bounds = element.get_bounding_client_rect();
    let window = services.core_browser.window();
    let style = window
        .get_computed_style(element)?
        .ok_or_else(|| js_sys::Error::new("Link element has no computed style"))?;
    let dimensions = services.render_service.dimensions()?;
    let geometry = MouseGeometry {
        rect_left: bounds.left(),
        rect_top: bounds.top(),
        padding_left: parse_css_padding(&style.get_property_value("padding-left")?),
        padding_top: parse_css_padding(&style.get_property_value("padding-top")?),
        valid_char_size: services.metrics.state.has_valid_size(),
        cell_width: dimensions.css.cell.width,
        cell_height: dimensions.css.cell.height,
        canvas_width: dimensions.css.canvas.width,
        canvas_height: dimensions.css.canvas.height,
    };
    let (cols, rows, offset) = snapshot(services)?;
    let Some([x, y]) = geometry.coords(
        f64::from(event.client_x()),
        f64::from(event.client_y()),
        cols as f64,
        rows as f64,
        false,
    ) else {
        return Ok(None);
    };
    // Non-finite geometry cannot name a Rust buffer cell. Ordinary dimensions
    // are clamped by MouseGeometry before conversion, matching the source.
    if !x.is_finite() || !y.is_finite() {
        return Ok(None);
    }
    let y = (y as usize)
        .checked_add(offset)
        .ok_or_else(|| js_sys::RangeError::new("link buffer row overflow"))?;
    Ok(Some(Position { x: x as usize, y }))
}
