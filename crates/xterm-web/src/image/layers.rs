// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{
    js::{call, context, get, nullish, number, number_or, set},
    primitives::create_canvas,
    BrowserImageRenderer,
};
use wasm_bindgen::JsValue;
use xterm_addons::image::storage::ImageLayer;

impl BrowserImageRenderer {
    pub fn insert_layer(&self, layer: ImageLayer) -> Result<(), JsValue> {
        let document = self.host.document()?;
        let screen = self.host.screen_element()?;
        if nullish(&document) || nullish(&screen) {
            web_sys::console::warn_1(&"image addon: cannot insert output canvas to DOM, missing document or screenElement".into());
            return Ok(());
        }
        if self.has_layer(layer) {
            return Ok(());
        }
        let dimensions = self.host.dimensions()?.unwrap_or_default();
        let canvas = create_canvas(
            &document,
            number_or(dimensions.canvas_width, 0.0),
            number_or(dimensions.canvas_height, 0.0),
        )?;
        let name = match layer {
            ImageLayer::Top => "top",
            ImageLayer::Bottom => "bottom",
        };
        call(
            &get(&canvas, "classList")?,
            "add",
            &[format!("xterm-image-layer-{name}").into()],
        )?;
        // Isolation contains negative-z image layers without replacing an
        // integrator's z-index on the screen element.
        let screen = self.host.screen_element()?;
        set(&get(&screen, "style")?, "isolation", &"isolate".into())?;
        set(
            &get(&canvas, "style")?,
            "zIndex",
            &if layer == ImageLayer::Bottom {
                "-1"
            } else {
                "0"
            }
            .into(),
        )?;
        if layer == ImageLayer::Bottom {
            call(
                &screen,
                "insertBefore",
                &[canvas.clone(), get(&screen, "firstChild")?],
            )?;
        } else {
            call(&screen, "appendChild", &[canvas.clone()])?;
        }
        let Some(context) = context(&canvas, Some(true))? else {
            call(&canvas, "remove", &[])?;
            return Ok(());
        };
        self.layers.set(&super::layer_key(layer), &context);
        self.clear_all(Some(layer))
    }

    pub fn remove_layer(&self, layer: ImageLayer) -> Result<(), JsValue> {
        if let Some(context) = self.context(layer) {
            call(&get(&context, "canvas")?, "remove", &[])?;
            self.layers.delete(&super::layer_key(layer));
        }
        Ok(())
    }

    pub fn rescale_canvas(&self) -> Result<(), JsValue> {
        let dimensions = self.host.dimensions()?.unwrap_or_default();
        let width = number_or(dimensions.canvas_width, 0.0);
        let height = number_or(dimensions.canvas_height, 0.0);
        // A live Map iterator observes layer insertion/removal during callbacks.
        for ctx in self.layers.values() {
            let ctx = ctx?;
            let canvas = get(&ctx, "canvas")?;
            if number(&canvas, "width")? != width || number(&canvas, "height")? != height {
                set(&canvas, "width", &width.into())?;
                set(&canvas, "height", &height.into())?;
            }
        }
        Ok(())
    }

    pub fn clear_lines(
        &self,
        start: f64,
        end: f64,
        layer: Option<ImageLayer>,
    ) -> Result<(), JsValue> {
        let d = self.host.dimensions()?.unwrap_or_default();
        let height = number_or(d.cell_height, 0.0);
        for key in [ImageLayer::Top, ImageLayer::Bottom] {
            if layer.is_none() || layer == Some(key) {
                if let Some(ctx) = self.context(key) {
                    call(
                        &ctx,
                        "clearRect",
                        &[
                            0.into(),
                            (start * height).into(),
                            number_or(d.canvas_width, 0.0).into(),
                            ((end + 1.0 - start) * height).into(),
                        ],
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn clear_all(&self, layer: Option<ImageLayer>) -> Result<(), JsValue> {
        for key in [ImageLayer::Top, ImageLayer::Bottom] {
            if layer.is_none() || layer == Some(key) {
                if let Some(ctx) = self.context(key) {
                    let canvas = get(&ctx, "canvas")?;
                    call(
                        &ctx,
                        "clearRect",
                        &[
                            0.into(),
                            0.into(),
                            get(&canvas, "width")?,
                            get(&canvas, "height")?,
                        ],
                    )?;
                }
            }
        }
        Ok(())
    }
}
