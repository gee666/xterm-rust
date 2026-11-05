// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use serde_json::{json, Value};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::prelude::*;
use xterm_core::{
    color::Color, log_service::LogService, options::OptionsService,
    unicode_service::UnicodeService, unicode_v6::UnicodeV6,
};
use xterm_web::webgl::{
    browser_cache::{AtlasRequest, AtlasServices, BrowserAtlasCache},
    cache::CacheError,
    config::AtlasConfig,
    glyph_atlas::GlyphAtlas,
    texture_atlas::{GlyphKey, TextureAtlas},
};

#[wasm_bindgen(module = "/tests/browser_atlas_cache_probe.js")]
extern "C" {
    #[wasm_bindgen(js_name = beginCacheCapture)]
    fn begin();
    #[wasm_bindgen(js_name = endCacheCapture)]
    fn end() -> String;
    #[wasm_bindgen(js_name = cacheNote)]
    pub fn note(text: &str);
    #[wasm_bindgen(js_name = getCacheFailure)]
    pub fn failure() -> JsValue;
    #[wasm_bindgen(js_name = failCacheRemove)]
    pub fn fail_remove();
    #[wasm_bindgen(js_name = failCacheCreate)]
    pub fn fail_create();
    #[wasm_bindgen(js_name = reenterCacheRemove)]
    pub fn reenter_remove(callback: &js_sys::Function);
    #[wasm_bindgen(js_name = reenterCacheCreate)]
    pub fn reenter_create(callback: &js_sys::Function);
    #[wasm_bindgen(js_name = cacheCanvasDigest)]
    fn digest(canvas: &web_sys::HtmlCanvasElement) -> u32;
    #[wasm_bindgen(js_name = sourceCache)]
    pub fn source(name: &str) -> String;
}

pub struct Capture(bool);
impl Capture {
    pub fn new() -> Self {
        begin();
        Self(true)
    }
    pub fn finish(mut self) -> Value {
        self.0 = false;
        serde_json::from_str(&end()).unwrap()
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        if self.0 {
            end();
        }
    }
}

type Owner = xterm_core::headless::Terminal;
pub struct Harness {
    pub pool: Rc<BrowserAtlasCache<Owner>>,
    pub a: Rc<Owner>,
    pub b: Rc<Owner>,
    pub c: Rc<Owner>,
    pub config: AtlasConfig,
    atlases: RefCell<Vec<Rc<TextureAtlas>>>,
    ids: RefCell<Vec<usize>>,
    versions: RefCell<Vec<i64>>,
    base_version: Cell<Option<u64>>,
    states: RefCell<Vec<Value>>,
    pixels: RefCell<Vec<u32>>,
}
impl Harness {
    pub fn terminal() -> Rc<Owner> {
        Rc::new(Owner::new(80, 24, 100).unwrap())
    }
    pub fn new() -> Self {
        let mut config = crate::webgl_support::config();
        config.colors.foreground = Color {
            css: "#eeeeee".into(),
            rgba: 0xeeeeeeff,
        }
        .into();
        config.colors.background = Color {
            css: "#102030".into(),
            rgba: 0x102030ff,
        }
        .into();
        config.colors.ansi = vec![config.colors.foreground.clone(); 256];
        Self {
            pool: xterm_web::terminal::runtime::RuntimeContext::default()
                .webgl_atlas_cache(&web_sys::window().unwrap().document().unwrap()),
            a: Self::terminal(),
            b: Self::terminal(),
            c: Self::terminal(),
            config,
            atlases: RefCell::new(Vec::new()),
            ids: RefCell::new(Vec::new()),
            versions: RefCell::new(Vec::new()),
            base_version: Cell::new(None),
            states: RefCell::new(Vec::new()),
            pixels: RefCell::new(Vec::new()),
        }
    }
    pub fn services(&self, label: &str) -> AtlasServices {
        note(&format!("services:{label}"));
        Self::raw_services()
    }
    pub fn raw_services() -> AtlasServices {
        let unicode = Rc::new(UnicodeService::new());
        unicode.register(Rc::new(UnicodeV6::new())).unwrap();
        let log = Rc::new(
            LogService::new(
                Rc::new(OptionsService::default()),
                xterm_wasm_support::logger::console_sink(),
            )
            .unwrap(),
        );
        AtlasServices { unicode, log }
    }
    pub fn acquire(&self, owner: &Rc<Owner>, label: &str, size: f64) -> Rc<TextureAtlas> {
        self.try_acquire(owner, size, || Ok(self.services(label)))
            .unwrap()
    }
    pub fn try_acquire(
        &self,
        owner: &Rc<Owner>,
        size: f64,
        services: impl FnOnce() -> Result<AtlasServices, JsValue>,
    ) -> Result<Rc<TextureAtlas>, CacheError<JsValue>> {
        let mut options = self.config.options.clone();
        options.font_size = size;
        let atlas = self.pool.acquire(
            owner,
            AtlasRequest {
                dimensions: self.config.dimensions,
                options: &options,
                colors: &self.config.colors,
                custom_glyphs: true,
            },
            services,
        )?;
        let mut atlases = self.atlases.borrow_mut();
        let index = atlases
            .iter()
            .position(|a| Rc::ptr_eq(a, &atlas))
            .unwrap_or_else(|| {
                atlases.push(atlas.clone());
                atlases.len() - 1
            });
        self.ids.borrow_mut().push(index);
        // The first completed atlas may come from nested canvas creation.
        // Its page can be newer than the outer page. Preserve negative deltas.
        let version = atlas.pages()[0].texture().version;
        let base = self.base_version.get().unwrap_or(version);
        self.base_version.set(Some(base));
        self.versions
            .borrow_mut()
            .push(i64::try_from(version).unwrap() - i64::try_from(base).unwrap());
        Ok(atlas)
    }
    pub fn remove(&self, owner: &Rc<Owner>) {
        self.pool.remove_terminal(&Rc::downgrade(owner)).unwrap();
    }
    pub fn snapshot(&self, atlas: &TextureAtlas) {
        self.pixels.borrow_mut().push(digest(&atlas.canvases()[0]));
        self.states
            .borrow_mut()
            .push(json!([atlas.pages().len(), atlas.page_layout_version()]));
    }
    pub fn draw(&self, atlas: &TextureAtlas) {
        atlas
            .get_glyph(GlyphKey::Number(65), 0, 0, 0, false, None)
            .unwrap();
    }
    pub fn output(&self, capture: Value) -> Value {
        json!({"ids":*self.ids.borrow(),"versions":*self.versions.borrow(),
            "states":*self.states.borrow(),"pixels":*self.pixels.borrow(),
            "trace":capture["trace"],"removals":capture["removals"]})
    }
}
