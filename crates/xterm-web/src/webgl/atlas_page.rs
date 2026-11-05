// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// TextureAtlas.ts AtlasPage state. The owning atlas controls DOM removal/events.
use super::{
    atlas_pixels::GlyphBounds, glyph_atlas::AtlasPage as PageTexture, glyphs::RasterizedGlyph,
    gpu_resources::error,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement};

/// One explicitly-owned clock is shared by every atlas in a binding runtime.
/// This preserves globally-monotonic page uploads without mutable core globals.
#[derive(Default)]
pub struct PageClock(Cell<u64>);
impl PageClock {
    pub fn allocate(&self) -> Result<u64, JsValue> {
        let value = self
            .0
            .get()
            .checked_add(1)
            .ok_or_else(|| error("Atlas page version exhausted"))?;
        self.0.set(value);
        Ok(value)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AtlasRow {
    pub x: f64,
    pub y: f64,
    pub height: f64,
}
pub type RowRef = Rc<RefCell<AtlasRow>>;
pub type GlyphRef = Rc<RefCell<StoredGlyph>>;
pub struct StoredGlyph {
    pub raster: RasterizedGlyph,
    pub position: [f64; 2],
}
impl StoredGlyph {
    pub fn new(bounds: GlyphBounds) -> Self {
        Self {
            raster: RasterizedGlyph {
                size: bounds.size,
                offset: bounds.offset,
                ..Default::default()
            },
            position: [0.0, 0.0],
        }
    }
    pub fn place(&mut self, page: usize, position: [f64; 2], width: f64, height: f64) {
        self.position = position;
        self.raster.texture_page = page;
        self.raster.texture_position = [position[0] / width, position[1] / height];
        self.raster.texture_size = [self.raster.size[0] / width, self.raster.size[1] / height];
    }
}

pub struct CanvasAtlasPage {
    pub canvas: HtmlCanvasElement,
    pub context: CanvasRenderingContext2d,
    pub current: RefCell<RowRef>,
    pub fixed: RefCell<Vec<RowRef>>,
    pub glyphs: RefCell<Vec<GlyphRef>>,
    used: Cell<f64>,
    version: Cell<u64>,
}
impl CanvasAtlasPage {
    pub fn new(
        document: &Document,
        size: u32,
        clock: &PageClock,
        sources: &[Rc<Self>],
    ) -> Result<Rc<Self>, JsValue> {
        let version = clock.allocate()?;
        // The DOM tag fixes the interface. A main-realm instanceof check would
        // reject an otherwise valid canvas created by an iframe document.
        let canvas: HtmlCanvasElement = document.create_element("canvas")?.unchecked_into();
        canvas.set_width(size);
        canvas.set_height(size);
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"alpha".into(), &true.into())?;
        let context = canvas
            .get_context_with_context_options("2d", &options)?
            .ok_or_else(|| error("value must not be falsy"))?
            .unchecked_into();
        let mut glyphs = Vec::new();
        let mut used = 0.0;
        for source in sources {
            let incoming = source.glyphs.borrow();
            glyphs
                .try_reserve(incoming.len())
                .map_err(|e| error(&e.to_string()))?;
            glyphs.extend(incoming.iter().cloned());
            used += source.used.get();
        }
        Ok(Rc::new(Self {
            canvas,
            context,
            current: RefCell::new(Rc::new(RefCell::new(AtlasRow::default()))),
            fixed: RefCell::new(Vec::new()),
            glyphs: RefCell::new(glyphs),
            used: Cell::new(used),
            version: Cell::new(version),
        }))
    }
    pub fn current_row(&self) -> RowRef {
        Rc::clone(&self.current.borrow())
    }
    pub fn percentage_used(&self) -> f64 {
        self.used.get() / (f64::from(self.canvas.width()) * f64::from(self.canvas.height()))
    }
    pub fn add_glyph(&self, glyph: GlyphRef) -> Result<(), JsValue> {
        self.glyphs
            .borrow_mut()
            .try_reserve(1)
            .map_err(|e| error(&e.to_string()))?;
        let area = {
            let glyph = glyph.borrow();
            glyph.raster.size[0] * glyph.raster.size[1]
        };
        self.glyphs.borrow_mut().push(glyph);
        self.used.set(self.used.get() + area);
        Ok(())
    }
    pub fn touch(&self, clock: &PageClock) -> Result<(), JsValue> {
        self.version.set(clock.allocate()?);
        Ok(())
    }
    pub fn texture(&self) -> PageTexture {
        PageTexture {
            canvas: self.canvas.clone(),
            version: self.version.get(),
        }
    }
    pub fn clear(&self, clock: &PageClock) -> Result<(), JsValue> {
        self.context.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        *self.current_row().borrow_mut() = AtlasRow::default();
        self.fixed.borrow_mut().clear();
        // Source retains historical glyph references and used area on clear.
        // Merges still update those references; resetting them changes aliasing.
        self.touch(clock)
    }
}
