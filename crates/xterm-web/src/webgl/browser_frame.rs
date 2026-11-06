// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
//! Actual canvas/model/GPU composition for a typed cell source. The enclosing
//! renderer supplies live measurements and cursor/blink context. Raw JS service
//! access, timer subscriptions and the public addon are separate boundaries.
use super::{
    canvas_context::CanvasContext, glyph_gpu::GlyphRenderer, layer_services::LayerServices,
    link_layer::LinkRenderLayer, model::RenderModel, rectangle_gpu::RectangleRenderer,
    renderer_types::UpdateContext, renderer_update::RendererState, texture_atlas::TextureAtlas,
};
use crate::{
    api::{RenderDimensions, Size},
    renderer::service_api::RedrawRequest,
};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, HtmlElement};
use xterm_core::{
    event::{Emitter, Event},
    lifecycle::DisposableStore,
};
#[path = "browser_frame_host.rs"]
mod host;
#[path = "browser_frame_lifecycle.rs"]
mod lifecycle;
pub use host::CellSource;

struct GpuRenderers {
    rectangles: RectangleRenderer,
    glyphs: GlyphRenderer,
}

/// This exclusive-borrow API composes the real incremental model, rasterizer
/// and GPU renderers. It does not expose JS object getters or support recursive
/// calls into the same frame owner. A raw browser facade must implement its own
/// source-phase/reentry contract rather than claiming these typed inputs do so.
pub struct BrowserFrame {
    services: Rc<LayerServices>,
    screen: HtmlElement,
    context: Rc<CanvasContext>,
    link: Rc<LinkRenderLayer>,
    dimensions: RenderDimensions,
    state: RendererState,
    gpu: Option<GpuRenderers>,
    atlas: Option<Rc<TextureAtlas>>,
    atlas_listeners: DisposableStore,
    attached: bool,
    custom_glyphs: bool,
    max_texture_size: f64,
    changed: Emitter<HtmlCanvasElement>,
    added: Rc<Emitter<HtmlCanvasElement>>,
    removed: Rc<Emitter<HtmlCanvasElement>>,
    redraw: Emitter<RedrawRequest>,
    disposed: bool,
}
impl BrowserFrame {
    pub fn new(
        services: Rc<LayerServices>,
        screen: HtmlElement,
        dimensions: RenderDimensions,
        custom_glyphs: bool,
        preserve_drawing_buffer: Option<bool>,
    ) -> Result<Self, JsValue> {
        // A failed context must precede any layer subscription or DOM mutation.
        let context = Rc::new(CanvasContext::new(
            &services.browser.main_document(),
            preserve_drawing_buffer,
        )?);
        let link = LinkRenderLayer::new(services.clone(), screen.clone(), 2)?;
        let max_texture_size = context
            .gl()
            .get_parameter(web_sys::WebGl2RenderingContext::MAX_TEXTURE_SIZE)?
            .as_f64()
            .ok_or_else(|| js_sys::TypeError::new("WebGL texture size is not a number"))?;
        let mut this = Self {
            services,
            screen,
            context,
            link,
            dimensions,
            state: RendererState::default(),
            gpu: None,
            atlas: None,
            atlas_listeners: DisposableStore::new(),
            attached: false,
            custom_glyphs,
            max_texture_size,
            changed: Emitter::new(),
            added: Rc::new(Emitter::new()),
            removed: Rc::new(Emitter::new()),
            redraw: Emitter::new(),
            disposed: false,
        };
        this.context.attach(&this.screen)?;
        this.initialize_gpu()?;
        this.attached = this.screen.is_connected();
        Ok(this)
    }
    pub fn context(&self) -> &Rc<CanvasContext> {
        &self.context
    }
    pub fn link_layer(&self) -> &Rc<LinkRenderLayer> {
        &self.link
    }
    pub fn dimensions(&self) -> RenderDimensions {
        self.dimensions
    }
    pub fn model(&self) -> &RenderModel {
        &self.state.model
    }
    pub fn selection_changed(
        &mut self,
        selection: crate::renderer::row_types::Selection,
        viewport_y: usize,
    ) -> Result<(), JsValue> {
        self.require_live()?;
        let rows = self
            .services
            .terminal
            .rows()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        self.state.model.selection = crate::renderer::selection_model::SelectionRenderModel::update(
            rows, viewport_y, selection,
        );
        self.request_redraw(false)
    }
    pub fn atlas(&self) -> Option<Rc<TextureAtlas>> {
        self.atlas.clone()
    }
    pub fn texture_atlas(&self) -> Option<HtmlCanvasElement> {
        self.atlas
            .as_ref()
            .and_then(|atlas| atlas.canvases().first().cloned())
    }
    pub fn on_change_texture_atlas(&self) -> Event<HtmlCanvasElement> {
        self.changed.event()
    }
    pub fn on_add_texture_atlas_canvas(&self) -> Event<HtmlCanvasElement> {
        self.added.event()
    }
    pub fn on_remove_texture_atlas_canvas(&self) -> Event<HtmlCanvasElement> {
        self.removed.event()
    }
    pub fn on_request_redraw(&self) -> Event<RedrawRequest> {
        self.redraw.event()
    }

    /// Measurements are read by the enclosing renderer at its source call site.
    /// A detached/unmeasured screen must not mutate either model or GPU data.
    pub fn render_rows<S: CellSource>(
        &mut self,
        start: i64,
        end: i64,
        context: UpdateContext,
        measured: Size,
        source: &mut S,
    ) -> Result<(), JsValue> {
        self.require_live()?;
        if !self.attached {
            if !self.screen.is_connected() || measured.width == 0.0 || measured.height == 0.0 {
                return Ok(());
            }
            self.refresh_atlas()?;
            self.attached = true;
        }
        let gpu = self
            .gpu
            .as_mut()
            .ok_or_else(|| error("WebGL GPU renderers are unavailable"))?;
        let mut host = host::GpuHost {
            gpu,
            source,
            has_atlas: self.atlas.is_some(),
        };
        self.state
            .render_rows(start, end, context, &mut host)
            .map_err(|e| match e {
                super::renderer_types::UpdateError::Host(error) => error,
                super::renderer_types::UpdateError::InvalidGrid => error(
                    "WebGL update requires a resized, nonempty grid and nonnegative cell widths",
                ),
                super::renderer_types::UpdateError::InvalidJoin(range) => error(&format!(
                    "WebGL join range {range:?} is outside the grid or does not progress",
                )),
            })
    }
    pub fn resize(&mut self, dimensions: RenderDimensions) -> Result<(), JsValue> {
        self.require_live()?;
        self.dimensions = dimensions;
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
        self.state
            .resize(cols, rows)
            .map_err(|e| error(&e.to_string()))?;
        self.link.resize(dimensions)?;
        // Always invoke both setters. Assigning even unchanged canvas dimensions
        // clears its backing store, unlike the observer's physical resize path.
        let canvas = self.context.canvas();
        super::canvas_context::set(
            canvas.as_ref(),
            "width",
            &dimensions.device.canvas.width.into(),
        )?;
        super::canvas_context::set(
            canvas.as_ref(),
            "height",
            &dimensions.device.canvas.height.into(),
        )?;
        for element in [canvas.unchecked_ref::<HtmlElement>(), &self.screen] {
            element.style().set_property(
                "width",
                &format!(
                    "{}px",
                    ryu_js::Buffer::new().format(dimensions.css.canvas.width)
                ),
            )?;
            element.style().set_property(
                "height",
                &format!(
                    "{}px",
                    ryu_js::Buffer::new().format(dimensions.css.canvas.height)
                ),
            )?;
        }
        if let Some(gpu) = &mut self.gpu {
            gpu.rectangles.set_dimensions(dimensions);
            gpu.rectangles.handle_resize(cols, rows);
            gpu.glyphs.set_dimensions(dimensions);
            gpu.glyphs.handle_resize(cols, rows)?;
        }
        self.refresh_atlas()?;
        self.state.clear_model();
        self.request_redraw(true)
    }
    fn clear_model(&mut self) {
        self.state.clear_model();
        if let Some(gpu) = &mut self.gpu {
            gpu.glyphs.clear();
        }
    }
    pub fn clear(&mut self) -> Result<(), JsValue> {
        self.require_live()?;
        self.clear_model();
        self.link.reset()?;
        self.state.reset_blinking_rows();
        Ok(())
    }
    pub fn clear_texture_atlas(&mut self) -> Result<(), JsValue> {
        self.require_live()?;
        if let Some(atlas) = &self.atlas {
            atlas.clear_texture()?;
        }
        self.clear_model();
        self.request_redraw(false)
    }
    pub fn colors_changed(&mut self) -> Result<(), JsValue> {
        self.require_live()?;
        self.refresh_atlas()?;
        self.clear_model();
        Ok(())
    }
    fn require_live(&self) -> Result<(), JsValue> {
        if self.disposed {
            Err(error("WebGL frame owner is disposed"))
        } else {
            Ok(())
        }
    }
}
fn error(message: &str) -> JsValue {
    js_sys::Error::new(message).into()
}
