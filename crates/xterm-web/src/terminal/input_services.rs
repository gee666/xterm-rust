//! Live rendering and selection callbacks for the engine input host.
use super::{
    browser_services::{BrowserServices, WeakBrowserServices},
    input_host::BrowserInputServices,
};
use crate::{
    js_error::to_js_error,
    selection::{
        model::Dimensions,
        pointer::{Modifiers, SelectionMouseOptions},
        service::SelectionPlatform,
    },
    utf16::to_js_string,
};
use std::num::NonZeroU32;
use wasm_bindgen::JsValue;
use web_sys::{HtmlElement, KeyboardEvent, MouseEvent, WheelEvent};
use xterm_core::lifecycle::DisposableHandle;

impl BrowserServices {
    pub fn input_dimensions(&self) -> Result<Dimensions, JsValue> {
        let (cols, rows, buffer) = self
            .native
            .with_state(|state| (state.cols(), state.rows(), state.active_buffer()))
            .map_err(|e| to_js_error(&e))?;
        let scrollback = buffer.try_borrow().map_err(|e| to_js_error(&e))?.ybase;
        Ok(Dimensions {
            columns: NonZeroU32::new(u32::try_from(cols).map_err(|e| to_js_error(&e))?)
                .ok_or_else(|| js_sys::Error::new("Columns must be positive"))?,
            rows: NonZeroU32::new(u32::try_from(rows).map_err(|e| to_js_error(&e))?)
                .ok_or_else(|| js_sys::Error::new("Rows must be positive"))?,
            scrollback: u32::try_from(scrollback).map_err(|e| to_js_error(&e))?,
        })
    }
    fn selection_options(&self) -> Result<SelectionMouseOptions, JsValue> {
        let (options, mouse) = self
            .native
            .with_state(|state| (state.options.clone(), state.mouse.clone()))
            .map_err(|e| to_js_error(&e))?;
        Ok(SelectionMouseOptions {
            mouse_events_require_alt: options
                .get("mouseEventsRequireAlt")
                .map_err(|e| to_js_error(&e))?
                .is_truthy(),
            mac_option_click_forces_selection: options
                .get("macOptionClickForcesSelection")
                .map_err(|e| to_js_error(&e))?
                .is_truthy(),
            mouse_events_active: mouse.are_mouse_events_active(),
        })
    }
    fn selection_platform(&self) -> SelectionPlatform {
        SelectionPlatform {
            mac: self.platform.is_mac,
            windows: self.platform.is_windows,
            linux: self.platform.is_linux,
        }
    }
}

impl BrowserInputServices for WeakBrowserServices {
    fn retain_host(&self) -> Result<std::rc::Rc<dyn std::any::Any>, JsValue> {
        Ok(self.upgrade()?)
    }
    fn render_dimensions(&self) -> Result<crate::api::RenderDimensions, JsValue> {
        self.upgrade()?.render_service.dimensions()
    }
    fn valid_char_size(&self) -> Result<bool, JsValue> {
        Ok(self.upgrade()?.metrics.state.has_valid_size())
    }
    fn dpr(&self) -> Result<f64, JsValue> {
        self.upgrade()?.core_browser.dpr()
    }
    fn has_selection(&self) -> Result<bool, JsValue> {
        let services = self.upgrade()?;
        services
            .selection
            .has_selection(services.input_dimensions()?)
            .map_err(|e| to_js_error(&e))
    }
    fn selection_text(&self) -> Result<Vec<u16>, JsValue> {
        let services = self.upgrade()?;
        let dimensions = services.input_dimensions()?;
        let buffer = services
            .native
            .with_state(|state| state.active_buffer())
            .map_err(|e| to_js_error(&e))?;
        let buffer = buffer.try_borrow().map_err(|e| to_js_error(&e))?;
        services
            .selection
            .selection_text(&buffer, dimensions)
            .map_err(|e| to_js_error(&e))
    }
    fn select_all(&self) -> Result<(), JsValue> {
        let services = self.upgrade()?;
        let buffer = services
            .native
            .with_state(|state| state.buffer_service.clone())
            .map_err(|e| to_js_error(&e))?;
        services
            .selection
            .select_all(&buffer, services.input_dimensions()?)
            .map_err(|e| to_js_error(&e))
    }
    fn right_click_select(&self, event: &MouseEvent) -> Result<(), JsValue> {
        let services = self.upgrade()?;
        let selection = services
            .selection_dom
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::Error::new("Selection DOM is not initialized"))?;
        selection.right_click_select(event)
    }
    fn force_selection(&self, event: &MouseEvent) -> Result<bool, JsValue> {
        let services = self.upgrade()?;
        Ok(crate::selection::pointer::should_force_selection(
            services.selection_platform(),
            services.selection_options()?,
            Modifiers {
                alt: event.alt_key(),
                shift: event.shift_key(),
            },
        ))
    }
    fn set_selection_enabled(&self, enabled: bool) -> Result<(), JsValue> {
        let services = self.upgrade()?;
        if enabled {
            services.selection.enable();
            return Ok(());
        }
        let buffer = services
            .native
            .with_state(|state| state.buffer_service.clone())
            .map_err(|e| to_js_error(&e))?;
        services
            .selection
            .disable(&buffer, services.input_dimensions()?)
            .map_err(|e| to_js_error(&e))
    }
    fn custom_key(&self, event: &KeyboardEvent) -> Result<bool, JsValue> {
        let services = self.upgrade()?;
        let handler = services.custom_key.borrow().clone();
        match handler {
            Some(handler) => Ok(handler
                .call1(&JsValue::UNDEFINED, event.as_ref())?
                .as_bool()
                != Some(false)),
            None => Ok(true),
        }
    }
    fn custom_wheel(&self, event: &WheelEvent) -> Result<bool, JsValue> {
        let services = self.upgrade()?;
        let handler = services.custom_wheel.borrow().clone();
        match handler {
            Some(handler) => Ok(handler
                .call1(&JsValue::UNDEFINED, event.as_ref())?
                .as_bool()
                != Some(false)),
            None => Ok(true),
        }
    }
    fn on_key(&self, key: &[u16], event: &KeyboardEvent) -> Result<(), JsValue> {
        let services = self.upgrade()?;
        let payload = super::events::key_event(&to_js_string(key), event)?;
        services
            .events
            .key
            .fire(&payload)
            .map_err(|error| to_js_error(&error))
    }
    fn update_cursor_style(&self, event: &KeyboardEvent) -> Result<(), JsValue> {
        let services = self.upgrade()?;
        let column = crate::selection::pointer::should_column_select(
            services.selection_platform(),
            services.selection_options()?,
            Modifiers {
                alt: event.alt_key(),
                shift: event.shift_key(),
            },
        );
        services
            .dom
            .element
            .class_list()
            .toggle_with_force("column-select", column)?;
        Ok(())
    }
    fn refresh_cursor(&self) -> Result<(), JsValue> {
        let services = self.upgrade()?;
        let buffer = services
            .native
            .with_state(|state| state.active_buffer())
            .map_err(|e| to_js_error(&e))?;
        let row = buffer.try_borrow().map_err(|e| to_js_error(&e))?.y;
        services.refresh(row as f64, row as f64)
    }
    fn touch_scroll(&self, translation_y: f64) -> Result<(), JsValue> {
        self.upgrade()?.handle_touch_scroll(translation_y)
    }
    fn gesture_target(&self, screen: &HtmlElement) -> Result<DisposableHandle, JsValue> {
        self.upgrade()?.gesture.add_target(screen)
    }
}
