// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use crate::webgl::{
    browser_cache::{AtlasRequest, AtlasServices},
    cache::CacheError,
    config::AtlasDimensions,
};
use xterm_core::lifecycle::Disposable;

impl BrowserFrame {
    pub(super) fn initialize_gpu(&mut self) -> Result<(), JsValue> {
        let cols = self
            .services
            .terminal
            .cols()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let rows = self
            .services
            .terminal
            .rows()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let runtime = self.services.cache.runtime();
        if runtime.max_pages.get().is_none() {
            // The source initializes module-wide limits from the first context.
            // Keep them in the existing binding cache, not in a second pool.
            const MAX_ATLAS_PAGES: usize = 32;
            let maximum = self
                .context
                .gl()
                .get_parameter(web_sys::WebGl2RenderingContext::MAX_TEXTURE_IMAGE_UNITS)?
                .as_f64()
                .filter(|n| n.is_finite() && *n >= 1.0)
                .ok_or_else(|| error("WebGL texture unit capacity is unavailable"))?;
            runtime
                .max_pages
                .set(Some(maximum.min(MAX_ATLAS_PAGES as f64) as usize));
            // MAX_TEXTURE_SIZE is a positive WebGL GLint on a real context.
            runtime
                .max_texture_size
                .set(Some(self.max_texture_size as u32));
        }
        // Construct each replacement before disposing the corresponding previous
        // resource, matching MutableDisposable assignment's evaluation order.
        let rectangles = RectangleRenderer::new(
            self.context.gl().clone(),
            cols,
            rows,
            self.dimensions,
            &self.services.theme,
        )?;
        if let Some(gpu) = &mut self.gpu {
            gpu.rectangles = rectangles;
            let glyphs = GlyphRenderer::new(
                self.context.gl().clone(),
                cols,
                rows,
                self.dimensions,
                Some(self.screen.clone()),
            )?;
            gpu.glyphs = glyphs;
        } else {
            let glyphs = GlyphRenderer::new(
                self.context.gl().clone(),
                cols,
                rows,
                self.dimensions,
                Some(self.screen.clone()),
            )?;
            self.gpu = Some(GpuRenderers { rectangles, glyphs });
        }
        self.resize(self.dimensions)
    }
    /// Restoration runs after the source-global timeout has been cancelled by
    /// ContextLifecycle. Release the terminal's cache ownership before replacing
    /// GPUs, then reacquire/rasterize before the enclosing restoration redraw.
    pub fn restore(&mut self) -> Result<(), JsValue> {
        self.require_live()?;
        self.release_atlas()?;
        self.initialize_gpu()
    }
    pub(super) fn refresh_atlas(&mut self) -> Result<(), JsValue> {
        let dim = self.dimensions.device;
        if dim.character.width <= 0.0 && dim.character.height <= 0.0 {
            self.attached = false;
            return Ok(());
        }
        let options = self.services.options()?;
        let colors = self.services.theme.colors().borrow().clone();
        let atlas = self
            .services
            .cache
            .acquire(
                &self.services.terminal,
                AtlasRequest {
                    dimensions: AtlasDimensions {
                        device_cell_width: dim.cell.width,
                        device_cell_height: dim.cell.height,
                        device_char_width: dim.character.width,
                        device_char_height: dim.character.height,
                        device_pixel_ratio: self.services.browser.dpr()?,
                        device_max_texture_size: self.max_texture_size,
                    },
                    options: &options,
                    colors: &colors,
                    custom_glyphs: self.custom_glyphs,
                },
                || {
                    self.services
                        .terminal
                        .with_state(|state| AtlasServices {
                            unicode: state.unicode.clone(),
                            log: state.logger.clone(),
                        })
                        .map_err(|e| crate::js_error::to_js_error(&e))
                },
            )
            .map_err(cache_error)?;
        if self
            .atlas
            .as_ref()
            .is_none_or(|old| !Rc::ptr_eq(old, &atlas))
        {
            let first = atlas
                .canvases()
                .first()
                .cloned()
                .ok_or_else(|| error("WebGL atlas has no first canvas"))?;
            self.changed
                .fire(&first)
                .map_err(|e| crate::js_error::to_js_error(&e))?;
            // Replacing the pair only after the change event preserves the
            // source's observable old-atlas forwarding during that event.
            self.atlas_listeners.clear();
            let added = self.added.clone();
            self.atlas_listeners.add(
                atlas
                    .on_add_canvas()
                    .try_subscribe(move |canvas| added.fire(canvas)),
            );
            let removed = self.removed.clone();
            self.atlas_listeners.add(
                atlas
                    .on_remove_canvas()
                    .try_subscribe(move |canvas| removed.fire(canvas)),
            );
        }
        self.atlas = Some(atlas.clone());
        atlas.warm_up()?;
        if let Some(gpu) = &mut self.gpu {
            gpu.glyphs.set_atlas(atlas);
        }
        Ok(())
    }
    fn release_atlas(&self) -> Result<(), JsValue> {
        self.services
            .cache
            .remove_terminal(&Rc::downgrade(&self.services.terminal))
            .map_err(cache_error)
    }
    pub(super) fn request_redraw(&self, sync: bool) -> Result<(), JsValue> {
        let rows = self
            .services
            .terminal
            .rows()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        self.redraw
            .fire(&RedrawRequest {
                start: 0.0,
                end: rows as f64 - 1.0,
                sync,
            })
            .map_err(|e| crate::js_error::to_js_error(&e))
    }
    /// Dispose GPUs/listeners, then link layer, canvas and terminal atlas owner.
    /// The enclosing renderer must first dispose its cursor/text/context timers.
    /// An explicit cleanup failure is reported and is never silently retried by
    /// Drop. Shared atlases survive while another terminal owns the entry.
    pub fn dispose(&mut self) -> Result<(), JsValue> {
        if self.disposed {
            return Ok(());
        }
        self.disposed = true;
        self.atlas_listeners.dispose();
        self.gpu.take();
        self.changed.dispose();
        self.added.dispose();
        self.removed.dispose();
        self.redraw.dispose();
        self.link.dispose()?;
        self.context.detach()?;
        self.release_atlas()
    }
}
impl Drop for BrowserFrame {
    fn drop(&mut self) {
        if !self.disposed {
            if let Err(error) = self.dispose() {
                crate::write_scheduler::report_unhandled_error(&error);
            }
        }
    }
}
fn cache_error(error: CacheError<JsValue>) -> JsValue {
    match error {
        CacheError::Config(error) => js_sys::TypeError::new(&error.to_string()).into(),
        CacheError::Create(error) | CacheError::Dispose(error) => error,
    }
}
