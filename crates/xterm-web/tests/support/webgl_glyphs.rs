// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, WebGl2RenderingContext as Gl};
use xterm_web::{
    api::{CharacterDimensions, RenderDimensions, Size},
    webgl::{
        glyph_atlas::{AtlasPage, GlyphAtlas},
        glyphs::{GlyphCell, RasterizedGlyph},
        model::RenderModel,
    },
};

#[wasm_bindgen(inline_js = "
export function recordResources(gl, fail) {
  gl.__resources = [];
  gl.__uploads = 0;
  const texImage = gl.texImage2D.bind(gl);
  gl.texImage2D = (...args) => { if(args.length === 6) gl.__uploads++; return texImage(...args); };
  for (const name of ['Shader', 'Program', 'Buffer', 'VertexArray', 'Texture']) {
    const create = gl['create' + name].bind(gl);
    gl['create' + name] = (...args) => {
      const result = create(...args);
      if(result) gl.__resources.push([name, result]);
      return result;
    };
  }
  if(fail === 'shader') gl.shaderSource = shader => WebGL2RenderingContext.prototype.shaderSource.call(gl, shader, 'invalid shader');
  if(fail === 'uniform') gl.getUniformLocation = () => null;
  for(const name of ['Buffer','Texture']) if(fail.toLowerCase() === name.toLowerCase()) {
    const create = gl['create' + name].bind(gl);
    let count = 0;
    gl['create' + name] = () => ++count === 3 ? null : create();
  }
}
export function liveResources(gl) { return gl.__resources.filter(([name,value]) => gl['is'+name](value)).length; }
export function uploads(gl) { return gl.__uploads; }
export function loseContext(gl) {
  const ext = gl.getExtension('WEBGL_lose_context');
  if(!ext) throw Error('WEBGL_lose_context is required for this assertion');
  ext.loseContext();
}
export function startWarnings() {
  globalThis.__glyphWarnings = [];
  globalThis.__oldGlyphWarn = console.warn;
  console.warn = (...args) => globalThis.__glyphWarnings.push(args.join(' '));
}
export function stopWarnings() {
  console.warn = globalThis.__oldGlyphWarn;
  return globalThis.__glyphWarnings.length;
}
")]
extern "C" {
    #[wasm_bindgen(js_name = recordResources)]
    pub fn record_resources(gl: &Gl, fail: &str);
    #[wasm_bindgen(js_name = liveResources)]
    pub fn live_resources(gl: &Gl) -> usize;
    pub fn uploads(gl: &Gl) -> usize;
    #[wasm_bindgen(js_name = loseContext)]
    pub fn lose_context(gl: &Gl);
    #[wasm_bindgen(js_name = startWarnings)]
    pub fn start_warnings();
    #[wasm_bindgen(js_name = stopWarnings)]
    pub fn stop_warnings() -> usize;
}
pub fn canvas(width: u32, height: u32) -> HtmlCanvasElement {
    let c: HtmlCanvasElement = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();
    c.set_width(width);
    c.set_height(height);
    c
}
pub fn paint(canvas: &HtmlCanvasElement, color: &str) {
    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();
    ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    ctx.set_fill_style_str(color);
    ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
}
pub fn context() -> (HtmlCanvasElement, Gl) {
    let c = canvas(24, 20);
    let options = js_sys::Object::new();
    for key in ["antialias", "depth", "premultipliedAlpha"] {
        js_sys::Reflect::set(&options, &key.into(), &false.into()).unwrap();
    }
    js_sys::Reflect::set(&options, &"preserveDrawingBuffer".into(), &true.into()).unwrap();
    let gl: Gl = c
        .get_context_with_context_options("webgl2", &options)
        .unwrap()
        .expect("This browser must support WebGL2 to pass GPU assertions")
        .dyn_into()
        .unwrap();
    (c, gl)
}
pub fn dimensions() -> RenderDimensions {
    let mut d = RenderDimensions::default();
    d.device.canvas = Size {
        width: 24.0,
        height: 20.0,
    };
    d.device.cell = Size {
        width: 8.0,
        height: 10.0,
    };
    d.device.character = CharacterDimensions {
        width: 8.0,
        height: 10.0,
        left: 0.0,
        top: 0.0,
    };
    d
}
pub fn pixel(gl: &Gl, x: i32, top_y: i32) -> [u8; 4] {
    let mut values = [0; 4];
    gl.read_pixels_with_opt_u8_array(
        x,
        19 - top_y,
        1,
        1,
        Gl::RGBA,
        Gl::UNSIGNED_BYTE,
        Some(&mut values),
    )
    .unwrap();
    assert_eq!(gl.get_error(), Gl::NO_ERROR);
    values
}
pub fn clear(gl: &Gl) {
    gl.clear_color(0.0, 0.0, 0.0, 0.0);
    gl.clear(Gl::COLOR_BUFFER_BIT);
}
pub fn cell(x: usize, code: i64, page: u32) -> GlyphCell<'static> {
    GlyphCell {
        x,
        y: 0,
        code: Some(code),
        bg: 0,
        fg: 17,
        ext: page,
        chars: &[65],
        width: 1.0,
        last_bg: 0,
    }
}
pub fn model(lengths: &[u32]) -> RenderModel {
    RenderModel {
        line_lengths: lengths.to_vec(),
        ..Default::default()
    }
}
#[derive(Debug, PartialEq)]
pub struct Call {
    pub combined: bool,
    pub code: Option<i64>,
    pub chars: Vec<u16>,
    pub bg: u32,
    pub fg: u32,
    pub ext: u32,
}
pub struct TestAtlas {
    pub pages: RefCell<Vec<AtlasPage>>,
    pub version: RefCell<u64>,
    pub glyph: RefCell<RasterizedGlyph>,
    pub calls: RefCell<Vec<Call>>,
    pub elements: RefCell<Vec<Option<web_sys::HtmlElement>>>,
    pub failure: RefCell<Option<JsValue>>,
}
impl TestAtlas {
    pub fn new() -> Rc<Self> {
        let pages = ["red", "lime"]
            .into_iter()
            .enumerate()
            .map(|(i, color)| {
                let canvas = canvas(8, 10);
                paint(&canvas, color);
                AtlasPage {
                    canvas,
                    version: i as u64 + 1,
                }
            })
            .collect();
        Rc::new(Self {
            pages: RefCell::new(pages),
            version: RefCell::new(0),
            glyph: RefCell::new(RasterizedGlyph {
                size: [8.0, 10.0],
                texture_size: [1.0, 1.0],
                ..Default::default()
            }),
            calls: RefCell::new(Vec::new()),
            elements: RefCell::new(Vec::new()),
            failure: RefCell::new(None),
        })
    }
}
impl GlyphAtlas for TestAtlas {
    fn page_layout_version(&self) -> u64 {
        *self.version.borrow()
    }
    fn page_count(&self) -> usize {
        self.pages.borrow().len()
    }
    fn page(&self, index: usize) -> Option<AtlasPage> {
        self.pages.borrow().get(index).cloned()
    }
    fn rasterize(
        &self,
        cell: GlyphCell<'_>,
        combined: bool,
        element: Option<&web_sys::HtmlElement>,
    ) -> Result<RasterizedGlyph, JsValue> {
        if let Some(error) = self.failure.borrow().as_ref() {
            return Err(error.clone());
        }
        self.elements.borrow_mut().push(element.cloned());
        self.calls.borrow_mut().push(Call {
            combined,
            code: cell.code,
            chars: cell.chars.to_vec(),
            bg: cell.bg,
            fg: cell.fg,
            ext: cell.ext,
        });
        Ok(RasterizedGlyph {
            texture_page: cell.ext as usize,
            ..*self.glyph.borrow()
        })
    }
}
