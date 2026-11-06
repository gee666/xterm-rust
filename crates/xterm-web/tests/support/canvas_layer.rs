// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use serde_json::Value;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{HtmlElement, HtmlTextAreaElement};
use xterm_core::{headless::Terminal, options::OptionsService};
use xterm_web::{
    api::{CharacterDimensions, CssDimensions, DeviceDimensions, RenderDimensions, Size},
    core_browser::CoreBrowserService,
    terminal::runtime::RuntimeContext,
    theme::{Theme, ThemeService},
    webgl::{canvas_layer::CanvasLayer, layer_services::LayerServices, link_layer::LinkLayerEvent},
};

#[wasm_bindgen(module = "/tests/canvas_layer_probe.js")]
extern "C" {
    #[wasm_bindgen(js_name = beginLayerCapture)]
    fn begin();
    #[wasm_bindgen(js_name = endLayerCapture)]
    fn end() -> String;
    #[wasm_bindgen(js_name = layerNote)]
    pub fn note(value: &str);
    #[wasm_bindgen(js_name = layerScheduled)]
    pub fn scheduled();
    #[wasm_bindgen(js_name = failLayer)]
    pub fn fail(key: &str);
    #[wasm_bindgen(js_name = layerFailure)]
    pub fn failure() -> JsValue;
    #[wasm_bindgen(js_name = layerSnapshot)]
    pub fn snapshot(canvas: &web_sys::HtmlCanvasElement);
    #[wasm_bindgen(js_name = reenterLayer)]
    pub fn reenter(key: &str, callback: &js_sys::Function);
    #[wasm_bindgen(js_name = reenterLayerDpr)]
    pub fn reenter_dpr(callback: &js_sys::Function);
    #[wasm_bindgen(js_name = sourceLayer)]
    pub fn source(name: &str) -> String;
}
pub struct Capture(bool);
impl Capture {
    pub fn new() -> Self {
        begin();
        Self(true)
    }
    pub fn finish(mut self) -> Value {
        self.0 = false;
        serde_json::from_str(&end()).unwrap()
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        if self.0 {
            end();
        }
    }
}
pub struct Harness {
    pub services: Rc<LayerServices>,
    pub container: HtmlElement,
    textarea: HtmlTextAreaElement,
}
impl Harness {
    pub fn new() -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
        let textarea: HtmlTextAreaElement = document
            .create_element("textarea")
            .unwrap()
            .dyn_into()
            .unwrap();
        document.body().unwrap().append_child(&container).unwrap();
        document.body().unwrap().append_child(&textarea).unwrap();
        let browser = CoreBrowserService::new(&textarea, &window, &document).unwrap();
        let options = Rc::new(OptionsService::new(
            false,
            [
                ("cols", 8.0.into()),
                ("rows", 4.0.into()),
                ("fontFamily", "monospace".into()),
                ("fontSize", 14.0.into()),
            ],
        ));
        let terminal = Rc::new(Terminal::with_options(options).unwrap());
        let theme = Rc::new(ThemeService::new(
            &Self::theme("#eeeeee", "#102030"),
            Rc::new(xterm_core::color::css::to_color),
        ));
        let cache = RuntimeContext::default().webgl_atlas_cache(&document);
        Self {
            services: Rc::new(LayerServices {
                terminal,
                browser,
                theme,
                cache,
            }),
            container,
            textarea,
        }
    }
    pub fn theme(foreground: &str, background: &str) -> Theme {
        Theme {
            foreground: Some(foreground.into()),
            background: Some(background.into()),
            ..Theme::default()
        }
    }
    pub fn base(&self, alpha: bool) -> Rc<CanvasLayer> {
        let base = CanvasLayer::new(
            self.services.clone(),
            self.container.clone(),
            "test",
            3,
            alpha,
        )
        .unwrap();
        base.subscribe_theme(|| {
            note("reset");
            Ok(())
        });
        base
    }
    pub fn children(&self) {
        note(&format!(
            "children:{}",
            self.container.child_element_count()
        ));
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        self.services
            .cache
            .remove_terminal(&Rc::downgrade(&self.services.terminal))
            .unwrap();
        self.services.browser.dispose().unwrap();
        self.container.remove();
        self.textarea.remove();
    }
}
pub fn dim() -> RenderDimensions {
    RenderDimensions {
        device: DeviceDimensions {
            cell: Size {
                width: 10.0,
                height: 20.0,
            },
            character: CharacterDimensions {
                width: 8.0,
                height: 16.0,
                left: 1.0,
                top: 2.0,
            },
            canvas: Size {
                width: 80.0,
                height: 80.0,
            },
        },
        css: CssDimensions {
            cell: Size {
                width: 10.0 / 1.5,
                height: 20.0 / 1.5,
            },
            canvas: Size {
                width: 80.0 / 1.5,
                height: 80.0 / 1.5,
            },
        },
    }
}
pub fn event(x1: f64, y1: f64, x2: f64, y2: f64) -> LinkLayerEvent {
    LinkLayerEvent {
        x1,
        y1,
        x2,
        y2,
        cols: 8.0,
        foreground: None,
    }
}
pub fn live(x1: f64, y1: f64, x2: f64, y2: f64) -> Rc<Cell<LinkLayerEvent>> {
    Rc::new(Cell::new(event(x1, y1, x2, y2)))
}
pub fn redraw(rows: usize) -> Result<(), JsValue> {
    note(&format!("grid:0:{}", rows - 1));
    Ok(())
}
