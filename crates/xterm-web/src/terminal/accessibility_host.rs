//! Accessibility reads the actual active buffer and shares browser selection.
//! Weak service ownership permits reentrant callbacks without reference cycles.
use super::browser_services::{BrowserServices, WeakBrowserServices};
use crate::{
    accessibility::browser::{AccessibilityBuffer, Listener},
    accessibility::{AccessibilityEvent, AccessibilityHost, Dimensions, Row},
    js_error::{to_js_error, JsCallbackError},
    selection::{model::Position, service::buffer_dimensions},
    utf16::from_js_string,
};
use js_sys::{JsString, Reflect};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use xterm_core::{
    buffer::buffer_set::BufferRef,
    event::{EmitError, Event},
    lifecycle::DisposableHandle,
    terminal::TerminalEvent,
};

pub(super) fn update(services: &Rc<BrowserServices>) -> Result<(), JsValue> {
    let enabled = services
        .native
        .options()
        .get("screenReaderMode")
        .map_err(|error| to_js_error(&error))?
        .is_truthy();
    let current = services.accessibility.borrow().clone();
    match (enabled, current) {
        (true, None) => {
            let host: Rc<dyn AccessibilityHost> =
                Rc::new(WeakBrowserServices(Rc::downgrade(services)));
            // Source TimeBasedDebouncer uses the module realm's globals, even
            // when AccessibilityManager creates nodes in another document.
            let window = web_sys::window()
                .ok_or_else(|| js_sys::Error::new("Browser Terminal requires a window"))?;
            let manager = crate::accessibility::AccessibilityManager::new(
                window,
                services.dom.document.clone(),
                Some(&services.dom.element),
                host,
                services.strings.clone(),
            )?;
            services.accessibility.replace(Some(manager));
        }
        (false, Some(manager)) => {
            manager.dispose()?;
            services.accessibility.borrow_mut().take();
        }
        (true, Some(_)) | (false, None) => {}
    }
    Ok(())
}
pub(super) fn dispose(services: &BrowserServices) -> Result<(), JsValue> {
    let manager = services.accessibility.borrow().clone();
    if let Some(manager) = manager {
        manager.dispose()?;
        services.accessibility.borrow_mut().take();
    }
    Ok(())
}

// Retain identity, not a copied row snapshot or a borrow guard. DOM callbacks
// may reset the terminal or mutate this buffer between rows in the same batch.
struct RenderBuffer(BufferRef);
impl AccessibilityBuffer for RenderBuffer {
    fn display_offset(&self) -> Result<usize, JsValue> {
        Ok(self
            .0
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .ydisp)
    }
    fn line_count(&self) -> Result<usize, JsValue> {
        Ok(self
            .0
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .lines
            .len())
    }
    fn row(&self, absolute: usize) -> Result<Option<Row>, JsValue> {
        let line = self
            .0
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .lines
            .get(absolute);
        line.map(|line| {
            let line = line.try_borrow().map_err(|error| to_js_error(&error))?;
            let mut columns = Vec::new();
            let text = line.translate_to_string(true, None, None, Some(&mut columns));
            Ok(Row { text, columns })
        })
        .transpose()
    }
}

impl AccessibilityHost for WeakBrowserServices {
    fn rows(&self) -> Result<usize, JsValue> {
        self.upgrade()?
            .native
            .rows()
            .map_err(|error| to_js_error(&error))
    }
    fn cols(&self) -> Result<usize, JsValue> {
        self.upgrade()?
            .native
            .cols()
            .map_err(|error| to_js_error(&error))
    }
    fn render_buffer(self: Rc<Self>) -> Result<Rc<dyn AccessibilityBuffer>, JsValue> {
        let buffers = self
            .upgrade()?
            .native
            .with_state(|state| state.buffer_service.clone())
            .map_err(|error| to_js_error(&error))?;
        Ok(Rc::new(RenderBuffer(buffers.buffer())))
    }
    fn buffer_length(&self) -> Result<usize, JsValue> {
        let buffers = self
            .upgrade()?
            .native
            .with_state(|state| state.buffer_service.clone())
            .map_err(|error| to_js_error(&error))?;
        let active = buffers.buffer();
        let result = active
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .lines
            .len();
        Ok(result)
    }
    fn dimensions(&self) -> Result<Dimensions, JsValue> {
        let services = self.upgrade()?;
        let dimensions = services.render_service.dimensions()?;
        let font_size = services
            .native
            .options()
            .get("fontSize")
            .map_err(|error| to_js_error(&error))?
            .to_number()
            .map_err(|error| to_js_error(&error))?;
        Ok(Dimensions {
            cell_width: dimensions.css.cell.width,
            cell_height: dimensions.css.cell.height,
            canvas_width: dimensions.css.canvas.width,
            font_size,
        })
    }
    fn scroll_lines(&self, amount: i32) -> Result<(), JsValue> {
        self.upgrade()?.scroll_lines(i64::from(amount), false)
    }
    fn clear_selection(&self) -> Result<(), JsValue> {
        let services = self.upgrade()?;
        let buffers = services
            .native
            .with_state(|state| state.buffer_service.clone())
            .map_err(|error| to_js_error(&error))?;
        let dimensions = buffer_dimensions(&buffers).map_err(|error| to_js_error(&error))?;
        services
            .selection
            .clear_selection(&buffers, dimensions)
            .map_err(|error| to_js_error(&error))
    }
    fn select(&self, column: f64, row: f64, length: f64) -> Result<(), JsValue> {
        let services = self.upgrade()?;
        let buffers = services
            .native
            .with_state(|state| state.buffer_service.clone())
            .map_err(|error| to_js_error(&error))?;
        let dimensions = buffer_dimensions(&buffers).map_err(|error| to_js_error(&error))?;
        services
            .selection
            .set_selection(
                &buffers,
                dimensions,
                Position::new(integer(column)?, integer(row)?),
                integer(length)?,
            )
            .map_err(|error| to_js_error(&error))
    }
    fn subscribe(&self, listener: Listener) -> Result<Vec<DisposableHandle>, JsValue> {
        let services = self.upgrade()?;
        let mut handles = vec![
            forward(services.native.on_resize(), &listener, |event| {
                Ok(Some(AccessibilityEvent::Resize(event.rows)))
            }),
            forward(services.events.render.event(), &listener, |event| {
                Ok(Some(AccessibilityEvent::Render {
                    start: event.start,
                    end: event.end,
                }))
            }),
            forward(services.native.on_scroll(), &listener, |_| {
                Ok(Some(AccessibilityEvent::Scroll))
            }),
            forward(services.native.on_event(), &listener, |event| {
                Ok(match event {
                    TerminalEvent::AccessibilityChar(text) => {
                        Some(AccessibilityEvent::Character(text.clone()))
                    }
                    _ => None,
                })
            }),
            forward(services.native.on_line_feed(), &listener, |_| {
                Ok(Some(AccessibilityEvent::LineFeed))
            }),
            forward(services.native.on_event(), &listener, |event| {
                Ok(match event {
                    TerminalEvent::AccessibilityTab(count) => Some(AccessibilityEvent::Tab(*count)),
                    _ => None,
                })
            }),
            forward(services.events.key.event(), &listener, |event| {
                let key = Reflect::get(event, &"key".into())?;
                if !key.is_string() {
                    return Err(js_sys::TypeError::new("onKey.key must be a string").into());
                }
                Ok(Some(AccessibilityEvent::Key(from_js_string(
                    &key.unchecked_into::<JsString>(),
                ))))
            }),
            forward(services.events.blur.event(), &listener, |_| {
                Ok(Some(AccessibilityEvent::Blur))
            }),
            forward(services.events.dimensions.event(), &listener, |_| {
                Ok(Some(AccessibilityEvent::Dimensions))
            }),
        ];
        handles.push(forward(
            services.events.dpr_change.event(),
            &listener,
            |_| Ok(Some(AccessibilityEvent::Dimensions)),
        ));
        Ok(handles)
    }
}
fn forward<T: 'static>(
    event: Event<T>,
    listener: &Listener,
    map: impl Fn(&T) -> Result<Option<AccessibilityEvent>, JsValue> + 'static,
) -> DisposableHandle {
    let listener = Rc::clone(listener);
    event.try_subscribe(move |value| {
        map(value)
            .and_then(|event| event.map_or(Ok(()), |event| listener(event)))
            .map_err(|error| EmitError::callback(JsCallbackError(error)))
    })
}
fn integer(value: f64) -> Result<i64, JsValue> {
    // Core coordinates use signed i64; validate before conversion rather than
    // saturating or truncating NaN/fractions supplied by edited DOM attributes.
    if !value.is_finite()
        || value.fract() != 0.0
        || value < i64::MIN as f64
        || value >= i64::MAX as f64
    {
        return Err(js_sys::RangeError::new(
            "accessibility selection is outside integer coordinate bounds",
        )
        .into());
    }
    Ok(value as i64)
}
