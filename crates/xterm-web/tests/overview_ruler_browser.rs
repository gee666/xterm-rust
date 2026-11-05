#![cfg(target_arch = "wasm32")]
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_core::{
    buffer::marker::Marker,
    decoration::{DecorationOptions, DecorationService, OverviewPosition, OverviewRulerOptions},
    event::Emitter,
    headless::Terminal,
    options::{OptionObject, OptionValue},
};
use xterm_web::{
    char_size_browser::CharSizeService,
    renderer::{
        overview_ruler::{OverviewRulerRenderer, OverviewServices},
        service::RenderService,
        width_cache::Font,
    },
    theme::ThemeService,
};
#[path = "support/render_backend.rs"]
mod support;
use support::*;
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/support/render_service_host.mjs")]
extern "C" {
    #[wasm_bindgen(catch,js_name=withRenderHarness)]
    fn with_harness(callback: &js_sys::Function) -> Result<JsValue, JsValue>;
}
#[wasm_bindgen(module = "/tests/support/ruler_context.mjs")]
extern "C" {
    #[wasm_bindgen(catch,js_name=recordRulerContext)]
    fn record_context(canvas: &web_sys::HtmlCanvasElement) -> Result<JsValue, JsValue>;
}
fn props(values: impl IntoIterator<Item = (&'static str, OptionValue)>) -> OptionValue {
    let values: BTreeMap<_, _> = values.into_iter().map(|(k, v)| (k.to_owned(), v)).collect();
    OptionValue::Object(OptionObject::with_properties(
        values,
        |_| Ok("[object Object]".into()),
        |values, key| Ok(values.get(key).cloned().unwrap_or(OptionValue::Undefined)),
    ))
}
fn run_cases(host: JsValue) -> Result<JsValue, JsValue> {
    let fixtures: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/overview-ruler.json")).unwrap();
    let document = web_sys::window().unwrap().document().unwrap();
    let sheet = document.create_element("style")?;
    sheet.set_text_content(Some(include_str!("../assets/xterm.css")));
    document
        .query_selector("head")?
        .unwrap()
        .append_child(&sheet)?;
    for case in fixtures {
        let length = case["length"].as_u64().unwrap() as usize;
        let dpr = case["dpr"].as_f64().unwrap();
        let height = case["height"].as_f64().unwrap();
        let native =
            Rc::new(Terminal::new(4, length, 0).map_err(|e| js_sys::Error::new(&e.to_string()))?);
        let options = native.with_state(|s| s.options.clone()).unwrap();
        options
            .set(
                "scrollbar",
                props([
                    ("showScrollbar", true.into()),
                    ("width", case["width"].as_f64().unwrap().into()),
                    (
                        "overviewRuler",
                        props([
                            ("showTopBorder", true.into()),
                            ("showBottomBorder", true.into()),
                        ]),
                    ),
                ]),
            )
            .map_err(|e| js_sys::Error::new(&e.to_string()))?;
        let browser = Rc::new(Browser {
            window: js_sys::Reflect::get(&host, &"window".into())?.unchecked_into(),
            pixel_ratio: dpr,
            dpr: Emitter::new(),
            changed: Emitter::new(),
        });
        let container: web_sys::HtmlElement = document.create_element("div")?.unchecked_into();
        container.set_class_name("xterm");
        document.body().unwrap().append_child(&container)?;
        let viewport: web_sys::HtmlElement = document.create_element("div")?.unchecked_into();
        container.append_child(&viewport)?;
        let metrics = Rc::new(CharSizeService::new(
            &document,
            &container,
            Font {
                family: "monospace".into(),
                size: 15.0,
                ..Font::default()
            },
        )?);
        metrics
            .state
            .update_measurement(Some(5.0), Some(10.0))
            .unwrap();
        let theme = Rc::new(ThemeService::default());
        let buffer = native.with_state(|s| s.buffer_service.clone()).unwrap();
        let decorations = DecorationService::with_buffers(&buffer.buffers).unwrap();
        for raw in case["decorations"].as_array().unwrap() {
            let mut decoration = DecorationOptions::new(Marker::new(raw["line"].as_i64().unwrap()));
            let position = match raw["position"].as_str().unwrap() {
                "left" => OverviewPosition::Left,
                "center" => OverviewPosition::Center,
                "right" => OverviewPosition::Right,
                "full" => OverviewPosition::Full,
                _ => panic!("unknown fixture lane"),
            };
            decoration.overview_ruler_options = Some(OverviewRulerOptions {
                color: raw["color"].as_str().unwrap().into(),
                position: Some(position),
            });
            decorations
                .register_decoration(Rc::new(RefCell::new(decoration)))
                .unwrap();
        }
        let service = RenderService::new(
            browser.clone(),
            container.clone(),
            native,
            theme.clone(),
            metrics,
            decorations.clone(),
        )?;
        let log: Log = Rc::default();
        let backend = Backend::new("A", &log);
        let mut dimensions = backend.dimensions.get();
        dimensions.css.canvas.height = height / dpr;
        dimensions.device.canvas.height = height;
        backend.dimensions.set(dimensions);
        service.set_renderer(backend.clone())?;
        let ruler = OverviewRulerRenderer::new(
            &viewport,
            &document,
            OverviewServices {
                buffer,
                decorations,
                render: service.clone(),
                options,
                theme,
                browser,
            },
        )?;
        let canvas = ruler.canvas();
        let recorder = record_context(&canvas)?;
        backend.request(0.0, 0.0, false)?;
        invoke(&host, "frame", &[])?;
        let calls = js_sys::Reflect::get(&recorder, &"calls".into())?;
        let calls: serde_json::Value =
            serde_json::from_str(&js_sys::JSON::stringify(&calls)?.as_string().unwrap()).unwrap();
        assert_eq!(
            calls, case["calls"],
            "overview source drawing call mismatch"
        );
        assert_eq!(
            u64::from(canvas.width()),
            case["canvas"]["width"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(canvas.height()),
            case["canvas"]["height"].as_u64().unwrap()
        );
        let computed = document
            .default_view()
            .unwrap()
            .get_computed_style(&canvas)?
            .unwrap();
        assert_eq!(computed.get_property_value("position")?, "absolute");
        assert_eq!(computed.get_property_value("z-index")?, "8");
        assert_eq!(computed.get_property_value("pointer-events")?, "none");
        if canvas.width() > 0 && canvas.height() > 0 {
            let context: web_sys::CanvasRenderingContext2d =
                canvas.get_context("2d")?.unwrap().unchecked_into();
            assert_eq!(
                context.get_image_data(0.0, 0.0, 1.0, 1.0)?.data().0,
                [255, 255, 255, 255]
            );
        }
        invoke(&recorder, "restore", &[])?;
        ruler.dispose()?;
        service.dispose()?;
        container.remove();
        assert_eq!(
            pending(&host)?,
            "{\"frames\":0,\"timers\":0,\"idle\":0,\"observers\":0}"
        );
    }
    sheet.remove();
    Ok(JsValue::UNDEFINED)
}
#[wasm_bindgen_test]
fn source_rectangles_pixels_styles_and_cleanup_match() {
    let callback =
        Closure::wrap(Box::new(run_cases) as Box<dyn Fn(JsValue) -> Result<JsValue, JsValue>>);
    with_harness(callback.as_ref().unchecked_ref()).unwrap();
}
