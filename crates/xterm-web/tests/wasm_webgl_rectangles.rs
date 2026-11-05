// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
#![cfg(target_arch = "wasm32")]
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl};
use xterm_core::buffer::constants::{attributes, fg_flags};
use xterm_web::{
    api::{RenderDimensions, Size},
    theme::ThemeService,
    webgl::{
        model::{CursorRenderModel, CursorShape, RenderModel},
        rectangle_gpu::RectangleRenderer,
    },
};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(inline_js = "
export function recordResources(gl, fail) {
  gl.__resources = [];
  for (const name of ['Shader', 'Program', 'Buffer', 'VertexArray']) {
    const create = gl['create' + name].bind(gl);
    gl['create' + name] = (...args) => {
      const result = create(...args);
      if (result) gl.__resources.push([name, result]);
      return result;
    };
  }
  if (fail === 'shader') gl.shaderSource = shader => WebGL2RenderingContext.prototype.shaderSource.call(gl, shader, 'invalid shader');
  if (fail === 'uniform') gl.getUniformLocation = () => null;
  if (fail === 'buffer') {
    const create = gl.createBuffer.bind(gl);
    let count = 0;
    gl.createBuffer = () => ++count === 3 ? null : create();
  }
}
export function liveResources(gl) { return gl.__resources.filter(([name, value]) => gl['is' + name](value)).length; }
export function recordedResources(gl) { return gl.__resources.length; }
")]
extern "C" {
    #[wasm_bindgen(js_name = recordResources)]
    fn record_resources(gl: &Gl, fail: &str);
    #[wasm_bindgen(js_name = liveResources)]
    fn live_resources(gl: &Gl) -> usize;
    #[wasm_bindgen(js_name = recordedResources)]
    fn recorded_resources(gl: &Gl) -> usize;
}
fn context() -> (HtmlCanvasElement, Gl) {
    let canvas: HtmlCanvasElement = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();
    canvas.set_width(24);
    canvas.set_height(20);
    let options = js_sys::Object::new();
    for key in ["antialias", "depth"] {
        js_sys::Reflect::set(&options, &key.into(), &false.into()).unwrap();
    }
    js_sys::Reflect::set(&options, &"preserveDrawingBuffer".into(), &true.into()).unwrap();
    let gl: Gl = canvas
        .get_context_with_context_options("webgl2", &options)
        .unwrap()
        .expect("This browser must support WebGL2 to pass GPU assertions")
        .dyn_into()
        .unwrap();
    gl.viewport(0, 0, 24, 20);
    (canvas, gl)
}
fn dimensions() -> RenderDimensions {
    let mut result = RenderDimensions::default();
    result.device.cell = Size {
        width: 8.0,
        height: 10.0,
    };
    result.device.canvas = Size {
        width: 24.0,
        height: 20.0,
    };
    result
}
fn pixel(gl: &Gl, x: i32, top_y: i32) -> [u8; 4] {
    let mut values = [0; 4];
    gl.read_pixels_with_opt_u8_array(
        x,
        19 - top_y,
        1,
        1,
        Gl::RGBA,
        Gl::UNSIGNED_BYTE,
        Some(&mut values),
    )
    .unwrap();
    assert_eq!(gl.get_error(), Gl::NO_ERROR);
    values
}
fn model() -> RenderModel {
    let mut model = RenderModel::new();
    model.resize(3, 2).unwrap();
    model
}

#[wasm_bindgen_test]
fn real_gpu_draws_default_rgb_palette_inverse_and_resizing() {
    let (_canvas, gl) = context();
    let theme = ThemeService::default();
    theme.colors().borrow().ansi[37].borrow_mut().rgba = 0x00ff0040;
    let mut renderer = RectangleRenderer::new(gl.clone(), 3, 2, dimensions(), &theme).unwrap();
    renderer.handle_resize(3, 2);
    let mut model = model();
    model.cells[1] = attributes::CM_RGB | 0xff0000;
    model.cells[5] = attributes::CM_P256 | 37;
    model.cells[10] = attributes::CM_RGB | fg_flags::INVERSE | 0x0000ff;
    renderer.update_backgrounds(&model).unwrap();
    renderer.render_backgrounds().unwrap();
    assert_eq!(pixel(&gl, 2, 2), [255, 0, 0, 255]);
    assert_eq!(pixel(&gl, 10, 2), [0, 255, 0, 255]);
    assert_eq!(pixel(&gl, 18, 2), [0, 0, 255, 255]);
    assert_eq!(pixel(&gl, 2, 12), [0, 0, 0, 255]);
    model.cells.fill(0);
    renderer.update_backgrounds(&model).unwrap();
    let mut resized = dimensions();
    resized.device.cell.width = 4.0;
    resized.device.cell.height = 5.0;
    renderer.set_dimensions(resized);
    renderer.handle_resize(3, 2);
    gl.clear_color(1.0, 0.0, 1.0, 1.0);
    gl.clear(Gl::COLOR_BUFFER_BIT);
    renderer.render_backgrounds().unwrap();
    assert_eq!(pixel(&gl, 2, 2), [0, 0, 0, 255]);
    assert_eq!(pixel(&gl, 18, 12), [255, 0, 255, 255]);
}

#[wasm_bindgen_test]
fn theme_events_update_transparent_viewport_and_cursor_shapes() {
    let (_canvas, gl) = context();
    let theme = ThemeService::default();
    let mut renderer = RectangleRenderer::new(gl.clone(), 3, 2, dimensions(), &theme).unwrap();
    renderer.handle_resize(3, 2);
    let mut model = model();
    renderer.update_backgrounds(&model).unwrap();
    theme
        .modify_colors(|colors| {
            let colors = colors.borrow();
            colors.background.borrow_mut().rgba = 0x4080c080;
            colors.cursor.borrow_mut().rgba = 0x00ff00ff;
        })
        .unwrap();
    for style in [
        CursorShape::Bar,
        CursorShape::Underline,
        CursorShape::Outline,
        CursorShape::Block,
        CursorShape::None,
    ] {
        model.cursor = Some(CursorRenderModel {
            x: 1,
            y: 0,
            width: 2,
            style,
            cursor_width: 2.0,
            device_pixel_ratio: 1.0,
        });
        renderer.update_cursor(&model);
        renderer.render_backgrounds().unwrap();
        renderer.render_cursor().unwrap();
        assert_eq!(pixel(&gl, 2, 12), [64, 128, 192, 128]);
        assert_eq!(pixel(&gl, 12, 4), [64, 128, 192, 128]);
        let green = [0, 255, 0, 255];
        let background = [64, 128, 192, 128];
        assert_eq!(
            pixel(&gl, 8, 4),
            if matches!(style, CursorShape::Bar | CursorShape::Outline) {
                green
            } else {
                background
            }
        );
        assert_eq!(
            pixel(&gl, 12, 9),
            if matches!(style, CursorShape::Underline | CursorShape::Outline) {
                green
            } else {
                background
            }
        );
        assert_eq!(
            pixel(&gl, 23, 4),
            if style == CursorShape::Outline {
                green
            } else {
                background
            }
        );
        assert_eq!(
            pixel(&gl, 12, 0),
            if style == CursorShape::Outline {
                green
            } else {
                background
            }
        );
    }
    model.cursor = None;
    renderer.update_cursor(&model);
    renderer.render_backgrounds().unwrap();
    renderer.render_cursor().unwrap();
    assert_eq!(pixel(&gl, 8, 4), [64, 128, 192, 128]);
}

#[wasm_bindgen_test]
fn disposal_and_drop_delete_all_gpu_resources_and_detach_theme_listener() {
    let (_canvas, gl) = context();
    let theme = ThemeService::default();
    record_resources(&gl, "");
    let mut renderer = RectangleRenderer::new(gl.clone(), 3, 2, dimensions(), &theme).unwrap();
    renderer.update_backgrounds(&model()).unwrap();
    renderer.render_backgrounds().unwrap();
    assert_eq!(recorded_resources(&gl), 7);
    assert_eq!(live_resources(&gl), 5);
    renderer.dispose();
    renderer.dispose();
    // A deleted current program remains alive until it is no longer current.
    gl.use_program(None);
    assert_eq!(live_resources(&gl), 0);
    // A lingering subscription would borrow the live colors and panic here.
    let colors = theme.colors();
    let borrowed = colors.borrow_mut();
    theme.modify_colors(|_| {}).unwrap();
    drop(borrowed);
    assert!(renderer.render_backgrounds().is_err());
    assert!(renderer.render_cursor().is_err());
    theme.set_theme(&Default::default()).unwrap();
    let renderer = RectangleRenderer::new(gl.clone(), 3, 2, dimensions(), &theme).unwrap();
    assert_eq!(live_resources(&gl), 5);
    drop(renderer);
    gl.use_program(None);
    assert_eq!(live_resources(&gl), 0);
    theme.set_theme(&Default::default()).unwrap();
}

#[wasm_bindgen_test]
fn construction_failures_report_errors_and_release_partial_allocations() {
    for failure in ["shader", "uniform", "buffer"] {
        let (_canvas, gl) = context();
        let theme = ThemeService::default();
        record_resources(&gl, failure);
        let result = RectangleRenderer::new(gl.clone(), 3, 2, dimensions(), &theme);
        let error = result
            .err()
            .expect("injected construction failure must be returned");
        assert!(error.is_instance_of::<js_sys::Error>());
        let message = js_sys::Reflect::get(&error, &"message".into())
            .unwrap()
            .as_string()
            .unwrap();
        assert!(!message.is_empty());
        assert_eq!(live_resources(&gl), 0, "failure {failure}");
        assert!(recorded_resources(&gl) > 0);
    }
}
