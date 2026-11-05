// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-webgl/src/RectangleRenderer.ts.
use super::{
    gpu_resources::{error, Buffer, Program, UnitQuad, VertexArray},
    math::PROJECTION_MATRIX,
    model::RenderModel,
    rectangles::{RectangleGeometry, RectangleVertices},
};
use crate::{api::RenderDimensions, theme::ThemeService};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext as Gl, WebGlUniformLocation};
use xterm_core::lifecycle::{Disposable, DisposableStore};

const VERTEX_SHADER: &str = "#version 300 es
layout (location = 0) in vec2 a_position;
layout (location = 1) in vec2 a_size;
layout (location = 2) in vec4 a_color;
layout (location = 3) in vec2 a_unitquad;
uniform mat4 u_projection;
out vec4 v_color;
void main() {
  vec2 zeroToOne = a_position + (a_unitquad * a_size);
  gl_Position = u_projection * vec4(zeroToOne, 0.0, 1.0);
  v_color = a_color;
}";
const FRAGMENT_SHADER: &str = "#version 300 es
precision lowp float;
in vec4 v_color;
out vec4 outColor;
void main() { outColor = v_color; }";
const STRIDE: i32 = 8 * 4;

struct Gpu {
    gl: Gl,
    program: Program,
    projection: WebGlUniformLocation,
    vao: VertexArray,
    attributes: Buffer,
    _quad: UnitQuad,
}
impl Gpu {
    fn new(gl: Gl) -> Result<Self, JsValue> {
        let program = Program::new(&gl, VERTEX_SHADER, FRAGMENT_SHADER)?;
        let projection = gl
            .get_uniform_location(&program.value, "u_projection")
            .ok_or_else(|| error("WebGL projection uniform was not found"))?;
        let vao = VertexArray::new(&gl)?;
        gl.bind_vertex_array(Some(&vao.value));
        let quad = UnitQuad::new(&gl, 3)?;
        let attributes = Buffer::new(&gl)?;
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&attributes.value));
        for (location, size, offset) in [(0, 2, 0), (1, 2, 8), (2, 4, 16)] {
            gl.enable_vertex_attrib_array(location);
            gl.vertex_attrib_pointer_with_i32(location, size, Gl::FLOAT, false, STRIDE, offset);
            gl.vertex_attrib_divisor(location, 1);
        }
        Ok(Self {
            gl,
            program,
            projection,
            vao,
            attributes,
            _quad: quad,
        })
    }
    fn render(&self, vertices: &RectangleVertices) -> Result<(), JsValue> {
        if self.gl.is_context_lost() {
            return Err(error("WebGL context is lost"));
        }
        let count =
            i32::try_from(vertices.count()).map_err(|_| error("Too many WebGL rectangles"))?;
        self.gl.use_program(Some(&self.program.value));
        self.gl.bind_vertex_array(Some(&self.vao.value));
        self.gl
            .uniform_matrix4fv_with_f32_array(Some(&self.projection), false, &PROJECTION_MATRIX);
        self.gl
            .bind_buffer(Gl::ARRAY_BUFFER, Some(&self.attributes.value));
        // Copy, rather than unsafe views into WASM memory. WebGL owns its upload.
        let values = js_sys::Float32Array::from(vertices.attributes());
        self.gl
            .buffer_data_with_array_buffer_view(Gl::ARRAY_BUFFER, &values, Gl::DYNAMIC_DRAW);
        self.gl.draw_elements_instanced_with_i32(
            Gl::TRIANGLE_STRIP,
            4,
            Gl::UNSIGNED_BYTE,
            0,
            count,
        );
        Ok(())
    }
}

/// Actual WebGL2 instanced rectangle drawing, independent of terminal/DOM state.
/// The caller owns canvas sizing, viewport and blend state, as WebglRenderer
/// does upstream. Theme changes update cached viewport/cursor colors immediately.
/// Disposal unregisters the theme listener and deletes all owned GPU resources.
/// This component does not provide glyph rendering or a WebglAddon facade.
pub struct RectangleRenderer {
    gpu: Option<Gpu>,
    geometry: Rc<RefCell<RectangleGeometry>>,
    listeners: DisposableStore,
}
impl RectangleRenderer {
    pub fn new(
        gl: Gl,
        cols: usize,
        rows: usize,
        dimensions: RenderDimensions,
        theme: &ThemeService,
    ) -> Result<Self, JsValue> {
        let gpu = Gpu::new(gl)?;
        let geometry = Rc::new(RefCell::new(RectangleGeometry::new(
            cols,
            rows,
            dimensions,
            theme.colors(),
        )));
        let listeners = DisposableStore::new();
        let weak = Rc::downgrade(&geometry);
        listeners.add(theme.on_change_colors().subscribe(move |_| {
            if let Some(geometry) = weak.upgrade() {
                geometry.borrow_mut().colors_changed();
            }
        }));
        Ok(Self {
            gpu: Some(gpu),
            geometry,
            listeners,
        })
    }
    pub fn set_dimensions(&mut self, dimensions: RenderDimensions) {
        self.geometry.borrow_mut().set_dimensions(dimensions);
    }
    pub fn handle_resize(&mut self, cols: usize, rows: usize) {
        self.geometry.borrow_mut().handle_resize(cols, rows);
    }
    pub fn update_backgrounds(&mut self, model: &RenderModel) -> Result<(), JsValue> {
        self.geometry
            .borrow_mut()
            .update_backgrounds(model)
            .map_err(|e| error(&e.to_string()))
    }
    pub fn update_cursor(&mut self, model: &RenderModel) {
        self.geometry.borrow_mut().update_cursor(model);
    }
    pub fn render_backgrounds(&self) -> Result<(), JsValue> {
        self.gpu
            .as_ref()
            .ok_or_else(|| error("WebGL rectangle renderer is disposed"))?
            .render(self.geometry.borrow().backgrounds())
    }
    pub fn render_cursor(&self) -> Result<(), JsValue> {
        self.gpu
            .as_ref()
            .ok_or_else(|| error("WebGL rectangle renderer is disposed"))?
            .render(self.geometry.borrow().cursors())
    }
    pub fn dispose(&mut self) {
        self.listeners.dispose();
        self.gpu = None;
    }
}
impl Drop for RectangleRenderer {
    fn drop(&mut self) {
        self.dispose();
    }
}
