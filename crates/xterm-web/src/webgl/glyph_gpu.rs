// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-webgl/src/GlyphRenderer.ts.
use super::{
    glyph_atlas::GlyphAtlas,
    glyph_shaders,
    glyphs::{GlyphCell, GlyphGeometry, GLYPH_STRIDE},
    gpu_resources::{error, Buffer, Program, UnitQuad, VertexArray},
    math::PROJECTION_MATRIX,
    model::RenderModel,
};
use crate::api::RenderDimensions;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::{HtmlElement, WebGl2RenderingContext as Gl, WebGlTexture, WebGlUniformLocation};

struct Texture {
    gl: Gl,
    value: WebGlTexture,
    version: Option<u64>,
}
impl Texture {
    fn new(gl: &Gl, index: usize) -> Result<Self, JsValue> {
        let texture = Self {
            gl: gl.clone(),
            value: gl
                .create_texture()
                .ok_or_else(|| error("WebGL could not allocate an atlas texture"))?,
            version: None,
        };
        gl.active_texture(Gl::TEXTURE0 + index as u32);
        gl.bind_texture(Gl::TEXTURE_2D, Some(&texture.value));
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            Gl::TEXTURE_2D,
            0,
            Gl::RGBA as i32,
            1,
            1,
            0,
            Gl::RGBA,
            Gl::UNSIGNED_BYTE,
            Some(&[255, 0, 0, 255]),
        )?;
        Ok(texture)
    }
}
impl Drop for Texture {
    fn drop(&mut self) {
        self.gl.delete_texture(Some(&self.value));
    }
}
struct Gpu {
    gl: Gl,
    program: Program,
    resolution: WebGlUniformLocation,
    vao: VertexArray,
    attributes: Buffer,
    _quad: UnitQuad,
    textures: Vec<Texture>,
}
impl Gpu {
    fn new(gl: Gl) -> Result<Self, JsValue> {
        let maximum = gl
            .get_parameter(Gl::MAX_TEXTURE_IMAGE_UNITS)?
            .as_f64()
            .filter(|n| n.is_finite() && *n >= 1.0)
            .ok_or_else(|| error("WebGL texture unit capacity is unavailable"))?;
        let capacity = maximum.min(32.0) as usize;
        let program = Program::new(
            &gl,
            glyph_shaders::VERTEX,
            &glyph_shaders::fragment(capacity),
        )?;
        let uniform = |name| {
            gl.get_uniform_location(&program.value, name)
                .ok_or_else(|| error(&format!("WebGL {name} uniform was not found")))
        };
        let projection = uniform("u_projection")?;
        let resolution = uniform("u_resolution")?;
        let sampler = uniform("u_texture")?;
        let vao = VertexArray::new(&gl)?;
        gl.bind_vertex_array(Some(&vao.value));
        let quad = UnitQuad::new(&gl, 0)?;
        let attributes = Buffer::new(&gl)?;
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&attributes.value));
        for (location, size, offset) in [
            (2, 2, 0),
            (3, 2, 8),
            (4, 1, 16),
            (5, 2, 20),
            (6, 2, 28),
            (1, 2, 36),
        ] {
            gl.enable_vertex_attrib_array(location);
            gl.vertex_attrib_pointer_with_i32(
                location,
                size,
                Gl::FLOAT,
                false,
                (GLYPH_STRIDE * 4) as i32,
                offset,
            );
            gl.vertex_attrib_divisor(location, 1);
        }
        gl.use_program(Some(&program.value));
        gl.uniform1iv_with_i32_array(Some(&sampler), &(0..capacity as i32).collect::<Vec<_>>());
        gl.uniform_matrix4fv_with_f32_array(Some(&projection), false, &PROJECTION_MATRIX);
        let mut textures = Vec::with_capacity(capacity);
        for i in 0..capacity {
            textures.push(Texture::new(&gl, i)?);
        }
        gl.enable(Gl::BLEND);
        gl.blend_func(Gl::SRC_ALPHA, Gl::ONE_MINUS_SRC_ALPHA);
        Ok(Self {
            gl,
            program,
            resolution,
            vao,
            attributes,
            _quad: quad,
            textures,
        })
    }
    fn resize(&self, dimensions: RenderDimensions) -> Result<(), JsValue> {
        let canvas = dimensions.device.canvas;
        let width = pixel_dimension(canvas.width)?;
        let height = pixel_dimension(canvas.height)?;
        self.gl.use_program(Some(&self.program.value));
        self.gl.viewport(0, 0, width, height);
        self.gl.uniform2f(
            Some(&self.resolution),
            canvas.width as f32,
            canvas.height as f32,
        );
        Ok(())
    }
    fn draw(&mut self, attributes: &[f32], atlas: &dyn GlyphAtlas) -> Result<(), JsValue> {
        if self.gl.is_context_lost() {
            return Err(error("WebGL context is lost"));
        }
        let count = i32::try_from(attributes.len() / GLYPH_STRIDE)
            .map_err(|_| error("Too many WebGL glyphs"))?;
        let gl = &self.gl;
        gl.use_program(Some(&self.program.value));
        gl.bind_vertex_array(Some(&self.vao.value));
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&self.attributes.value));
        let values = js_sys::Float32Array::from(attributes);
        gl.buffer_data_with_array_buffer_view(Gl::ARRAY_BUFFER, &values, Gl::STREAM_DRAW);
        for (i, texture) in self
            .textures
            .iter_mut()
            .take(atlas.page_count())
            .enumerate()
        {
            let page = atlas
                .page(i)
                .ok_or_else(|| error("Atlas page disappeared during upload"))?;
            if texture.version == Some(page.version) {
                continue;
            }
            gl.active_texture(Gl::TEXTURE0 + i as u32);
            gl.bind_texture(Gl::TEXTURE_2D, Some(&texture.value));
            gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
            gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);
            gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::LINEAR as i32);
            gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::LINEAR as i32);
            gl.tex_image_2d_with_u32_and_u32_and_html_canvas_element(
                Gl::TEXTURE_2D,
                0,
                Gl::RGBA as i32,
                Gl::RGBA,
                Gl::UNSIGNED_BYTE,
                &page.canvas,
            )?;
            texture.version = Some(page.version);
        }
        gl.draw_elements_instanced_with_i32(Gl::TRIANGLE_STRIP, 4, Gl::UNSIGNED_BYTE, 0, count);
        Ok(())
    }
}
fn pixel_dimension(value: f64) -> Result<i32, JsValue> {
    if !value.is_finite() || value < 0.0 || value > i32::MAX as f64 {
        Err(error("WebGL canvas dimension exceeds supported range"))
    } else {
        Ok(value as i32)
    }
}

/// Real WebGL2 glyph drawing with a caller-owned rasterization atlas.
/// Full TextureAtlas rasterization and the public addon remain separate work.
pub struct GlyphRenderer {
    gpu: Option<Gpu>,
    geometry: GlyphGeometry,
    element: Option<HtmlElement>,
    atlas: Option<Rc<dyn GlyphAtlas>>,
    layout_version: Option<u64>,
    page_overflow_warned: bool,
}
impl GlyphRenderer {
    pub fn new(
        gl: Gl,
        cols: usize,
        rows: usize,
        dimensions: RenderDimensions,
        element: Option<HtmlElement>,
    ) -> Result<Self, JsValue> {
        let geometry =
            GlyphGeometry::new(cols, rows, dimensions).map_err(|e| error(&e.to_string()))?;
        let gpu = Gpu::new(gl)?;
        gpu.resize(dimensions)?;
        Ok(Self {
            gpu: Some(gpu),
            geometry,
            element,
            atlas: None,
            layout_version: None,
            page_overflow_warned: false,
        })
    }
    pub fn texture_capacity(&self) -> usize {
        self.gpu.as_ref().map_or(0, |gpu| gpu.textures.len())
    }
    pub fn set_atlas(&mut self, atlas: Rc<dyn GlyphAtlas>) {
        self.atlas = Some(atlas);
        self.layout_version = None;
        self.invalidate_atlas_textures();
    }
    pub fn invalidate_atlas_textures(&mut self) {
        if let Some(gpu) = &mut self.gpu {
            for texture in &mut gpu.textures {
                texture.version = None;
            }
        }
    }
    /// True means the full model must be rebuilt before the next draw.
    pub fn begin_frame(&mut self) -> bool {
        let Some(atlas) = &self.atlas else {
            return true;
        };
        let version = atlas.page_layout_version();
        if self.layout_version != Some(version) {
            self.layout_version = Some(version);
            true
        } else {
            false
        }
    }
    pub fn update_cell(
        &mut self,
        cell: GlyphCell<'_>,
        rescale_overlapping: bool,
    ) -> Result<(), JsValue> {
        if self.gpu.is_none() {
            return Err(error("WebGL glyph renderer is disposed"));
        }
        let glyph = if cell.code.is_some_and(|code| code != 0) {
            self.atlas
                .as_ref()
                .map(|atlas| {
                    atlas.rasterize(
                        cell,
                        cell.chars.encode_utf16().count() > 1,
                        self.element.as_ref(),
                    )
                })
                .transpose()?
        } else {
            None
        };
        let page_width = if let (Some(atlas), Some(glyph)) = (&self.atlas, &glyph) {
            atlas
                .page(glyph.texture_page)
                .ok_or_else(|| error("Glyph references a missing atlas page"))?
                .canvas
                .width() as f64
        } else {
            0.0
        };
        self.geometry
            .update_cell(cell, glyph.as_ref(), page_width, rescale_overlapping)
            .map_err(|e| error(&e.to_string()))
    }
    pub fn set_dimensions(&mut self, dimensions: RenderDimensions) {
        self.geometry.set_dimensions(dimensions);
    }
    pub fn clear(&mut self) {
        self.geometry.clear();
    }
    pub fn handle_resize(&mut self, cols: usize, rows: usize) -> Result<(), JsValue> {
        self.gpu
            .as_ref()
            .ok_or_else(|| error("WebGL glyph renderer is disposed"))?
            .resize(self.geometry.dimensions())?;
        self.geometry
            .resize(cols, rows)
            .map_err(|e| error(&e.to_string()))
    }
    pub fn render(&mut self, model: &RenderModel) -> Result<(), JsValue> {
        let gpu = self
            .gpu
            .as_mut()
            .ok_or_else(|| error("WebGL glyph renderer is disposed"))?;
        let Some(atlas) = &self.atlas else {
            return Ok(());
        };
        if atlas.page_count() > gpu.textures.len() && !self.page_overflow_warned {
            self.page_overflow_warned = true;
            web_sys::console::warn_1(&format!("Atlas page count ({}) exceeds the renderer's texture capacity ({}), some glyphs will not render correctly", atlas.page_count(), gpu.textures.len()).into());
        }
        let attributes = self
            .geometry
            .compact(&model.line_lengths)
            .map_err(|e| error(&e.to_string()))?;
        gpu.draw(attributes, atlas.as_ref())
    }
    pub fn dispose(&mut self) {
        self.gpu = None;
        self.atlas = None;
        self.element = None;
    }
}
impl Drop for GlyphRenderer {
    fn drop(&mut self) {
        self.dispose();
    }
}
