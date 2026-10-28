//! Input's connection to the real Rust engine. Render/selection services remain
//! owned by the browser coordinator, never replaced by local input-only state.

#[path = "input_keyboard.rs"]
mod keyboard;
#[path = "input_mouse.rs"]
mod mouse;

use crate::{
    dom::BrowserDom,
    input::{
        composition_geometry::CursorGeometry,
        dom::{InputHost, InputOptions, OverlayLayout},
        keyboard_dispatch::KeyboardAction,
    },
};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::{KeyboardEvent, MouseEvent};
use xterm_core::{
    buffer::BufferService,
    core_service::CoreService,
    headless::Terminal,
    mouse_state::MouseStateService,
    options::{OptionValue, OptionsService},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct InputPlatform {
    pub mac: bool,
    pub windows: bool,
    pub linux: bool,
    pub firefox: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct RenderInputMetrics {
    pub valid: bool,
    pub cell_width: f64,
    pub cell_height: f64,
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub device_cell_height: f64,
    pub dpr: f64,
}

/// The coordinator delegates each method to its actual renderer, selection,
/// gesture and event services. Methods are synchronous and may reenter input.
pub trait BrowserInputServices {
    fn metrics(&self) -> Result<RenderInputMetrics, JsValue>;
    fn has_selection(&self) -> Result<bool, JsValue>;
    fn selection_text(&self) -> Result<Vec<u16>, JsValue>;
    fn select_all(&self) -> Result<(), JsValue>;
    fn right_click_select(&self, event: &MouseEvent) -> Result<(), JsValue>;
    fn force_selection(&self, event: &MouseEvent) -> Result<bool, JsValue>;
    fn set_selection_enabled(&self, enabled: bool) -> Result<(), JsValue>;
    fn custom_key(&self, event: &KeyboardEvent) -> Result<bool, JsValue>;
    fn custom_wheel(&self, event: &web_sys::WheelEvent) -> Result<bool, JsValue>;
    fn on_key(&self, key: &[u16], event: &KeyboardEvent) -> Result<(), JsValue>;
    fn update_cursor_style(&self, event: &KeyboardEvent) -> Result<(), JsValue>;
    fn refresh_cursor(&self) -> Result<(), JsValue>;
    fn touch_scroll(&self, translation_y: f64) -> Result<(), JsValue>;
    fn gesture_target(
        &self,
        screen: &web_sys::HtmlElement,
    ) -> Result<xterm_core::lifecycle::DisposableHandle, JsValue>;
    fn report_error(&self, error: JsValue);
}

pub struct EngineInputHost {
    pub(super) terminal: Rc<Terminal>,
    pub(super) dom: Rc<BrowserDom>,
    pub(super) services: Rc<dyn BrowserInputServices>,
    pub(super) platform: InputPlatform,
    pub(super) options: Rc<OptionsService>,
    pub(super) core: Rc<CoreService>,
    pub(super) buffer: Rc<BufferService>,
    pub(super) mouse: Rc<MouseStateService>,
}

impl EngineInputHost {
    pub fn new(
        terminal: Rc<Terminal>,
        dom: Rc<BrowserDom>,
        services: Rc<dyn BrowserInputServices>,
        platform: InputPlatform,
    ) -> Result<Self, JsValue> {
        let (options, core, buffer, mouse) = terminal
            .with_state(|state| {
                (
                    state.options.clone(),
                    state.core.clone(),
                    state.buffer_service.clone(),
                    state.mouse.clone(),
                )
            })
            .map_err(error)?;
        Ok(Self {
            terminal,
            dom,
            services,
            platform,
            options,
            core,
            buffer,
            mouse,
        })
    }

    pub(super) fn option_bool(&self, name: &str) -> Result<bool, JsValue> {
        Ok(self.options.get(name).map_err(error)?.is_truthy())
    }
    pub(super) fn option_number(&self, name: &str) -> Result<f64, JsValue> {
        self.options
            .get(name)
            .map_err(error)?
            .to_number()
            .map_err(error)
    }
    pub(super) fn extension(&self, name: &str) -> Result<bool, JsValue> {
        let extensions = self.options.get("vtExtensions").map_err(error)?;
        if matches!(extensions, OptionValue::Null | OptionValue::Undefined) {
            return Ok(false);
        }
        Ok(extensions.get_property(name).map_err(error)?.is_truthy())
    }
}

impl InputHost for EngineInputHost {
    fn options(&self) -> Result<InputOptions, JsValue> {
        Ok(InputOptions {
            bracketed_paste: self
                .core
                .dec_private_modes
                .try_borrow()
                .map_err(error)?
                .bracketed_paste_mode,
            ignore_bracketed_paste: self
                .options
                .get("ignoreBracketedPasteMode")
                .map_err(error)?
                == OptionValue::Bool(true),
            screen_reader: self.option_bool("screenReaderMode")?,
            right_click_selects_word: self.option_bool("rightClickSelectsWord")?,
            is_firefox: self.platform.firefox,
            is_linux: self.platform.linux,
        })
    }

    fn custom_key_handler(&self, event: &KeyboardEvent) -> Result<bool, JsValue> {
        self.services.custom_key(event)
    }

    fn keyboard_effect(
        &self,
        event: Option<&KeyboardEvent>,
        action: KeyboardAction<'_>,
    ) -> Result<(), JsValue> {
        match action {
            KeyboardAction::ScrollLines(lines) => self.terminal.scroll_lines(lines).map_err(error),
            KeyboardAction::ScrollToBottom => self.terminal.scroll_to_bottom().map_err(error),
            KeyboardAction::SelectAll => self.services.select_all(),
            KeyboardAction::OnKey(key) => self.services.on_key(
                key,
                event.ok_or_else(|| message("Missing onKey DOM event"))?,
            ),
            KeyboardAction::UpdateCursorStyle => self
                .services
                .update_cursor_style(event.ok_or_else(|| message("Missing cursor DOM event"))?),
            KeyboardAction::ShowCursor => {
                if !self.core.is_cursor_initialized.get() {
                    self.core.is_cursor_initialized.set(true);
                    self.services.refresh_cursor()?;
                }
                Ok(())
            }
            // These are local adapter effects and must never reach the host.
            KeyboardAction::Cancel
            | KeyboardAction::ClearTextarea
            | KeyboardAction::Data { .. }
            | KeyboardAction::Focus => Err(message("Unexpected local input effect")),
        }
    }

    fn overlay_layout(&self) -> Result<OverlayLayout, JsValue> {
        let metrics = self.services.metrics()?;
        let buffer = self.buffer.buffer();
        let (x, y, in_viewport) = {
            let buffer = buffer.try_borrow().map_err(error)?;
            (buffer.x, buffer.y, buffer.is_cursor_in_viewport())
        };
        Ok(OverlayLayout {
            cursor: CursorGeometry {
                x,
                y,
                cols: self.buffer.cols(),
                in_viewport,
                cell_width: metrics.cell_width,
                cell_height: metrics.cell_height,
                overlay_width: 0.0,
                overlay_height: 0.0,
            },
            font_family: self
                .options
                .get("fontFamily")
                .map_err(error)?
                .to_utf16()
                .map_err(error)?,
            font_size: self.option_number("fontSize")?,
        })
    }

    fn data(&self, data: &[u16], user_input: bool) -> Result<(), JsValue> {
        self.terminal.input(data, user_input).map_err(error)
    }
    fn has_selection(&self) -> Result<bool, JsValue> {
        self.services.has_selection()
    }
    fn selection_text(&self) -> Result<Vec<u16>, JsValue> {
        self.services.selection_text()
    }
    fn right_click_select(&self, event: &MouseEvent) -> Result<(), JsValue> {
        self.services.right_click_select(event)
    }

    fn sync_textarea(&self) -> Result<(), JsValue> {
        let metrics = self.services.metrics()?;
        let buffer = self.buffer.buffer();
        let (x, y, width) = {
            let buffer = buffer.try_borrow().map_err(error)?;
            if !buffer.is_cursor_in_viewport() {
                return Ok(());
            }
            let x = buffer.x.min(self.buffer.cols() - 1);
            let Some(line) = buffer.lines.get(buffer.ybase + buffer.y) else {
                return Ok(());
            };
            let width = line
                .try_borrow()
                .map_err(error)?
                .get_width(x)
                .ok_or_else(|| message("Cursor cell missing"))?;
            (x, buffer.y, width)
        };
        let style = self.dom.textarea.style();
        for (key, value) in [
            ("left", x as f64 * metrics.cell_width),
            ("top", y as f64 * metrics.cell_height),
            ("width", f64::from(width) * metrics.cell_width),
            ("height", metrics.cell_height),
            ("line-height", metrics.cell_height),
        ] {
            style.set_property(key, &format!("{value}px"))?;
        }
        style.set_property("z-index", "-5")
    }
    fn report_error(&self, value: JsValue) {
        self.services.report_error(value);
    }
}

pub(super) fn error(value: impl std::error::Error + 'static) -> JsValue {
    crate::js_error::to_js_error(&value)
}
fn message(value: &str) -> JsValue {
    js_sys::Error::new(value).into()
}
