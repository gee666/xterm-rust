// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{HtmlElement, HtmlTextAreaElement};
use xterm_core::{buffer::CellData, headless::Terminal, options::OptionsService};
use xterm_web::{
    api::{CharacterDimensions, RenderDimensions, Size},
    core_browser::CoreBrowserService,
    terminal::runtime::RuntimeContext,
    theme::ThemeService,
    webgl::{
        browser_frame::CellSource,
        cell_colors::{CellColorContext, CellColorResolver},
        layer_services::LayerServices,
        model::CursorShape,
        renderer_types::{ModelCell, UpdateContext},
    },
};
#[wasm_bindgen(inline_js = "
export function sourceFrame(name) {return globalThis.__webglFrameSource.sourceFrame(name);}
export function frameInputs(name) {return JSON.stringify(globalThis.__webglFrameSource.frameInputs(name));}
export async function releaseFrameContext(gl) {
 const ext=gl.getExtension('WEBGL_lose_context'); if(!ext)throw Error('WEBGL_lose_context required');
 await new Promise(resolve => { gl.canvas.addEventListener('webglcontextlost',resolve,{once:true}); ext.loseContext(); });
 // WebKit frees the active context after event dispatch returns. A real frame
 // boundary avoids allocating its successor inside the lost-event dispatch.
 await new Promise(requestAnimationFrame);
}")]
extern "C" {
    #[wasm_bindgen(catch, js_name=sourceFrame)]
    pub async fn source_frame(name: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name=frameInputs)]
    pub fn inputs(name: &str) -> String;
    #[wasm_bindgen(js_name=releaseFrameContext)]
    pub async fn release(gl: &web_sys::WebGl2RenderingContext);
}
pub struct Harness {
    pub services: Rc<LayerServices>,
    pub screen: HtmlElement,
    textarea: HtmlTextAreaElement,
}
impl Harness {
    pub fn new(runtime: &RuntimeContext, attached: bool) -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let screen: HtmlElement = document.create_element("div").unwrap().unchecked_into();
        let textarea: HtmlTextAreaElement = document
            .create_element("textarea")
            .unwrap()
            .unchecked_into();
        if attached {
            document.body().unwrap().append_child(&screen).unwrap();
        }
        document.body().unwrap().append_child(&textarea).unwrap();
        let browser = CoreBrowserService::new(&textarea, &window, &document).unwrap();
        let options = Rc::new(OptionsService::new(
            false,
            [
                ("cols", 8.0.into()),
                ("rows", 4.0.into()),
                ("fontFamily", "monospace".into()),
                ("fontSize", 14.0.into()),
                ("fontWeight", 400.0.into()),
                ("fontWeightBold", 700.0.into()),
                ("blinkIntervalDuration", 0.0.into()),
            ],
        ));
        let terminal = Rc::new(Terminal::with_options(options).unwrap());
        Self {
            services: Rc::new(LayerServices {
                terminal,
                browser,
                theme: Rc::new(ThemeService::default()),
                cache: runtime.webgl_atlas_cache(&document),
            }),
            screen,
            textarea,
        }
    }
    pub fn context(&self) -> UpdateContext {
        let (buffer, core) = self
            .services
            .terminal
            .with_state(|s| (s.active_buffer(), s.core.clone()))
            .unwrap();
        let b = buffer.borrow();
        UpdateContext {
            cols: 8,
            rows: 4,
            viewport_y: b.ydisp,
            cursor_x: b.x,
            cursor_y: b.ybase + b.y,
            focused: false,
            cursor_initialized: core.is_cursor_initialized.get(),
            cursor_hidden: core.is_cursor_hidden.get(),
            cursor_blink_visible: true,
            cursor_style: CursorShape::Block,
            inactive_style: CursorShape::Outline,
            cursor_width: 1.0,
            device_pixel_ratio: 1.0,
            text_blink_enabled: false,
            text_blink_on: true,
        }
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        self.services
            .cache
            .remove_terminal(&Rc::downgrade(&self.services.terminal))
            .unwrap();
        self.services.browser.dispose().unwrap();
        self.screen.remove();
        self.textarea.remove();
    }
}
pub fn dimensions() -> RenderDimensions {
    let mut d = RenderDimensions::default();
    d.device.character = CharacterDimensions {
        width: 8.0,
        height: 16.0,
        left: 0.0,
        top: 0.0,
    };
    d.device.cell = Size {
        width: 8.0,
        height: 16.0,
    };
    d.device.canvas = Size {
        width: 64.0,
        height: 64.0,
    };
    d.css.cell = d.device.cell;
    d.css.canvas = d.device.canvas;
    d
}
pub struct Source {
    pub services: Rc<LayerServices>,
    pub blinking: bool,
    pub fail: Option<JsValue>,
}
impl Source {
    fn line(&self, row: usize) -> Option<Rc<std::cell::RefCell<xterm_core::buffer::BufferLine>>> {
        self.services
            .terminal
            .with_state(|s| s.active_buffer().borrow().lines.get(row))
            .unwrap()
    }
}
impl CellSource for Source {
    fn has_line(&mut self, row: usize) -> Result<bool, JsValue> {
        Ok(self.line(row).is_some())
    }
    fn joined_characters(&mut self, _row: usize) -> Result<Vec<[usize; 2]>, JsValue> {
        Ok(Vec::new())
    }
    fn load_cell(&mut self, row: usize, x: usize, cell: &mut CellData) -> Result<(), JsValue> {
        if let Some(e) = self.fail.take() {
            return Err(e);
        }
        self.line(row).unwrap().borrow().load_cell(x, cell).unwrap();
        Ok(())
    }
    fn joined_text(&mut self, _row: usize, _range: [usize; 2]) -> Result<Vec<u16>, JsValue> {
        Err(js_sys::Error::new("no joiner registered in this terminal").into())
    }
    fn resolve_colors(
        &mut self,
        cell: &ModelCell,
        x: usize,
        row: usize,
        selected: bool,
        resolver: &mut CellColorResolver,
    ) -> Result<(), JsValue> {
        resolver
            .resolve(
                &cell.attributes,
                cell.code.unwrap_or(0),
                CellColorContext {
                    x: x as f64,
                    y: row as f64,
                    selected,
                    focused: false,
                    device_cell_width: 8.0,
                    device_cell_height: 16.0,
                    font_size: 14.0,
                    device_pixel_ratio: 1.0,
                },
                &self.services.theme.colors().borrow(),
                &[],
            )
            .map_err(|e| js_sys::Error::new(&e.to_string()).into())
    }
    fn cursor_colors(&mut self) -> Result<[u32; 2], JsValue> {
        let c = self.services.theme.colors();
        let c = c.borrow();
        let colors = [c.cursor_accent.borrow().rgba, c.cursor.borrow().rgba];
        Ok(colors)
    }
    fn rescale_overlapping_glyphs(&mut self) -> Result<bool, JsValue> {
        Ok(false)
    }
    fn needs_text_blink(&mut self, needed: bool) -> Result<(), JsValue> {
        self.blinking = needed;
        Ok(())
    }
    fn cursor_visible(&self) -> bool {
        true
    }
}
pub fn snapshot(frame: &xterm_web::webgl::browser_frame::BrowserFrame) -> serde_json::Value {
    let gl = frame.context().gl();
    gl.finish();
    let mut pixels = vec![0; 64 * 64 * 4];
    gl.read_pixels_with_opt_u8_array(
        0,
        0,
        64,
        64,
        web_sys::WebGl2RenderingContext::RGBA,
        web_sys::WebGl2RenderingContext::UNSIGNED_BYTE,
        Some(&mut pixels),
    )
    .unwrap();
    let hash = pixels.iter().fold(2166136261u32, |hash, pixel| {
        (hash ^ u32::from(*pixel)).wrapping_mul(16777619)
    });
    let cursor = frame.model().cursor.map(|c| {
        serde_json::json!({"x":c.x,"y":c.y,"width":c.width,
        "style":"outline","cursorWidth":c.cursor_width,"devicePixelRatio":c.device_pixel_ratio})
    });
    serde_json::json!({"cells":frame.model().cells,"lengths":frame.model().line_lengths,"cursor":cursor,
        "hash":hash,"nonzero":pixels.iter().filter(|p|**p!=0).count(),"pages":frame.atlas().unwrap().canvases().len()})
}
