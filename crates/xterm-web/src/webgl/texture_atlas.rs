// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! TextureAtlas owns text rasterization, cached glyph aliases and real canvas pages.
use super::{
    atlas_colors::AtlasColors,
    atlas_page::{CanvasAtlasPage, GlyphRef, PageClock, StoredGlyph},
    atlas_rasterizer::Rasterizer,
    cache::AtlasResource,
    config::{AtlasConfig, ConfigFactory},
    glyph_atlas::{AtlasPage, GlyphAtlas},
    glyphs::{GlyphCell, RasterizedGlyph},
};
use crate::task_queue::BrowserTaskQueue;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};
use wasm_bindgen::JsValue;
use web_sys::{Document, HtmlCanvasElement, HtmlElement};
use xterm_core::{
    event::{Emitter, Event},
    lifecycle::Disposable,
    log_service::LogService,
    unicode_service::UnicodeService,
};

pub(super) const TEXTURE_SIZE: u32 = 512;
pub(super) const FORCED_MAX_TEXTURE_SIZE: u32 = 4096;

/// Module-shared limits and upload clock belong to one browser binding runtime.
/// Every atlas in its AtlasCache must receive the same instance.
pub struct AtlasRuntime {
    pub config: ConfigFactory,
    pub clock: PageClock,
    pub max_pages: Cell<Option<usize>>,
    pub max_texture_size: Cell<Option<u32>>,
    null_glyph: GlyphRef,
}
impl Default for AtlasRuntime {
    fn default() -> Self {
        Self {
            config: ConfigFactory::new(),
            clock: PageClock::default(),
            max_pages: Cell::new(None),
            max_texture_size: Cell::new(None),
            null_glyph: Rc::new(RefCell::new(StoredGlyph {
                raster: RasterizedGlyph::default(),
                position: [0.0, 0.0],
            })),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GlyphKey {
    Number(i32),
    Text(Vec<u16>),
}
type Cache = HashMap<(GlyphKey, u32, u32, u32), GlyphRef>;

pub struct TextureAtlas {
    pub(super) document: Document,
    pub(super) config: Rc<AtlasConfig>,
    pub(super) runtime: Rc<AtlasRuntime>,
    pub(super) log: Rc<LogService>,
    pub(super) pages: RefCell<Vec<Rc<CanvasAtlasPage>>>,
    pub(super) active: RefCell<Vec<Rc<CanvasAtlasPage>>>,
    pub(super) overflow: RefCell<Option<Rc<CanvasAtlasPage>>>,
    pub(super) cache: RefCell<Cache>,
    pub(super) warmed: Cell<bool>,
    pub(super) layout_version: Cell<u64>,
    pub(super) added: Emitter<HtmlCanvasElement>,
    pub(super) removed: Emitter<HtmlCanvasElement>,
    pub(super) rasterizer: Rasterizer,
    warmups: RefCell<Vec<BrowserTaskQueue>>,
}
impl TextureAtlas {
    pub fn new(
        document: &Document,
        config: Rc<AtlasConfig>,
        runtime: Rc<AtlasRuntime>,
        unicode: Rc<UnicodeService>,
        log: Rc<LogService>,
    ) -> Result<Rc<Self>, JsValue> {
        // Source creates its first page before the temporary drawing canvas.
        let first = CanvasAtlasPage::new(document, TEXTURE_SIZE, &runtime.clock, &[])?;
        let colors = AtlasColors::new(config.clone(), &runtime.config);
        let rasterizer = Rasterizer::new(document, config.clone(), colors, unicode, log.clone())?;
        Ok(Rc::new(Self {
            document: document.clone(),
            config,
            runtime,
            log,
            pages: RefCell::new(vec![first.clone()]),
            active: RefCell::new(vec![first]),
            overflow: RefCell::new(None),
            cache: RefCell::new(HashMap::new()),
            warmed: Cell::new(false),
            layout_version: Cell::new(0),
            added: Emitter::new(),
            removed: Emitter::new(),
            rasterizer,
            warmups: RefCell::new(Vec::new()),
        }))
    }
    pub fn on_add_canvas(&self) -> Event<HtmlCanvasElement> {
        self.added.event()
    }
    pub fn on_remove_canvas(&self) -> Event<HtmlCanvasElement> {
        self.removed.event()
    }
    pub fn pages(&self) -> Vec<Rc<CanvasAtlasPage>> {
        self.pages.borrow().clone()
    }
    pub fn canvases(&self) -> Vec<HtmlCanvasElement> {
        self.pages
            .borrow()
            .iter()
            .map(|p| p.canvas.clone())
            .collect()
    }
    /// Returns the actual cached alias. Page merges update already returned glyphs.
    /// Height restriction and DOM container deliberately do not participate in keys.
    pub fn get_glyph(
        &self,
        key: GlyphKey,
        bg: u32,
        fg: u32,
        ext: u32,
        restrict_height: bool,
        element: Option<&HtmlElement>,
    ) -> Result<GlyphRef, JsValue> {
        let cache_key = (key, bg, fg, ext);
        let cached = self.cache.borrow().get(&cache_key).cloned();
        if let Some(glyph) = cached {
            return Ok(glyph);
        }
        let bitmap = self
            .rasterizer
            .draw(&cache_key.0, bg, fg, ext, restrict_height, element)?;
        let glyph = if let Some(bitmap) = bitmap {
            self.pack(bitmap)?
        } else {
            self.runtime.null_glyph.clone()
        };
        self.cache.borrow_mut().insert(cache_key, glyph.clone());
        Ok(glyph)
    }
    pub fn warm_up(self: &Rc<Self>) -> Result<(), JsValue> {
        if self.warmed.get() {
            return Ok(());
        }
        self.warmups
            .borrow_mut()
            .retain(|q| q.remaining_tasks() != 0);
        let queue = BrowserTaskQueue::source_idle_with_logger(self.log.clone())?;
        self.warmups.borrow_mut().push(queue.clone());
        for code in 33..126 {
            let atlas = Rc::downgrade(self);
            queue.enqueue_try(move |_| {
                if let Some(atlas) = atlas.upgrade() {
                    atlas.get_glyph(GlyphKey::Number(code), 0, 0, 0, false, None)?;
                }
                Ok(false)
            })?;
        }
        self.warmed.set(true);
        Ok(())
    }
    pub fn clear_texture(&self) -> Result<(), JsValue> {
        let first = self.pages.borrow().first().cloned();
        if let Some(page) = first {
            let row = *page.current_row().borrow();
            if row.x == 0.0 && row.y == 0.0 {
                return Ok(());
            }
        }
        let mut i = 0;
        loop {
            let page = self.pages.borrow().get(i).cloned();
            let Some(page) = page else { break };
            page.clear(&self.runtime.clock)?;
            i += 1;
        }
        self.cache.borrow_mut().clear();
        self.warmed.set(false);
        self.invalidate_layout();
        Ok(())
    }
    pub(super) fn invalidate_layout(&self) {
        self.layout_version.set(self.layout_version.get() + 1);
    }
}
impl GlyphAtlas for TextureAtlas {
    fn page_layout_version(&self) -> u64 {
        self.layout_version.get()
    }
    fn page_count(&self) -> usize {
        self.pages.borrow().len()
    }
    fn page(&self, index: usize) -> Option<AtlasPage> {
        let page = self.pages.borrow().get(index).cloned();
        page.map(|p| p.texture())
    }
    fn rasterize(
        &self,
        cell: GlyphCell<'_>,
        combined: bool,
        element: Option<&HtmlElement>,
    ) -> Result<RasterizedGlyph, JsValue> {
        let key = if combined {
            GlyphKey::Text(cell.chars.to_vec())
        } else {
            GlyphKey::Number(cell.code.unwrap_or(0) as i32)
        };
        let glyph = self.get_glyph(key, cell.bg, cell.fg, cell.ext, false, element)?;
        let raster = glyph.borrow().raster;
        Ok(raster)
    }
}
impl AtlasResource for TextureAtlas {
    type Error = JsValue;
    fn dispose(&self) -> Result<(), JsValue> {
        self.rasterizer.remove_canvas()?;
        let mut i = 0;
        loop {
            let page = self.pages.borrow().get(i).cloned();
            let Some(page) = page else { break };
            page.canvas.remove();
            i += 1;
        }
        self.added.dispose();
        self.removed.dispose();
        Ok(())
    }
}
