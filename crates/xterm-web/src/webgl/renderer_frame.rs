// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
//! WebglRenderer.renderRows orchestration. This does not install a browser addon.
use super::{
    model::RenderModel,
    renderer_types::{UpdateContext, UpdateError, UpdateHost},
    renderer_update::RendererState,
};

const MERGE_RETRY_LIMIT: usize = 32;

/// Actual GPU implementations can be composed with the update host. Attachment,
/// layer notification, atlas invalidation and draw calls keep their source order.
pub trait FrameHost: UpdateHost {
    /// Refresh dimensions/atlas when connected with measured characters. Return
    /// false when a detached or unmeasured terminal cannot yet draw.
    fn ensure_attached(&mut self) -> Result<bool, Self::Error>;
    fn grid_changed(&mut self, start: i64, end: i64) -> Result<(), Self::Error>;
    fn renderers_ready(&self) -> bool;
    fn begin_frame(&mut self) -> Result<bool, Self::Error>;
    fn has_atlas(&self) -> bool;
    fn clear_glyphs(&mut self) -> Result<(), Self::Error>;
    fn invalidate_atlas_textures(&mut self) -> Result<(), Self::Error>;
    fn render_backgrounds(&mut self) -> Result<(), Self::Error>;
    fn render_glyphs(&mut self, model: &RenderModel) -> Result<(), Self::Error>;
    fn cursor_visible(&self) -> bool;
    fn render_cursor(&mut self) -> Result<(), Self::Error>;
}
impl RendererState {
    pub fn render_rows<H: FrameHost>(
        &mut self,
        start: i64,
        end: i64,
        context: UpdateContext,
        host: &mut H,
    ) -> Result<(), UpdateError<H::Error>> {
        if !host.ensure_attached().map_err(UpdateError::Host)? {
            return Ok(());
        }
        host.grid_changed(start, end).map_err(UpdateError::Host)?;
        if !host.renderers_ready() {
            return Ok(());
        }
        let last_row = context
            .rows
            .checked_sub(1)
            .and_then(|r| i64::try_from(r).ok())
            .ok_or(UpdateError::InvalidGrid)?;
        if host.begin_frame().map_err(UpdateError::Host)? {
            self.clear_model();
            host.clear_glyphs().map_err(UpdateError::Host)?;
            self.update(0, last_row, context, host)?;
        } else {
            self.update(start, end, context, host)?;
        }
        let mut merged = false;
        let mut retries = 0;
        // Upstream checks beginFrame before its post-increment limit test. The
        // final probe still runs after 32 retries, even when it returns true.
        while host.has_atlas() && host.begin_frame().map_err(UpdateError::Host)? {
            if retries == MERGE_RETRY_LIMIT {
                break;
            }
            retries += 1;
            merged = true;
            self.clear_model();
            host.clear_glyphs().map_err(UpdateError::Host)?;
            self.update(0, last_row, context, host)?;
        }
        if merged {
            host.invalidate_atlas_textures()
                .map_err(UpdateError::Host)?;
        }
        host.render_backgrounds().map_err(UpdateError::Host)?;
        host.render_glyphs(&self.model).map_err(UpdateError::Host)?;
        if host.cursor_visible() {
            host.render_cursor().map_err(UpdateError::Host)?;
        }
        Ok(())
    }
}
