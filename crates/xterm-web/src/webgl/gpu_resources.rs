// Copyright (c) 2017-2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. WebglUtils.ts shader operations with RAII ownership.
use wasm_bindgen::JsValue;
use web_sys::{
    WebGl2RenderingContext as Gl, WebGlBuffer, WebGlProgram, WebGlShader, WebGlVertexArrayObject,
};

pub(super) fn error(message: &str) -> JsValue {
    js_sys::Error::new(message).into()
}

struct Shader {
    gl: Gl,
    value: WebGlShader,
}
impl Shader {
    fn compile(gl: &Gl, kind: u32, source: &str) -> Result<Self, JsValue> {
        let shader = Self {
            gl: gl.clone(),
            value: gl
                .create_shader(kind)
                .ok_or_else(|| error("WebGL could not allocate a shader"))?,
        };
        gl.shader_source(&shader.value, source);
        gl.compile_shader(&shader.value);
        if gl
            .get_shader_parameter(&shader.value, Gl::COMPILE_STATUS)
            .as_bool()
            != Some(true)
        {
            return Err(error(&format!(
                "WebGL shader compilation failed: {}",
                gl.get_shader_info_log(&shader.value).unwrap_or_default()
            )));
        }
        Ok(shader)
    }
}
impl Drop for Shader {
    fn drop(&mut self) {
        self.gl.delete_shader(Some(&self.value));
    }
}

pub(super) struct Program {
    gl: Gl,
    pub value: WebGlProgram,
}
impl Program {
    pub fn new(gl: &Gl, vertex: &str, fragment: &str) -> Result<Self, JsValue> {
        let vertex = Shader::compile(gl, Gl::VERTEX_SHADER, vertex)?;
        let fragment = Shader::compile(gl, Gl::FRAGMENT_SHADER, fragment)?;
        let program = Self {
            gl: gl.clone(),
            value: gl
                .create_program()
                .ok_or_else(|| error("WebGL could not allocate a program"))?,
        };
        gl.attach_shader(&program.value, &vertex.value);
        gl.attach_shader(&program.value, &fragment.value);
        gl.link_program(&program.value);
        if gl
            .get_program_parameter(&program.value, Gl::LINK_STATUS)
            .as_bool()
            != Some(true)
        {
            return Err(error(&format!(
                "WebGL program linking failed: {}",
                gl.get_program_info_log(&program.value).unwrap_or_default()
            )));
        }
        // Detach so dropping shaders actually releases them while the program lives.
        gl.detach_shader(&program.value, &vertex.value);
        gl.detach_shader(&program.value, &fragment.value);
        Ok(program)
    }
}
impl Drop for Program {
    fn drop(&mut self) {
        self.gl.delete_program(Some(&self.value));
    }
}

pub(super) struct Buffer {
    gl: Gl,
    pub value: WebGlBuffer,
}
impl Buffer {
    pub fn new(gl: &Gl) -> Result<Self, JsValue> {
        Ok(Self {
            gl: gl.clone(),
            value: gl
                .create_buffer()
                .ok_or_else(|| error("WebGL could not allocate a buffer"))?,
        })
    }
}
impl Drop for Buffer {
    fn drop(&mut self) {
        self.gl.delete_buffer(Some(&self.value));
    }
}

/// Static triangle-strip quad shared by the two instanced renderers.
/// The buffers remain owned even though the VAO retains their bindings.
pub(super) struct UnitQuad {
    _vertices: Buffer,
    _indices: Buffer,
}
impl UnitQuad {
    pub fn new(gl: &Gl, location: u32) -> Result<Self, JsValue> {
        let vertices = Buffer::new(gl)?;
        let indices = Buffer::new(gl)?;
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&vertices.value));
        let values =
            js_sys::Float32Array::from([0.0_f32, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0].as_slice());
        gl.buffer_data_with_array_buffer_view(Gl::ARRAY_BUFFER, &values, Gl::STATIC_DRAW);
        gl.enable_vertex_attrib_array(location);
        gl.vertex_attrib_pointer_with_i32(location, 2, Gl::FLOAT, false, 0, 0);
        gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&indices.value));
        let values = js_sys::Uint8Array::from([0_u8, 1, 2, 3].as_slice());
        gl.buffer_data_with_array_buffer_view(Gl::ELEMENT_ARRAY_BUFFER, &values, Gl::STATIC_DRAW);
        Ok(Self {
            _vertices: vertices,
            _indices: indices,
        })
    }
}

pub(super) struct VertexArray {
    gl: Gl,
    pub value: WebGlVertexArrayObject,
}
impl VertexArray {
    pub fn new(gl: &Gl) -> Result<Self, JsValue> {
        Ok(Self {
            gl: gl.clone(),
            value: gl
                .create_vertex_array()
                .ok_or_else(|| error("WebGL could not allocate a vertex array"))?,
        })
    }
}
impl Drop for VertexArray {
    fn drop(&mut self) {
        self.gl.delete_vertex_array(Some(&self.value));
    }
}
