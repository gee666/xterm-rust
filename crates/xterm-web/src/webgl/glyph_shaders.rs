// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. GlyphRenderer.ts shaders, with runtime texture capacity.
pub(super) const VERTEX: &str = "#version 300 es
layout (location = 0) in vec2 a_unitquad;
layout (location = 1) in vec2 a_cellpos;
layout (location = 2) in vec2 a_offset;
layout (location = 3) in vec2 a_size;
layout (location = 4) in float a_texpage;
layout (location = 5) in vec2 a_texcoord;
layout (location = 6) in vec2 a_texsize;
uniform mat4 u_projection;
uniform vec2 u_resolution;
out vec2 v_texcoord;
flat out int v_texpage;
void main() {
  vec2 zeroToOne = (a_offset / u_resolution) + a_cellpos + (a_unitquad * a_size);
  gl_Position = u_projection * vec4(zeroToOne, 0.0, 1.0);
  v_texpage = int(a_texpage);
  v_texcoord = a_texcoord + a_unitquad * a_texsize;
}";
pub(super) fn fragment(capacity: usize) -> String {
    let mut source = format!(
        "#version 300 es
precision lowp float;
in vec2 v_texcoord;
flat in int v_texpage;
uniform sampler2D u_texture[{capacity}];
out vec4 outColor;
void main() {{
  if (v_texpage == 0) {{ outColor = texture(u_texture[0], v_texcoord); }}"
    );
    for i in 1..capacity {
        source.push_str(&format!(
            " else if (v_texpage == {i}) {{ outColor = texture(u_texture[{i}], v_texcoord); }}"
        ));
    }
    source.push_str("\n}");
    source
}
