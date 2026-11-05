#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Reflect, JSON};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement};
use xterm_core::{
    buffer::marker::Marker,
    decoration::{DecorationOptions, OverviewPosition, OverviewRulerOptions},
};
use xterm_web::Terminal;
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/support/ruler_context.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = recordRulerContext)]
    fn record_context(canvas: &HtmlCanvasElement) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = nextRulerFrame)]
    async fn next_frame();
}
fn set(terminal: &Terminal, name: &str, json: &str) {
    assert!(Reflect::set(
        &terminal.options(),
        &name.into(),
        &JSON::parse(json).unwrap()
    )
    .unwrap());
}
fn canvas(parent: &HtmlElement) -> Option<HtmlCanvasElement> {
    parent
        .query_selector(".xterm-decoration-overview-ruler")
        .unwrap()
        .map(JsCast::unchecked_into)
}
fn calls(recorder: &JsValue) -> Array {
    Reflect::get(recorder, &"calls".into())
        .unwrap()
        .unchecked_into()
}
async fn paint() {
    // RenderService schedules the overview redraw from its own frame. The
    // second native animation frame observes that dependent canvas update.
    next_frame().await;
    next_frame().await;
}
fn fixture(options: &str) -> (Terminal, HtmlElement) {
    let document = web_sys::window().unwrap().document().unwrap();
    let parent: HtmlElement = document.create_element("div").unwrap().unchecked_into();
    document.body().unwrap().append_child(&parent).unwrap();
    let terminal = Terminal::new(JSON::parse(options).unwrap()).unwrap();
    terminal.open(parent.clone().into()).unwrap();
    (terminal, parent)
}

#[wasm_bindgen_test]
async fn public_scrollbar_options_lazily_create_retain_resize_and_dispose_canvas() {
    let (terminal, parent) = fixture(r#"{"cols":4,"rows":3,"scrollbar":{"width":0}}"#);
    assert!(canvas(&parent).is_none());
    set(
        &terminal,
        "scrollbar",
        r#"{"showScrollbar":false,"width":12}"#,
    );
    assert!(canvas(&parent).is_none());
    set(&terminal, "scrollbar", r#"{"width":12}"#);
    let ruler = canvas(&parent).unwrap();
    let recorder = record_context(&ruler).unwrap();
    paint().await;
    let dpr = web_sys::window().unwrap().device_pixel_ratio();
    assert_eq!(ruler.width(), (12.0 * dpr).round() as u32);
    assert!(ruler.height() > 0);
    set(
        &terminal,
        "scrollbar",
        r#"{"showScrollbar":false,"width":12}"#,
    );
    paint().await;
    assert_eq!(ruler.width(), 0);
    assert_eq!(canvas(&parent).unwrap(), ruler);
    set(&terminal, "scrollbar", r#"{"width":9}"#);
    paint().await;
    assert_eq!(ruler.width(), (9.0 * dpr).round() as u32);
    let previous_height = ruler.height();
    terminal.resize(4.0, 5.0).unwrap();
    paint().await;
    assert!(ruler.height() > previous_height);
    let render = terminal.render_service_handle().unwrap();
    render.refresh_rows(0.0, 4.0, true, false).unwrap();
    terminal.dispose().unwrap();
    let count = calls(&recorder).length();
    paint().await;
    assert_eq!(
        calls(&recorder).length(),
        count,
        "disposed ruler still painted"
    );
    assert_eq!(parent.child_element_count(), 0);
    assert!(ruler.parent_element().is_none());
    Reflect::get(&recorder, &"restore".into())
        .unwrap()
        .unchecked_into::<js_sys::Function>()
        .call0(&recorder)
        .unwrap();
    parent.remove();
}

#[wasm_bindgen_test]
async fn terminal_decorations_theme_and_alternate_buffer_update_live_canvas() {
    let (terminal, parent) = fixture(
        r##"{"cols":4,"rows":5,"scrollbar":{"width":12},"theme":{"overviewRulerBorder":"#ff0000"}}"##,
    );
    let ruler = canvas(&parent).unwrap();
    paint().await;
    let context: CanvasRenderingContext2d =
        ruler.get_context("2d").unwrap().unwrap().unchecked_into();
    assert_eq!(
        context.get_image_data(0.0, 0.0, 1.0, 1.0).unwrap().data().0,
        [255, 0, 0, 255]
    );
    set(&terminal, "theme", r##"{"overviewRulerBorder":"#0000ff"}"##);
    paint().await;
    assert_eq!(
        context.get_image_data(0.0, 0.0, 1.0, 1.0).unwrap().data().0,
        [0, 0, 255, 255]
    );
    let recorder = record_context(&ruler).unwrap();
    let mut options = DecorationOptions::new(Marker::new(2));
    options.overview_ruler_options = Some(OverviewRulerOptions {
        color: "#00ff00".into(),
        position: Some(OverviewPosition::Full),
    });
    let decoration = terminal
        .decoration_service()
        .register_decoration(Rc::new(RefCell::new(options)))
        .unwrap()
        .unwrap();
    paint().await;
    let with_decoration = JSON::stringify(&calls(&recorder))
        .unwrap()
        .as_string()
        .unwrap();
    assert!(
        with_decoration.contains("35,48,48,102,102,48,48"),
        "missing decoration color: {with_decoration}"
    );
    calls(&recorder).set_length(0);
    decoration.dispose().unwrap();
    paint().await;
    let removed = JSON::stringify(&calls(&recorder))
        .unwrap()
        .as_string()
        .unwrap();
    assert!(!removed.contains("35,48,48,102,102,48,48"));
    terminal.shared_native().write("\x1b[?1049h").unwrap();
    assert_eq!(ruler.style().get_property_value("display").unwrap(), "none");
    terminal.shared_native().write("\x1b[?1049l").unwrap();
    assert_eq!(
        ruler.style().get_property_value("display").unwrap(),
        "block"
    );
    Reflect::get(&recorder, &"restore".into())
        .unwrap()
        .unchecked_into::<js_sys::Function>()
        .call0(&recorder)
        .unwrap();
    terminal.dispose().unwrap();
    parent.remove();
}
