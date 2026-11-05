// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;

#[wasm_bindgen_test]
fn actual_gpu_preserves_signed_contrast_keys_and_source_atlas_pixels() {
    let source: Value =
        serde_json::from_str(&source_scenario("signedGpuContrast").unwrap()).unwrap();
    let atlas = make_atlas(&source["spec"]);
    let (canvas, gl, mut renderer) = gpu(atlas.clone());
    let mut model = RenderModel::new();
    model.resize(2, 1).unwrap();
    model.line_lengths[0] = 1;
    for (index, step) in source["spec"]["steps"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let current = [
            step["bg"].as_i64().unwrap(),
            step["fg"].as_i64().unwrap(),
            0,
        ];
        renderer
            .update_cell_with_colors(
                GlyphCell {
                    x: 0,
                    y: 0,
                    code: Some(77),
                    chars: &[77],
                    width: 1.0,
                    bg: current[0] as u32,
                    fg: current[1] as u32,
                    ext: 0,
                    last_bg: 0,
                },
                GlyphColorNumbers {
                    current,
                    previous_background: 0,
                },
                false,
            )
            .unwrap();
        // Check before any direct atlas call, which could otherwise hide a
        // missing or incorrectly forwarded key by rasterizing it itself.
        assert_eq!(
            atlas
                .pages()
                .iter()
                .map(|p| p.glyphs.borrow().len())
                .sum::<usize>(),
            index + 1
        );
        let cached = atlas
            .get_glyph_with_colors(GlyphKey::Number(77), current, false, None)
            .unwrap();
        assert_same(
            &glyph(&cached),
            &source["expected"]["initial"][index],
            "GPU contrast glyph",
        );
        renderer.render(&model).unwrap();
        assert_eq!(gl.get_error(), Gl::NO_ERROR);
    }
    assert_eq!(atlas.pages().len(), 1);
    let page = atlas.pages()[0].clone();
    let data = page
        .context
        .get_image_data(
            0.0,
            0.0,
            page.canvas.width().into(),
            page.canvas.height().into(),
        )
        .unwrap()
        .data()
        .0;
    // FNV over all RGBA bytes is the source probe's exact page-pixel assertion.
    let hash = data.iter().fold(2166136261_u32, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(16777619)
    });
    assert_eq!(
        u64::from(hash),
        source["expected"]["pages"][0]["hash"].as_u64().unwrap()
    );
    renderer.dispose();
    atlas.dispose().unwrap();
    canvas.remove();
}
