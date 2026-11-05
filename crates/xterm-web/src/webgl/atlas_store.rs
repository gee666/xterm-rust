// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Page merge, eviction and shared glyph-coordinate invalidation.
use super::{
    atlas_page::CanvasAtlasPage,
    texture_atlas::{TextureAtlas, FORCED_MAX_TEXTURE_SIZE, TEXTURE_SIZE},
};
use crate::js_error::to_js_error;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use xterm_core::log_service::LogLevel;

impl TextureAtlas {
    pub(super) fn page_limit(&self) -> Option<usize> {
        self.runtime.max_pages.get().filter(|n| *n != 0)
    }
    pub(super) fn create_page(&self) -> Result<Rc<CanvasAtlasPage>, JsValue> {
        if self
            .page_limit()
            .is_some_and(|max| self.pages.borrow().len() >= max.max(4))
        {
            let max_size = self
                .runtime
                .max_texture_size
                .get()
                .filter(|n| *n != 0)
                .unwrap_or(FORCED_MAX_TEXTURE_SIZE);
            let mut candidates: Vec<_> = self
                .pages
                .borrow()
                .iter()
                .filter(|p| p.canvas.width().saturating_mul(2) <= max_size)
                .cloned()
                .collect();
            // Stable sort retains source page order for equally full pages.
            candidates.sort_by(|a, b| {
                b.canvas
                    .width()
                    .cmp(&a.canvas.width())
                    .then_with(|| b.percentage_used().total_cmp(&a.percentage_used()))
            });
            let mut start = None;
            let mut size = 0;
            for (i, page) in candidates.iter().enumerate() {
                if page.canvas.width() != size {
                    start = Some(i);
                    size = page.canvas.width();
                } else if start.is_some_and(|start| i - start == 3) {
                    break;
                }
            }
            let merging: Vec<_> = start
                .map(|i| candidates[i..candidates.len().min(i + 4)].to_vec())
                .unwrap_or_default();
            if merging.len() < 4
                || merging
                    .iter()
                    .any(|p| p.canvas.width() != merging[0].canvas.width())
            {
                self.evict_pages()?;
                return self.append_active_page();
            }
            let mut indices = Vec::with_capacity(4);
            for page in &merging {
                let glyph = page.glyphs.borrow().first().cloned().ok_or_else(|| {
                    js_sys::TypeError::new("Cannot read texturePage of undefined")
                })?;
                indices.push(glyph.borrow().raster.texture_page);
            }
            indices.sort_unstable();
            let merged_index = self.pages.borrow().len() - merging.len();
            let merged = self.merge_pages(&merging, merged_index)?;
            merged.touch(&self.runtime.clock)?;
            for index in indices.into_iter().rev() {
                self.delete_page(index)?;
            }
            self.pages.borrow_mut().push(merged.clone());
            self.invalidate_layout();
            self.added
                .fire(&merged.canvas)
                .map_err(|e| to_js_error(&e))?;
        }
        self.append_active_page()
    }
    fn append_active_page(&self) -> Result<Rc<CanvasAtlasPage>, JsValue> {
        let page = CanvasAtlasPage::new(&self.document, TEXTURE_SIZE, &self.runtime.clock, &[])?;
        self.pages.borrow_mut().push(page.clone());
        self.active.borrow_mut().push(page.clone());
        self.added.fire(&page.canvas).map_err(|e| to_js_error(&e))?;
        Ok(page)
    }
    fn merge_pages(
        &self,
        pages: &[Rc<CanvasAtlasPage>],
        index: usize,
    ) -> Result<Rc<CanvasAtlasPage>, JsValue> {
        let size = pages[0].canvas.width() * 2;
        let merged = CanvasAtlasPage::new(&self.document, size, &self.runtime.clock, pages)?;
        for (i, page) in pages.iter().enumerate() {
            let x = (i as u32 * page.canvas.width()) % size;
            let y = (i as u32 / 2) * page.canvas.height();
            merged
                .context
                .draw_image_with_html_canvas_element(&page.canvas, x.into(), y.into())?;
            let mut j = 0;
            loop {
                let glyph = page.glyphs.borrow().get(j).cloned();
                let Some(glyph) = glyph else { break };
                let mut glyph = glyph.borrow_mut();
                let pos = [
                    glyph.position[0] + f64::from(x),
                    glyph.position[1] + f64::from(y),
                ];
                glyph.place(index, pos, size.into(), size.into());
                j += 1;
            }
            self.removed
                .fire(&page.canvas)
                .map_err(|e| to_js_error(&e))?;
            let at = self
                .active
                .borrow()
                .iter()
                .position(|p| Rc::ptr_eq(p, page));
            if let Some(at) = at {
                self.active.borrow_mut().remove(at);
            }
        }
        Ok(merged)
    }
    fn delete_page(&self, index: usize) -> Result<(), JsValue> {
        if index < self.pages.borrow().len() {
            self.pages.borrow_mut().remove(index);
        }
        let mut j = index;
        loop {
            let page = self.pages.borrow().get(j).cloned();
            let Some(page) = page else { break };
            for glyph in page.glyphs.borrow().iter() {
                let mut glyph = glyph.borrow_mut();
                glyph.raster.texture_page = glyph
                    .raster
                    .texture_page
                    .checked_sub(1)
                    .ok_or_else(|| js_sys::Error::new("Atlas glyph page index underflow"))?;
            }
            page.touch(&self.runtime.clock)?;
            j += 1;
        }
        Ok(())
    }
    pub(super) fn evict_pages(&self) -> Result<(), JsValue> {
        let performance = web_sys::window()
            .and_then(|w| w.performance())
            .ok_or_else(|| js_sys::Error::new("Atlas eviction requires performance.now"))?;
        let start = performance.now();
        let count = self.pages.borrow().len();
        let mut i = 0;
        loop {
            let page = self.pages.borrow().get(i).cloned();
            let Some(page) = page else { break };
            self.removed
                .fire(&page.canvas)
                .map_err(|e| to_js_error(&e))?;
            page.canvas.remove();
            i += 1;
        }
        self.pages.borrow_mut().clear();
        self.active.borrow_mut().clear();
        self.overflow.borrow_mut().take();
        self.cache.borrow_mut().clear();
        self.warmed.set(false);
        self.invalidate_layout();
        let message = format!(
            "Evicted {count} WebGL atlas pages in {:.2}ms",
            performance.now() - start
        );
        self.log
            .log(
                LogLevel::Debug,
                &message.encode_utf16().collect::<Vec<_>>(),
                Vec::new(),
            )
            .map_err(|e| to_js_error(&e))
    }
}
