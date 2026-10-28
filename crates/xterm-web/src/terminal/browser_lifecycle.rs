use super::browser_services::BrowserServices;
use crate::{js_error::to_js_error, utf16::to_js_string};
use js_sys::Reflect;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use xterm_core::lifecycle::Disposable;

impl BrowserServices {
    pub fn sync_textarea(&self) -> Result<(), JsValue> {
        let input = self.input.borrow().clone();
        if input.as_ref().is_some_and(|input| input.is_composing()) {
            return Ok(());
        }
        let metrics = self.render_service.dimensions()?;
        let (core, buffer) = self
            .native
            .with_state(|state| (Rc::clone(&state.core), state.active_buffer()))
            .map_err(|error| to_js_error(&error))?;
        let placement = {
            let buffer = buffer.try_borrow().map_err(|error| to_js_error(&error))?;
            if !buffer.is_cursor_in_viewport() {
                return Ok(());
            }
            let cols = self.native.cols().map_err(|error| to_js_error(&error))?;
            let x = buffer.x.min(cols - 1);
            let Some(line) = buffer.lines.get(buffer.ybase + buffer.y) else {
                return Ok(());
            };
            let width = line
                .try_borrow()
                .map_err(|error| to_js_error(&error))?
                .get_width(x)
                .ok_or_else(|| js_sys::Error::new("Cursor cell missing"))?;
            (
                x as f64 * metrics.css.cell.width,
                buffer.y as f64 * metrics.css.cell.height,
                f64::from(width) * metrics.css.cell.width,
            )
        };
        // Cursor initialization lives in core; positioning itself must not turn
        // a hidden/never-used cursor on, especially during IME composition.
        drop(core);
        let style = self.dom.textarea.style();
        for (key, value) in [
            ("left", placement.0),
            ("top", placement.1),
            ("width", placement.2),
            ("height", metrics.css.cell.height),
            ("line-height", metrics.css.cell.height),
        ] {
            style.set_property(key, &format!("{value}px"))?;
        }
        style.set_property("z-index", "-5")
    }

    pub fn focus_changed(&self, focused: bool) -> Result<(), JsValue> {
        if !focused {
            self.dom.textarea.set_value("");
        }
        let core = self
            .native
            .with_state(|state| Rc::clone(&state.core))
            .map_err(|error| to_js_error(&error))?;
        if !focused {
            self.refresh_cursor()?;
        }
        if core.dec_private_modes.borrow().send_focus {
            self.native
                .input(
                    &if focused { "\x1b[I" } else { "\x1b[O" }
                        .encode_utf16()
                        .collect::<Vec<_>>(),
                    false,
                )
                .map_err(|error| to_js_error(&error))?;
        }
        self.dom
            .element
            .class_list()
            .toggle_with_force("focus", focused)?;
        if focused {
            if !core.is_cursor_initialized.get() {
                core.is_cursor_initialized.set(true);
                self.refresh_cursor()?;
            }
            self.render_service.handle_focus()?;
        } else {
            self.render_service.handle_blur()?;
            self.events
                .blur
                .fire(&())
                .map_err(|error| to_js_error(&error))?;
        }
        Ok(())
    }

    pub fn refresh_cursor(&self) -> Result<(), JsValue> {
        let y = self
            .native
            .with_state(|state| state.active_buffer().borrow().y)
            .map_err(|error| to_js_error(&error))?;
        self.refresh(y as f64, y as f64)
    }

    pub fn reset_browser_state(&self) -> Result<(), JsValue> {
        let input = self.input.borrow().clone();
        if let Some(input) = input {
            input.reset_mouse();
        }
        let buffers = self
            .native
            .with_state(|state| Rc::clone(&state.buffer_service))
            .map_err(|error| to_js_error(&error))?;
        let dimensions = crate::selection::service::buffer_dimensions(&buffers)
            .map_err(|error| to_js_error(&error))?;
        self.selection
            .clear_selection(&buffers, dimensions)
            .map_err(|error| to_js_error(&error))?;
        self.decorations
            .reset()
            .map_err(|error| to_js_error(&error))?;
        self.render_rows(0.0, (buffers.rows() - 1) as f64)
    }

    pub fn dispose(&self) -> Result<(), JsValue> {
        if self.disposed.replace(true) {
            return Ok(());
        }
        if let Some(input) = self.input.borrow_mut().take() {
            input.dispose()?;
        }
        if let Some(selection) = self.selection_dom.borrow_mut().take() {
            selection.dispose()?;
        }
        if let Some(links) = self.links.borrow_mut().take() {
            links.dispose()?;
        }
        super::accessibility_host::dispose(self)?;
        for mut listener in std::mem::take(&mut *self.dom_listeners.borrow_mut()) {
            listener.dispose()?;
        }
        for listener in std::mem::take(&mut *self.listeners.borrow_mut()) {
            listener.dispose();
        }
        self.render_service.dispose()?;
        self.metrics.state.dispose();
        self.core_browser.dispose()?;
        self.theme.dispose();
        self.selection
            .dispose()
            .map_err(|error| to_js_error(&error))?;
        self.decorations
            .dispose()
            .map_err(|error| to_js_error(&error))?;
        self.events.key.dispose();
        self.events.render.dispose();
        self.events.selection.dispose();
        self.events.dimensions.dispose();
        self.events.blur.dispose();
        self.events.dpr_change.dispose();
        Ok(())
    }

    pub fn set_textarea_value(&self, value: &[u16]) -> Result<(), JsValue> {
        if !Reflect::set(&self.dom.textarea, &"value".into(), &to_js_string(value))? {
            return Err(js_sys::TypeError::new("Cannot update textarea value").into());
        }
        Ok(())
    }
}
