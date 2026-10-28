use super::Terminal;
use crate::write_scheduler::BrowserWriteScheduler;
use crate::{
    api::verify_integers,
    js_error::{to_js_error, JsCallbackError},
    utf16::from_js_string,
};
use js_sys::{Function, JsString};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::FocusOptions;
use xterm_core::input::write_buffer::{WriteCallback, WriteData, WriteError};

fn data(value: JsValue) -> Result<WriteData, JsValue> {
    if value.is_string() {
        Ok(WriteData::Text(from_js_string(
            &value.unchecked_into::<JsString>(),
        )))
    } else {
        // The source's Uint8Array annotation is a TypeScript cast. A realm-local
        // instanceof check would reject valid byte views from an iframe.
        Ok(WriteData::External(Rc::new(crate::byte_view::ByteView(
            value,
        ))))
    }
}
fn callback(value: Option<Function>) -> Option<WriteCallback<BrowserWriteScheduler>> {
    value
        .filter(|callback| callback.is_truthy())
        .map(|callback| {
            Rc::new(move |_: &_| {
                crate::byte_view::complete_write(callback.as_ref())
                    .map_err(|error| WriteError::external(JsCallbackError(error)))
            }) as WriteCallback<BrowserWriteScheduler>
        })
}
fn dimension(value: f64, minimum: usize) -> Result<usize, JsValue> {
    verify_integers(&[value]).map_err(|error| to_js_error(&error))?;
    let value = value.max(minimum as f64);
    if value > i32::MAX as f64 {
        return Err(js_sys::RangeError::new("Invalid array length").into());
    }
    Ok(value as usize)
}
#[wasm_bindgen]
impl Terminal {
    pub fn write(&self, value: JsValue, completion: Option<Function>) -> Result<(), JsValue> {
        self.shared
            .writer
            .write(data(value)?, callback(completion))
            .map_err(|error| to_js_error(&error))
    }
    pub fn writeln(&self, value: JsValue, completion: Option<Function>) -> Result<(), JsValue> {
        self.write(value, None)?;
        self.shared
            .writer
            .write(WriteData::text("\r\n"), callback(completion))
            .map_err(|error| to_js_error(&error))
    }
    pub fn input(&self, value: JsString, was_user_input: Option<bool>) -> Result<(), JsValue> {
        let was_user_input = was_user_input.unwrap_or(true);
        self.shared
            .native
            .input(&from_js_string(&value), was_user_input)
            .map_err(|error| to_js_error(&error))
    }
    pub fn resize(&self, columns: f64, rows: f64) -> Result<(), JsValue> {
        verify_integers(&[columns, rows]).map_err(|error| to_js_error(&error))?;
        if columns == self.cols()? && rows == self.rows()? {
            if let Some(services) = self.shared.services.borrow().clone() {
                if !services.metrics.state.has_valid_size() {
                    services.metrics.measure()?;
                }
            }
            return Ok(());
        }
        self.shared
            .writer
            .flush_sync()
            .map_err(|error| to_js_error(&error))?;
        // Preserve an unclamped request below the minimum so the native public
        // wrapper does not mistake it for the browser's same-size early return.
        self.shared
            .native
            .resize(dimension(columns, 0)?, dimension(rows, 0)?)
            .map_err(|error| to_js_error(&error))
    }
    pub fn reset(&self) -> Result<(), JsValue> {
        self.shared
            .native
            .reset()
            .map_err(|error| to_js_error(&error))?;
        let services = self.shared.services.borrow().clone();
        if let Some(services) = services {
            services.reset_browser_state()?;
        }
        Ok(())
    }
    pub fn clear(&self) -> Result<(), JsValue> {
        self.shared
            .native
            .clear()
            .map_err(|error| to_js_error(&error))
    }
    #[wasm_bindgen(js_name = scrollLines)]
    pub fn scroll_lines(&self, amount: f64) -> Result<(), JsValue> {
        verify_integers(&[amount]).map_err(|error| to_js_error(&error))?;
        let amount = self
            .shared
            .native
            .with_state(|state| {
                let buffer = state.active_buffer();
                let buffer = buffer.borrow();
                amount.clamp(-(buffer.ydisp as f64), (buffer.ybase - buffer.ydisp) as f64) as i64
            })
            .map_err(|error| to_js_error(&error))?;
        self.shared
            .native
            .scroll_lines(amount)
            .map_err(|error| to_js_error(&error))
    }
    #[wasm_bindgen(js_name = scrollPages)]
    pub fn scroll_pages(&self, amount: f64) -> Result<(), JsValue> {
        verify_integers(&[amount]).map_err(|error| to_js_error(&error))?;
        self.scroll_lines(amount * (self.rows()? - 1.0))
    }
    #[wasm_bindgen(js_name = scrollToTop)]
    pub fn scroll_to_top(&self) -> Result<(), JsValue> {
        self.shared
            .native
            .scroll_to_top()
            .map_err(|error| to_js_error(&error))
    }
    #[wasm_bindgen(js_name = scrollToBottom)]
    pub fn scroll_to_bottom(&self) -> Result<(), JsValue> {
        self.shared
            .native
            .scroll_to_bottom()
            .map_err(|error| to_js_error(&error))
    }
    #[wasm_bindgen(js_name = scrollToLine)]
    pub fn scroll_to_line(&self, line: f64) -> Result<(), JsValue> {
        verify_integers(&[line]).map_err(|error| to_js_error(&error))?;
        let line = self
            .shared
            .native
            .with_state(|state| {
                line.max(0.0)
                    .min(state.active_buffer().borrow().ybase as f64) as usize
            })
            .map_err(|error| to_js_error(&error))?;
        self.shared
            .native
            .scroll_to_line(line)
            .map_err(|error| to_js_error(&error))
    }
    pub fn focus(&self) -> Result<(), JsValue> {
        if let Some(dom) = self.shared.dom.borrow().as_ref() {
            let options = FocusOptions::new();
            options.set_prevent_scroll(true);
            dom.textarea.focus_with_options(&options)?;
        }
        Ok(())
    }
    pub fn blur(&self) -> Result<(), JsValue> {
        if let Some(dom) = self.shared.dom.borrow().as_ref() {
            dom.textarea.blur()?;
        }
        Ok(())
    }
    #[wasm_bindgen(js_name = _takeAddonOwner)]
    pub fn take_addon_owner(&self) -> Result<js_sys::Array, JsValue> {
        self.shared.addons.release_js_owner()
    }
    #[wasm_bindgen(js_name = _loadAddon)]
    pub fn load_addon_with_public_terminal(
        &self,
        public_terminal: JsValue,
        addon: JsValue,
    ) -> Result<(), JsValue> {
        self.shared.addons.load_addon(&public_terminal, &addon)
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        if self.shared.public_disposed.replace(true) {
            return Ok(());
        }
        self.dispose_core()?;
        self.shared.addons.dispose()
    }
    #[wasm_bindgen(js_name = _disposeCore)]
    pub fn dispose_core(&self) -> Result<(), JsValue> {
        if self.shared.core_disposed.replace(true) {
            return Ok(());
        }
        self.shared
            .native
            .dispose()
            .map_err(|error| to_js_error(&error))?;
        self.shared
            .writer
            .try_dispose()
            .map_err(|error| to_js_error(&error))?;
        if let Some(services) = self.shared.services.borrow().clone() {
            services.dispose()?;
        }
        if let Some(dom) = self.shared.dom.borrow().as_ref() {
            dom.element.remove();
        }
        self.shared.custom_key.borrow_mut().take();
        self.shared.custom_wheel.borrow_mut().take();
        for listener in &self.shared.listeners {
            listener.dispose();
        }
        for cleanup in self
            .shared
            .native
            .take_cleanup()
            .map_err(|error| to_js_error(&error))?
        {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(error) = cleanup.resolve().await {
                    crate::write_scheduler::report_unhandled_error(&to_js_error(&error));
                }
            });
        }
        if let Some(error) = self.shared.options.take_dispose_error() {
            return Err(error);
        }
        Ok(())
    }
}
impl Terminal {
    pub fn queued_write_handle(&self) -> Rc<dyn Fn(WriteData) -> Result<(), JsValue>> {
        let writer = self.shared.writer.clone();
        Rc::new(move |data| {
            writer
                .write(data, None)
                .map_err(|error| to_js_error(&error))
        })
    }
}
