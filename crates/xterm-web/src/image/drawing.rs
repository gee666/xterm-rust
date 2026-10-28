// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{
    js::{call, context, number},
    primitives::create_canvas,
    BrowserImageRenderer, BrowserImageSurface,
};
use wasm_bindgen::JsValue;
use xterm_addons::image::storage::{CellSize, ImageResource, ImageSpec};

fn clipped(offset: f64, extent: f64, limit: f64) -> f64 {
    if offset + extent > limit {
        limit - offset
    } else {
        extent
    }
}

impl BrowserImageRenderer {
    fn rescale_image(&self, spec: &ImageSpec, size: CellSize) -> Result<JsValue, JsValue> {
        if size != spec.actual_cell_size.get() {
            if size == spec.original_cell_size {
                spec.replace_actual_resource(Some(spec.original.clone()));
                spec.actual_cell_size.set(size);
            } else {
                let original = self.source(&spec.original)?;
                let canvas = create_canvas(
                    &self.host.document()?,
                    (number(&original, "width")? * size.width / spec.original_cell_size.width)
                        .ceil(),
                    (number(&original, "height")? * size.height / spec.original_cell_size.height)
                        .ceil(),
                )?;
                if let Some(ctx) = context(&canvas, None)? {
                    call(
                        &ctx,
                        "drawImage",
                        &[
                            original,
                            0.into(),
                            0.into(),
                            number(&canvas, "width")?.into(),
                            number(&canvas, "height")?.into(),
                        ],
                    )?;
                    spec.replace_actual_resource(Some(ImageResource::new(
                        BrowserImageSurface::canvas(canvas),
                    )));
                    spec.actual_cell_size.set(size);
                }
            }
        }
        let actual = spec
            .actual_resource()
            .ok_or_else(|| js_sys::TypeError::new("Image has no actual surface"))?;
        self.source(&actual)
    }

    pub fn draw(
        &self,
        spec: &ImageSpec,
        tile_id: f64,
        col: f64,
        row: f64,
        count: f64,
    ) -> Result<(), JsValue> {
        let Some(ctx) = self.context(spec.layer) else {
            return Ok(());
        };
        let size = self.cell_size()?;
        if size.width == -1.0 || size.height == -1.0 {
            return Ok(());
        }
        let image = self.rescale_image(spec, size)?;
        let width = number(&image, "width")?;
        let height = number(&image, "height")?;
        let cols = (width / size.width).ceil();
        let sx = (tile_id % cols) * size.width;
        let sy = (tile_id / cols).floor() * size.height;
        let fw = clipped(sx, count * size.width, width).ceil();
        let fh = clipped(sy, size.height, height).ceil();
        // Ceil extents but floor offsets. This prevents fractional-cell striping
        // and avoids source bounds overreads on Safari.
        call(
            &ctx,
            "drawImage",
            &[
                image,
                sx.floor().into(),
                sy.floor().into(),
                fw.into(),
                fh.into(),
                (col * size.width).floor().into(),
                (row * size.height).floor().into(),
                fw.into(),
                fh.into(),
            ],
        )?;
        Ok(())
    }

    pub fn extract_tile(
        &self,
        spec: &ImageSpec,
        tile_id: f64,
    ) -> Result<Option<ImageResource>, JsValue> {
        let size = self.cell_size()?;
        if size.width == -1.0 || size.height == -1.0 {
            return Ok(None);
        }
        let image = self.rescale_image(spec, size)?;
        let width = number(&image, "width")?;
        let height = number(&image, "height")?;
        let cols = (width / size.width).ceil();
        let sx = (tile_id % cols) * size.width;
        let sy = (tile_id / cols).floor() * size.height;
        let fw = clipped(sx, size.width, width);
        let fh = clipped(sy, size.height, height);
        let canvas = create_canvas(&self.host.document()?, fw, fh)?;
        let Some(ctx) = context(&canvas, None)? else {
            return Ok(None);
        };
        call(
            &ctx,
            "drawImage",
            &[
                image,
                sx.floor().into(),
                sy.floor().into(),
                fw.floor().into(),
                fh.floor().into(),
                0.into(),
                0.into(),
                fw.floor().into(),
                fh.floor().into(),
            ],
        )?;
        Ok(Some(ImageResource::new(BrowserImageSurface::canvas(
            canvas,
        ))))
    }
}
