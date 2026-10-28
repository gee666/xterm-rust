// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use xterm_addons::image::storage::{
    ImageResource, ImageStorage, ImageStorageOptions, TerminalStorageHost,
};
use xterm_core::headless::Terminal;
use xterm_web::image::{
    BrowserImageRenderer, BrowserImageSurface, CanvasDimensions, ImageCanvasHost,
};

pub fn get(value: &JsValue, key: &str) -> JsValue {
    js_sys::Reflect::get(value, &key.into()).unwrap()
}
pub fn call(value: &JsValue, key: &str, args: &[JsValue]) -> JsValue {
    get(value, key)
        .dyn_into::<js_sys::Function>()
        .unwrap()
        .apply(value, &args.iter().cloned().collect::<js_sys::Array>())
        .unwrap()
}
pub fn eval(code: &str) -> JsValue {
    js_sys::eval(code).unwrap()
}
pub fn pixels(canvas: &JsValue) -> Vec<u8> {
    let context = call(canvas, "getContext", &["2d".into()]);
    let data = call(
        &context,
        "getImageData",
        &[
            0.into(),
            0.into(),
            get(canvas, "width"),
            get(canvas, "height"),
        ],
    );
    js_sys::Uint8Array::new(&get(&get(&data, "data"), "buffer")).to_vec()
}

pub struct Host {
    pub document: JsValue,
    pub screen: JsValue,
    pub dimensions: Cell<Option<CanvasDimensions>>,
    pub screen_width: Cell<f64>,
}
impl Host {
    pub fn new(width: f64, height: f64) -> Rc<Self> {
        let document = web_sys::window().unwrap().document().unwrap();
        let screen = document.create_element("div").unwrap();
        screen.set_inner_html("<span>text</span>");
        document.body().unwrap().append_child(&screen).unwrap();
        Rc::new(Self {
            document: document.into(),
            screen: screen.into(),
            dimensions: Cell::new(Some(CanvasDimensions {
                cell_width: width,
                cell_height: height,
                canvas_width: 80.0,
                canvas_height: 60.0,
            })),
            screen_width: Cell::new(64.0),
        })
    }
    pub fn size(&self, width: f64, height: f64) {
        let mut d = self.dimensions.get().unwrap();
        d.cell_width = width;
        d.cell_height = height;
        self.dimensions.set(Some(d));
    }
}
impl ImageCanvasHost for Host {
    fn document(&self) -> Result<JsValue, JsValue> {
        Ok(self.document.clone())
    }
    fn screen_element(&self) -> Result<JsValue, JsValue> {
        Ok(self.screen.clone())
    }
    fn dimensions(&self) -> Result<Option<CanvasDimensions>, JsValue> {
        Ok(self.dimensions.get())
    }
    fn screen_width(&self) -> Result<f64, JsValue> {
        Ok(self.screen_width.get())
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        call(&self.screen, "remove", &[]);
    }
}

pub struct Harness {
    pub host: Rc<Host>,
    pub renderer: Rc<BrowserImageRenderer>,
    pub storage: ImageStorage,
    pub terminal: Rc<Terminal>,
}
impl Harness {
    pub fn new(width: f64, height: f64) -> Self {
        let host = Host::new(width, height);
        let renderer = Rc::new(BrowserImageRenderer::new(host.clone()).unwrap());
        let terminal = Rc::new(Terminal::new(20, 10, 10).unwrap());
        let terminal_host =
            Rc::new(TerminalStorageHost::from_terminal(&terminal, Rc::new(|_, _| Ok(()))).unwrap());
        let storage = ImageStorage::new(
            terminal_host,
            renderer.clone(),
            ImageStorageOptions::default(),
        )
        .unwrap();
        Self {
            host,
            renderer,
            storage,
            terminal,
        }
    }
    pub fn image(&self, width: u32, height: u32) -> ImageResource {
        let mut bytes = Vec::new();
        for y in 0..height {
            for x in 0..width {
                bytes.extend_from_slice(&[
                    (x * 17) as u8,
                    (y * 29) as u8,
                    ((x + y) * 11) as u8,
                    255,
                ]);
            }
        }
        let pixels = xterm_addons::image::types::RgbaImage::new(width, height, bytes).unwrap();
        self.renderer.upload_pixels(&pixels).unwrap()
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        self.renderer.dispose().unwrap();
    }
}
pub fn surface(image: &ImageResource) -> JsValue {
    image.surface::<BrowserImageSurface>().unwrap().value()
}

pub fn original(host: &Host) -> JsValue {
    let terminal = js_sys::Object::new();
    let core = eval("({open(){}, optionsService:{onOptionChange(){return {dispose(){}}}},_renderService:{setRenderer(){},refreshRows(){}}})");
    js_sys::Reflect::set(&core, &"screenElement".into(), &host.screen).unwrap();
    let browser = js_sys::Object::new();
    let window = js_sys::Object::new();
    js_sys::Reflect::set(&window, &"document".into(), &host.document).unwrap();
    js_sys::Reflect::set(&browser, &"window".into(), &window).unwrap();
    js_sys::Reflect::set(&core, &"_coreBrowserService".into(), &browser).unwrap();
    js_sys::Reflect::set(&terminal, &"_core".into(), &core).unwrap();
    js_sys::Reflect::set(&terminal, &"rows".into(), &10.into()).unwrap();
    set_dimensions(&terminal, host.dimensions.get().unwrap());
    let constructor = eval("globalThis.SourceImageRenderer")
        .dyn_into::<js_sys::Function>()
        .unwrap();
    js_sys::Reflect::construct(&constructor, &js_sys::Array::of1(&terminal)).unwrap()
}
pub fn set_dimensions(terminal: &JsValue, d: CanvasDimensions) {
    let dimensions = js_sys::Object::new();
    let css = js_sys::Object::new();
    let cell = js_sys::Object::new();
    let canvas = js_sys::Object::new();
    for (target, key, value) in [
        (&cell, "width", d.cell_width),
        (&cell, "height", d.cell_height),
        (&canvas, "width", d.canvas_width),
        (&canvas, "height", d.canvas_height),
    ] {
        js_sys::Reflect::set(target, &key.into(), &value.into()).unwrap();
    }
    js_sys::Reflect::set(&css, &"cell".into(), &cell).unwrap();
    js_sys::Reflect::set(&css, &"canvas".into(), &canvas).unwrap();
    js_sys::Reflect::set(&dimensions, &"css".into(), &css).unwrap();
    js_sys::Reflect::set(terminal, &"dimensions".into(), &dimensions).unwrap();
}
pub fn source_spec(image: JsValue, width: f64, height: f64, layer: &str) -> JsValue {
    let spec = js_sys::Object::new();
    let original = js_sys::Object::new();
    let actual = js_sys::Object::new();
    for target in [&original, &actual] {
        js_sys::Reflect::set(target, &"width".into(), &width.into()).unwrap();
        js_sys::Reflect::set(target, &"height".into(), &height.into()).unwrap();
    }
    for (key, value) in [
        ("orig", image.clone()),
        ("actual", image),
        ("origCellSize", original.into()),
        ("actualCellSize", actual.into()),
        ("layer", layer.into()),
    ] {
        js_sys::Reflect::set(&spec, &key.into(), &value).unwrap();
    }
    spec.into()
}
