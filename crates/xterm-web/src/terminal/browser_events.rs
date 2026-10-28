use super::browser_services::BrowserServices;
use crate::{js_error::{callback_error, to_js_error}, selection::service::buffer_dimensions};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use xterm_core::terminal::TerminalEvent;

impl BrowserServices {
    pub(super) fn bind_events(self: &Rc<Self>) -> Result<(), JsValue> {
        let weak = Rc::downgrade(self);
        let resize = self.native.on_resize().try_subscribe(move |event| {
            if let Some(services) = weak.upgrade() {
                services.renderer.char_size_service().measure().map_err(callback_error)?;
                services.renderer.handle_resize(event.cols, event.rows).map_err(callback_error)?;
                services.sync_dimensions().map_err(callback_error)?;
                let ydisp = services.native.with_state(|state| state.active_buffer().borrow().ydisp)
                    .map_err(|error| callback_error(to_js_error(&error)))?;
                let links = services.links.borrow().clone();
                if let Some(links) = links { links.resize(event.cols, ydisp).map_err(callback_error)?; }
                services.synchronize_scroll().map_err(callback_error)?;
                services.refresh(0.0, (event.rows - 1) as f64).map_err(callback_error)?;
            }
            Ok(())
        });
        self.listeners.borrow_mut().push(resize);
        let weak = Rc::downgrade(self);
        let cursor = self.native.on_cursor_move().try_subscribe(move |_| {
            if let Some(services) = weak.upgrade() {
                services.renderer.handle_cursor_move().map_err(callback_error)?;
                services.sync_textarea().map_err(callback_error)?;
            }
            Ok(())
        });
        self.listeners.borrow_mut().push(cursor);
        let weak = Rc::downgrade(self);
        let scroll = self.native.on_scroll().try_subscribe(move |_| {
            if let Some(services) = weak.upgrade() { services.synchronize_scroll().map_err(callback_error)?; }
            Ok(())
        });
        self.listeners.borrow_mut().push(scroll);
        let weak = Rc::downgrade(self);
        let native = self.native.on_event().try_subscribe(move |event| {
            if let Some(services) = weak.upgrade() { services.route_native_event(event).map_err(callback_error)?; }
            Ok(())
        });
        self.listeners.borrow_mut().push(native);
        let weak = Rc::downgrade(self);
        let redraw = self.renderer.on_request_redraw().try_subscribe(move |request| {
            if let Some(services) = weak.upgrade() {
                if request.sync { services.render_rows(request.start, request.end).map_err(callback_error)?; }
                else { services.refresh(request.start as f64, request.end as f64).map_err(callback_error)?; }
            }
            Ok(())
        });
        self.listeners.borrow_mut().push(redraw);
        let weak = Rc::downgrade(self);
        let colors = self.theme.on_change_colors().try_subscribe(move |_| {
            if let Some(services) = weak.upgrade() {
                let enabled = services.native.with_state(|state| state.core.dec_private_modes.borrow().color_scheme_updates)
                    .map_err(|error| callback_error(to_js_error(&error)))?;
                if enabled { services.report_color_scheme().map_err(callback_error)?; }
            }
            Ok(())
        });
        self.listeners.borrow_mut().push(colors);
        Ok(())
    }

    fn route_native_event(&self, event: &TerminalEvent) -> Result<(), JsValue> {
        match event {
            TerminalEvent::Render { start, end } => self.refresh(*start as f64, *end as f64),
            TerminalEvent::InputScroll(_) => self.synchronize_scroll(),
            TerminalEvent::RequestReset => self.reset_browser_state(),
            TerminalEvent::SyncScrollBar => self.queue_viewport_sync(),
            TerminalEvent::SendFocus => self.report_focus(),
            TerminalEvent::ColorSchemeQuery => self.report_color_scheme(),
            TerminalEvent::Color(requests) => self.color_requests(requests),
            TerminalEvent::WindowReport(report) => self.report_window(*report),
            // These have dedicated typed subscriptions above, or are already
            // forwarded by the native/public event and accessibility bindings.
            TerminalEvent::Resize { .. } | TerminalEvent::Scroll(_) | TerminalEvent::CursorMove
            | TerminalEvent::Bell | TerminalEvent::LineFeed | TerminalEvent::WriteParsed
            | TerminalEvent::Data(_) | TerminalEvent::Binary(_) | TerminalEvent::TitleChange(_)
            | TerminalEvent::AccessibilityChar(_) | TerminalEvent::AccessibilityTab(_) => Ok(()),
        }
    }

    pub fn synchronize_scroll(&self) -> Result<(), JsValue> {
        let buffers = self.native.with_state(|state| Rc::clone(&state.buffer_service)).map_err(|error| to_js_error(&error))?;
        let dimensions = buffer_dimensions(&buffers).map_err(|error| to_js_error(&error))?;
        self.selection.refresh(&buffers, dimensions, false).map_err(|error| to_js_error(&error))?;
        let ydisp = buffers.buffer().try_borrow().map_err(|error| to_js_error(&error))?.ydisp;
        let links = self.links.borrow().clone();
        if let Some(links) = links { links.set_scroll_offset(ydisp); }
        self.queue_viewport_sync()
    }
}
