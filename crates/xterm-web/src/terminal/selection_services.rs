//! Selection DOM callbacks into the opened terminal's shared Rust services.
use super::browser_services::WeakBrowserServices;
use crate::{
    js_error::to_js_error,
    selection::{
        dom::{LinkSelection, SelectionDomOptions, SelectionHost},
        model::Position,
        pointer::SelectionMouseOptions,
        service::{RedrawRequest, SelectionPlatform},
    },
    utf16::to_js_string,
};
use js_sys::Reflect;
use wasm_bindgen::JsValue;
use xterm_core::event::Event;

impl SelectionHost for WeakBrowserServices {
    fn options(&self) -> Result<SelectionDomOptions, JsValue> {
        let services = self.upgrade()?;
        let options = services.native.options();
        let flag = |key| {
            options
                .get(key)
                .map(|value| value.is_truthy())
                .map_err(|error| to_js_error(&error))
        };
        let (mouse, core) = services
            .native
            .with_state(|state| (state.mouse.clone(), state.core.clone()))
            .map_err(|error| to_js_error(&error))?;
        let application_cursor_keys = core
            .dec_private_modes
            .try_borrow()
            .map_err(|error| js_sys::Error::new(&error.to_string()))?
            .application_cursor_keys;
        Ok(SelectionDomOptions {
            platform: SelectionPlatform {
                windows: services.platform.is_windows,
                linux: services.platform.is_linux,
                mac: services.platform.is_mac,
            },
            mouse: SelectionMouseOptions {
                mouse_events_require_alt: flag("mouseEventsRequireAlt")?,
                mac_option_click_forces_selection: flag("macOptionClickForcesSelection")?,
                mouse_events_active: mouse.are_mouse_events_active(),
            },
            word_separator: options
                .get("wordSeparator")
                .and_then(|value| value.to_utf16())
                .map_err(|error| to_js_error(&error))?,
            alt_click_moves_cursor: flag("altClickMovesCursor")?,
            application_cursor_keys,
        })
    }
    fn current_link(&self) -> Result<Option<LinkSelection>, JsValue> {
        let services = self.upgrade()?;
        let linkifier = services.links.borrow().clone();
        let Some(link) = linkifier.and_then(|linkifier| linkifier.current_link()) else {
            return Ok(None);
        };
        let start_x = i64::try_from(link.range.start.x)
            .map_err(|error| js_sys::Error::new(&error.to_string()))?
            - 1;
        let start_y = i64::try_from(link.range.start.y)
            .map_err(|error| js_sys::Error::new(&error.to_string()))?
            - 1;
        let end_x = i64::try_from(link.range.end.x)
            .map_err(|error| js_sys::Error::new(&error.to_string()))?;
        let end_y = i64::try_from(link.range.end.y)
            .map_err(|error| js_sys::Error::new(&error.to_string()))?
            - 1;
        let columns = services
            .native
            .cols()
            .map_err(|error| to_js_error(&error))?;
        let columns =
            i64::try_from(columns).map_err(|error| js_sys::Error::new(&error.to_string()))?;
        let length = end_y
            .checked_sub(start_y)
            .and_then(|rows| rows.checked_mul(columns))
            .and_then(|length| length.checked_sub(start_x))
            .and_then(|length| length.checked_add(end_x))
            .ok_or_else(|| js_sys::Error::new("link selection range overflow"))?;
        Ok(Some(LinkSelection {
            start: Position::new(start_x, start_y),
            length,
        }))
    }
    fn on_user_input(&self) -> Result<Event<()>, JsValue> {
        self.upgrade()?
            .native
            .with_state(|state| state.core.on_user_input())
            .map_err(|error| to_js_error(&error))
    }
    fn scroll_lines(&self, amount: i64, suppress_scroll_event: bool) -> Result<(), JsValue> {
        self.upgrade()?.scroll_lines(amount, suppress_scroll_event)
    }
    fn data(&self, data: &[u16], user_input: bool) -> Result<(), JsValue> {
        self.upgrade()?
            .native
            .input(data, user_input)
            .map_err(|error| to_js_error(&error))
    }
    fn move_cursor(
        &self,
        position: Position,
        application_cursor_keys: bool,
    ) -> Result<(), JsValue> {
        let services = self.upgrade()?;
        let buffers = services
            .native
            .with_state(|state| state.buffer_service.clone())
            .map_err(|error| to_js_error(&error))?;
        let sequence = xterm_core::input::move_to_cell::move_to_cell_sequence(
            position.column,
            position.row,
            &buffers,
            application_cursor_keys,
        )
        .map_err(|error| to_js_error(&error))?;
        self.data(&sequence.encode_utf16().collect::<Vec<_>>(), true)
    }
    fn redraw(&self, event: RedrawRequest) -> Result<(), JsValue> {
        let renderer = self.upgrade()?.renderer.clone();
        renderer.handle_selection_changed(
            event.start.map(|point| [point.column, point.row]),
            event.end.map(|point| [point.column, point.row]),
            event.column_select_mode,
        )
    }
    fn selection_changed(&self) -> Result<(), JsValue> {
        self.upgrade()?
            .events
            .selection
            .fire(&())
            .map_err(|error| to_js_error(&error))
    }
    fn linux_mouse_selection(&self, text: &[u16]) -> Result<(), JsValue> {
        let textarea = self.upgrade()?.dom.textarea.clone();
        Reflect::set(&textarea, &"value".into(), &to_js_string(text))?;
        textarea.focus()?;
        textarea.select();
        Ok(())
    }
    fn report_error(&self, error: JsValue) {
        match self.upgrade() {
            Ok(services) => services.report_error(error),
            Err(_) => web_sys::console::error_1(&error),
        }
    }
}
