//! Browser link composition over the live Rust terminal and renderer.
use super::browser_services::BrowserServices;
use crate::{
    input::mouse_coords::{parse_css_padding, MouseGeometry},
    js_error::to_js_error,
    links::{
        browser_linkifier::{BrowserLinkifier, BrowserProviders, LinkCallback},
        native_osc::{NativeOscLinkProvider, OscLinkHandler},
        Link, LinkProviderService, Position, Range,
    },
    utf16::to_js_string,
};
use js_sys::{Array, Function, Object};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{MouseEvent, Window};
use xterm_core::{headless, options::OptionValue};

type RangeCache = Rc<RefCell<HashMap<*const Link, JsValue>>>;

pub fn create_registry(
    native: Rc<headless::Terminal>,
    window: &Window,
) -> Result<BrowserProviders, JsValue> {
    let providers: BrowserProviders = Rc::new(LinkProviderService::default());
    let weak = Rc::downgrade(&native);
    let settings = weak.clone();
    let provider = NativeOscLinkProvider::new(weak, window.clone(), move || {
        let native = settings
            .upgrade()
            .ok_or_else(|| js_sys::Error::new("Terminal has been disposed"))?;
        let value = native
            .options()
            .get("linkHandler")
            .map_err(|error| to_js_error(&error))?;
        if !value.is_truthy() {
            return Ok(None);
        }
        let protocol_handler = value.clone();
        let allow = Rc::new(move || {
            protocol_handler
                .get_property("allowNonHttpProtocols")
                .map(|value| value.is_truthy())
                .map_err(|error| to_js_error(&error))
        });
        let cache: RangeCache = Rc::new(RefCell::new(HashMap::new()));
        Ok(Some(OscLinkHandler {
            allow_non_http_protocols: allow,
            activate: handler_callback(value.clone(), "activate", false, cache.clone()),
            hover: Some(handler_callback(
                value.clone(),
                "hover",
                true,
                cache.clone(),
            )),
            leave: Some(handler_callback(value, "leave", true, cache)),
        }))
    });
    // Registration disposal is explicit; the service retains the first, highest
    // priority OSC provider until the owning terminal disposes the registry.
    providers.register(Rc::new(provider));
    Ok(providers)
}

fn handler_callback(
    handler: OptionValue,
    name: &'static str,
    optional: bool,
    cache: RangeCache,
) -> LinkCallback {
    Rc::new(move |event, link| {
        let member = handler
            .get_property(name)
            .map_err(|error| to_js_error(&error))?;
        let member = super::options::into_js(&member)?;
        if optional && (member.is_null() || member.is_undefined()) {
            return Ok(());
        }
        let function: Function = member.dyn_into().map_err(|_| {
            js_sys::TypeError::new(&format!("linkHandler.{name} is not a function"))
        })?;
        let key = link as *const Link;
        let existing = cache.borrow().get(&key).cloned();
        let range = match existing {
            Some(range) => range,
            None => {
                let range = range_object(link.range)?;
                cache.borrow_mut().insert(key, range.clone());
                range
            }
        };
        function.call3(
            &super::options::into_js(&handler)?,
            event.as_ref(),
            &to_js_string(&link.text),
            &range,
        )?;
        Ok(())
    })
}
fn range_object(range: Range) -> Result<JsValue, JsValue> {
    let start = Array::of2(&"start".into(), point_object(range.start)?.as_ref());
    let end = Array::of2(&"end".into(), point_object(range.end)?.as_ref());
    // fromEntries defines own data properties, as object literals do. Assignment
    // through Reflect.set would incorrectly invoke Object.prototype setters.
    Ok(Object::from_entries(Array::of2(start.as_ref(), end.as_ref()).as_ref())?.into())
}
fn point_object(position: Position) -> Result<Object, JsValue> {
    let x = Array::of2(&"x".into(), &JsValue::from_f64(position.x as f64));
    let y = Array::of2(&"y".into(), &JsValue::from_f64(position.y as f64));
    Object::from_entries(Array::of2(x.as_ref(), y.as_ref()).as_ref())
}

pub fn install(
    providers: BrowserProviders,
    services: &Rc<BrowserServices>,
) -> Result<Rc<BrowserLinkifier>, JsValue> {
    let (cols, _, offset) = snapshot(services)?;
    let coordinates = Rc::downgrade(services);
    let underline = Rc::downgrade(services);
    let linkifier = BrowserLinkifier::with_fallible_providers(
        services.dom.element.clone(),
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
                services.renderer.handle_link_underline(show, event)?;
            }
            Ok(())
        }),
        providers,
    )?;
    Ok(Rc::new(linkifier))
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
    let element = &services.dom.element;
    let bounds = element.get_bounding_client_rect();
    let window = services
        .dom
        .document
        .default_view()
        .ok_or_else(|| js_sys::Error::new("Link element has no owner window"))?;
    let style = window
        .get_computed_style(element)?
        .ok_or_else(|| js_sys::Error::new("Link element has no computed style"))?;
    let dimensions = services.renderer.dimensions();
    let geometry = MouseGeometry {
        rect_left: bounds.left(),
        rect_top: bounds.top(),
        padding_left: parse_css_padding(&style.get_property_value("padding-left")?),
        padding_top: parse_css_padding(&style.get_property_value("padding-top")?),
        valid_char_size: services.renderer.has_valid_char_size(),
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
