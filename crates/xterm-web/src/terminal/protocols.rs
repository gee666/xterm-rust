use super::browser_services::BrowserServices;
use crate::{
    js_error::to_js_error,
    theme::{ColorRef, ColorSet, ColorSlot},
};
use wasm_bindgen::JsValue;
use xterm_core::{
    color::{channels, rgb, value},
    input::xparse_color::to_rgb_string,
    terminal::{ColorIndex, ColorRequest, WindowReport},
};

fn slot(index: ColorIndex) -> ColorSlot {
    match index {
        ColorIndex::Indexed(index) => ColorSlot::Ansi(index),
        ColorIndex::Foreground => ColorSlot::Foreground,
        ColorIndex::Background => ColorSlot::Background,
        ColorIndex::Cursor => ColorSlot::Cursor,
    }
}
fn current(colors: &ColorSet, index: ColorIndex) -> &ColorRef {
    match index {
        ColorIndex::Indexed(index) => &colors.ansi[usize::from(index)],
        ColorIndex::Foreground => &colors.foreground,
        ColorIndex::Background => &colors.background,
        ColorIndex::Cursor => &colors.cursor,
    }
}
impl BrowserServices {
    fn reply(&self, text: &str) -> Result<(), JsValue> {
        self.native
            .input(&text.encode_utf16().collect::<Vec<_>>(), false)
            .map_err(|error| to_js_error(&error))
    }
    pub fn color_requests(&self, requests: &[ColorRequest]) -> Result<(), JsValue> {
        for request in requests {
            match *request {
                ColorRequest::Report { index } => {
                    let colors = self.theme.colors();
                    let rgb = value::to_color_rgb(&current(&colors.borrow(), index).borrow());
                    let identifier = match index {
                        ColorIndex::Indexed(index) => format!("4;{index}"),
                        ColorIndex::Foreground => "10".into(),
                        ColorIndex::Background => "11".into(),
                        ColorIndex::Cursor => "12".into(),
                    };
                    self.reply(&format!(
                        "\x1b]{identifier};{}\x1b\\",
                        to_rgb_string(rgb, None)
                    ))?;
                }
                ColorRequest::Set { index, color } => {
                    let [r, g, b] = color;
                    self.theme
                        .modify_colors(|colors| {
                            let mut colors = colors.borrow_mut();
                            let value =
                                channels::to_color(i64::from(r), i64::from(g), i64::from(b), None)
                                    .into();
                            match index {
                                ColorIndex::Indexed(index) => {
                                    colors.ansi[usize::from(index)] = value
                                }
                                ColorIndex::Foreground => colors.foreground = value,
                                ColorIndex::Background => colors.background = value,
                                ColorIndex::Cursor => colors.cursor = value,
                            }
                        })
                        .map_err(|error| to_js_error(&error))?;
                }
                ColorRequest::Restore { index } => self
                    .theme
                    .restore_color(index.map(slot))
                    .map_err(|error| to_js_error(&error))?,
            }
        }
        Ok(())
    }
    pub fn report_color_scheme(&self) -> Result<(), JsValue> {
        let mode = {
            let colors = self.theme.colors();
            let colors = colors.borrow();
            let background = rgb::relative_luminance(colors.background.borrow().rgba >> 8);
            let foreground = rgb::relative_luminance(colors.foreground.borrow().rgba >> 8);
            if background < foreground {
                1
            } else {
                2
            }
        };
        self.reply(&format!("\x1b[?997;{mode}n"))
    }
    pub fn report_window(&self, report: WindowReport) -> Result<(), JsValue> {
        let dimensions = self.render_service.dimensions()?;
        let (kind, size) = match report {
            WindowReport::TextAreaPixels => (4, dimensions.css.canvas),
            WindowReport::CellPixels => (6, dimensions.css.cell),
        };
        let width = String::from(js_sys::Number::from(size.width).to_fixed(0)?);
        let height = String::from(js_sys::Number::from(size.height).to_fixed(0)?);
        self.reply(&format!("\x1b[{kind};{height};{width}t"))
    }
    pub fn report_focus(&self) -> Result<(), JsValue> {
        let enabled = self
            .native
            .with_state(|state| state.core.dec_private_modes.borrow().send_focus)
            .map_err(|error| to_js_error(&error))?;
        if enabled {
            self.reply(if self.dom.element.class_list().contains("focus") {
                "\x1b[I"
            } else {
                "\x1b[O"
            })?;
        }
        Ok(())
    }
}
