// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Browser atlas contract consumed by GlyphRenderer.
use super::glyphs::{GlyphCell, RasterizedGlyph};
use wasm_bindgen::JsValue;
use web_sys::{HtmlCanvasElement, HtmlElement};

/// Page versions must be globally monotonic across page object replacement.
/// Repacking glyph positions also increments the atlas layout version.
#[derive(Clone)]
pub struct AtlasPage {
    pub canvas: HtmlCanvasElement,
    pub version: u64,
}

/// The rasterizer owns its canvases/cache. The GPU renderer keeps the atlas alive
/// until replacement or disposal; no callbacks execute while it borrows geometry.
/// This is the GlyphRenderer dependency contract, not a TextureAtlas implementation.
pub trait GlyphAtlas {
    fn page_layout_version(&self) -> u64;
    fn page_count(&self) -> usize;
    fn page(&self, index: usize) -> Option<AtlasPage>;
    fn rasterize(
        &self,
        cell: GlyphCell<'_>,
        combined: bool,
        element: Option<&HtmlElement>,
    ) -> Result<RasterizedGlyph, JsValue>;
}
