// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! TextureAtlas's current/fixed-row packing, including its exact edge tests.
use super::{
    atlas_canvas::GlyphBitmap,
    atlas_page::{AtlasRow, CanvasAtlasPage, GlyphRef, RowRef, StoredGlyph},
    texture_atlas::{TextureAtlas, TEXTURE_SIZE},
};
use crate::js_error::to_js_error;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsValue;

const ROW_PIXEL_THRESHOLD: f64 = 2.0;
impl TextureAtlas {
    pub(super) fn pack(&self, bitmap: GlyphBitmap) -> Result<GlyphRef, JsValue> {
        let [width, height] = bitmap.bounds.size;
        let (page, row) = self.find_row(width, height)?;
        let index = self
            .pages
            .borrow()
            .iter()
            .position(|p| Rc::ptr_eq(p, &page))
            .ok_or_else(|| js_sys::Error::new("Atlas page removed before glyph placement"))?;
        let mut glyph = StoredGlyph::new(bitmap.bounds);
        let position = {
            let row = row.borrow();
            [row.x, row.y]
        };
        glyph.place(
            index,
            position,
            page.canvas.width().into(),
            page.canvas.height().into(),
        );
        {
            let mut row = row.borrow_mut();
            row.height = row.height.max(height);
            row.x += width;
        }
        let context = super::atlas_context::Context(&page.context);
        let upload = context.method("putImageData")?;
        // Source reads its shared work box after page events and callee lookup.
        // Nested rasterization can change the crop, but not the outer glyph size.
        let [left, top] = self.rasterizer.upload_origin();
        context.apply(
            &upload,
            &[
                bitmap.image.into(),
                (position[0] - left).into(),
                (position[1] - top).into(),
                left.into(),
                top.into(),
                width.into(),
                height.into(),
            ],
        )?;
        let glyph = Rc::new(RefCell::new(glyph));
        page.add_glyph(glyph.clone())?;
        page.touch(&self.runtime.clock)?;
        Ok(glyph)
    }
    fn find_row(&self, width: f64, height: f64) -> Result<(Rc<CanvasAtlasPage>, RowRef), JsValue> {
        loop {
            let active = self.active.borrow().clone();
            let Some(mut page) = active.last().cloned() else {
                let page = self.create_page()?;
                let row = page.current_row();
                row.borrow_mut().height = height;
                return Ok((page, row));
            };
            let mut row = page.current_row();
            for p in &active {
                if height <= p.current_row().borrow().height {
                    page = p.clone();
                    row = p.current_row();
                }
            }
            for p in active.iter().rev() {
                for fixed in p.fixed.borrow().iter() {
                    if fixed.borrow().height <= row.borrow().height
                        && height <= fixed.borrow().height
                    {
                        page = p.clone();
                        row = fixed.clone();
                    }
                }
            }
            if width > f64::from(TEXTURE_SIZE) {
                let overflow = self.overflow.borrow().clone();
                let overflow = match overflow {
                    Some(page) => page,
                    None => {
                        if self
                            .page_limit()
                            .is_some_and(|max| self.pages.borrow().len() >= max)
                        {
                            self.evict_pages()?;
                        }
                        let size = self.config.dimensions.device_max_texture_size as u32;
                        let page =
                            CanvasAtlasPage::new(&self.document, size, &self.runtime.clock, &[])?;
                        *self.overflow.borrow_mut() = Some(page.clone());
                        self.pages.borrow_mut().push(page.clone());
                        self.invalidate_layout();
                        self.added.fire(&page.canvas).map_err(|e| to_js_error(&e))?;
                        page
                    }
                };
                let row = overflow.current_row();
                {
                    let mut r = row.borrow_mut();
                    if r.x + width >= f64::from(overflow.canvas.width()) {
                        r.x = 0.0;
                        r.y += r.height;
                        r.height = 0.0;
                    }
                }
                return Ok((overflow, row));
            }
            let needs_row = {
                let r = row.borrow();
                r.y + height >= f64::from(page.canvas.height())
                    || r.height > height + ROW_PIXEL_THRESHOLD
            };
            if needs_row {
                let mut found = false;
                let current = *page.current_row().borrow();
                if current.y + current.height + height >= f64::from(page.canvas.height()) {
                    let candidate = self
                        .active
                        .borrow()
                        .iter()
                        .find(|p| {
                            let r = *p.current_row().borrow();
                            r.y + r.height + height < f64::from(p.canvas.height())
                        })
                        .cloned();
                    if let Some(candidate) = candidate {
                        page = candidate;
                    } else {
                        let sufficient = {
                            let r = row.borrow();
                            self.page_limit()
                                .is_some_and(|max| self.pages.borrow().len() >= max)
                                && r.y + height <= f64::from(page.canvas.height())
                                && r.height >= height
                                && r.x + width <= f64::from(page.canvas.width())
                        };
                        if sufficient {
                            found = true;
                        } else {
                            page = self.create_page()?;
                            row = page.current_row();
                            row.borrow_mut().height = height;
                            found = true;
                        }
                    }
                }
                if !found {
                    let current = page.current_row();
                    let previous = *current.borrow();
                    if previous.height > 0.0 {
                        page.fixed.borrow_mut().push(current);
                    }
                    row = Rc::new(RefCell::new(AtlasRow {
                        x: 0.0,
                        y: previous.y + previous.height,
                        height,
                    }));
                    page.fixed.borrow_mut().push(row.clone());
                    *page.current.borrow_mut() = Rc::new(RefCell::new(AtlasRow {
                        x: 0.0,
                        y: row.borrow().y + height,
                        height: 0.0,
                    }));
                }
            }
            if row.borrow().x + width <= f64::from(page.canvas.width()) {
                return Ok((page, row));
            }
            if Rc::ptr_eq(&row, &page.current_row()) {
                let mut row = row.borrow_mut();
                row.x = 0.0;
                row.y += row.height;
                row.height = 0.0;
            } else {
                let index = page.fixed.borrow().iter().position(|r| Rc::ptr_eq(r, &row));
                if let Some(index) = index {
                    page.fixed.borrow_mut().remove(index);
                }
            }
        }
    }
}
