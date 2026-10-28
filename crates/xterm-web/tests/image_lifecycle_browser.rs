#![cfg(target_arch = "wasm32")]
#[path = "image_lifecycle_support/mod.rs"]
mod differential;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_addons::image::storage::ImageLayer;
use xterm_web::{
    image::ImageRendererLifecycle, renderer::service_api::RendererChangeAction, Terminal,
};
wasm_bindgen_test_configure!(run_in_browser);

fn eval(code: &str) -> JsValue {
    js_sys::eval(code).unwrap()
}
fn get(value: &JsValue, key: &str) -> JsValue {
    js_sys::Reflect::get(value, &key.into()).unwrap()
}
fn set(value: &JsValue, key: &str, next: &JsValue) {
    js_sys::Reflect::set(value, &key.into(), next).unwrap();
}
fn call(value: &JsValue, key: &str, args: &[JsValue]) -> JsValue {
    get(value, key)
        .unchecked_into::<js_sys::Function>()
        .apply(value, &args.iter().cloned().collect::<js_sys::Array>())
        .unwrap()
}
async fn frame() {
    JsFuture::from(
        eval("new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))")
            .unchecked_into::<js_sys::Promise>(),
    )
    .await
    .unwrap();
}
struct Live {
    terminal: Terminal,
    parent: web_sys::Element,
}
impl Live {
    fn new() -> Self {
        let terminal = Terminal::new(eval("({cols:10,rows:4,fontSize:14})")).unwrap();
        let parent = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap();
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&parent)
            .unwrap();
        Self { terminal, parent }
    }
    fn open(&self) {
        self.terminal.open(self.parent.clone().into()).unwrap();
    }
}
impl Drop for Live {
    fn drop(&mut self) {
        self.terminal.dispose().unwrap();
        self.parent.remove();
    }
}

#[wasm_bindgen_test]
fn lazy_open_failure_then_success_and_same_storage_renderer_identity() {
    let live = Live::new();
    let owner = ImageRendererLifecycle::new(&live.terminal).unwrap();
    let renderer = owner.renderer();
    assert!(Rc::ptr_eq(&renderer, &owner.renderer()));
    assert_eq!(renderer.cell_size().unwrap().height, -1.0);
    owner.show_placeholder(true).unwrap();
    assert!(live.terminal.open(JsValue::NULL).is_err());
    live.open();
    renderer.insert_layer(ImageLayer::Bottom).unwrap();
    renderer.insert_layer(ImageLayer::Top).unwrap();
    let top = renderer.canvas(ImageLayer::Top).unwrap().unwrap();
    let screen = live.terminal.screen_element().unwrap();
    assert!(get(&top, "parentElement") == JsValue::from(screen.clone()));
    assert!(get(&top, "ownerDocument") == JsValue::from(screen.owner_document().unwrap()));
    assert_eq!(
        renderer.cell_size().unwrap().height,
        live.terminal.render_dimensions().unwrap().css.cell.height
    );
    let service = live.terminal.render_service_handle().unwrap();
    service
        .set_renderer(service.current_renderer().unwrap())
        .unwrap();
    assert!(!renderer.has_layer(ImageLayer::Top));
    assert!(!renderer.has_layer(ImageLayer::Bottom));
    owner.dispose().unwrap();
}

#[wasm_bindgen_test]
fn attach_after_open_font_size_rescales_actual_canvases_only_while_owned() {
    let live = Live::new();
    live.open();
    let owner = ImageRendererLifecycle::new(&live.terminal).unwrap();
    let renderer = owner.renderer();
    renderer.insert_layer(ImageLayer::Top).unwrap();
    let canvas = renderer.canvas(ImageLayer::Top).unwrap().unwrap();
    set(&canvas, "width", &1.into());
    set(&live.terminal.options().into(), "fontSize", &22.into());
    let dimensions = live.terminal.render_dimensions().unwrap();
    assert_eq!(
        get(&canvas, "width").as_f64(),
        Some(dimensions.css.canvas.width)
    );
    assert_eq!(
        get(&canvas, "height").as_f64(),
        Some(dimensions.css.canvas.height)
    );
    set(&canvas, "width", &1.into());
    set(
        &live.terminal.options().into(),
        "fontFamily",
        &"monospace".into(),
    );
    assert_eq!(get(&canvas, "width").as_f64(), Some(1.0));
    owner.dispose().unwrap();
    owner.dispose().unwrap();
    assert!(get(&canvas, "parentElement").is_null());
    // Externally retained canvas operations still work after explicit disposal,
    // as upstream methods do. Only terminal subscriptions have been detached.
    renderer.insert_layer(ImageLayer::Top).unwrap();
    let after = renderer.canvas(ImageLayer::Top).unwrap().unwrap();
    set(&after, "width", &1.into());
    set(&live.terminal.options().into(), "fontSize", &23.into());
    assert_eq!(get(&after, "width").as_f64(), Some(1.0));
    let service = live.terminal.render_service_handle().unwrap();
    service
        .set_renderer(service.current_renderer().unwrap())
        .unwrap();
    assert!(renderer.has_layer(ImageLayer::Top));
    renderer.dispose().unwrap();
}

#[wasm_bindgen_test(async)]
async fn placeholder_toggle_requests_real_terminal_renders_and_drop_unhooks() {
    let live = Live::new();
    live.open();
    let owner = ImageRendererLifecycle::new(&live.terminal).unwrap();
    let renderer = owner.renderer();
    renderer.insert_layer(ImageLayer::Top).unwrap();
    frame().await;
    let count = Rc::new(Cell::new(0));
    let observed = Rc::clone(&count);
    let service = live.terminal.render_service_handle().unwrap();
    let listener = service
        .on_render()
        .subscribe(move |_| observed.set(observed.get() + 1));
    owner.show_placeholder(true).unwrap();
    frame().await;
    assert!(count.get() > 0);
    count.set(0);
    owner.show_placeholder(false).unwrap();
    frame().await;
    assert!(count.get() > 0);
    let canvas = renderer.canvas(ImageLayer::Top).unwrap().unwrap();
    drop(owner);
    assert!(get(&canvas, "parentElement").is_null());
    renderer.insert_layer(ImageLayer::Top).unwrap();
    service
        .set_renderer(service.current_renderer().unwrap())
        .unwrap();
    assert!(renderer.has_layer(ImageLayer::Top));
    listener.dispose();
    renderer.dispose().unwrap();
}

#[wasm_bindgen_test]
fn replacement_removes_snapshot_in_insertion_order_before_backend_and_preserves_throw() {
    let live = Live::new();
    live.open();
    let owner = ImageRendererLifecycle::new(&live.terminal).unwrap();
    let renderer = owner.renderer();
    renderer.insert_layer(ImageLayer::Bottom).unwrap();
    renderer.insert_layer(ImageLayer::Top).unwrap();
    let bottom = renderer.canvas(ImageLayer::Bottom).unwrap().unwrap();
    let top = renderer.canvas(ImageLayer::Top).unwrap().unwrap();
    let token = eval("({identity:'remove failure'})");
    set(
        &top,
        "remove",
        &eval("(function(){ throw globalThis.imageRemoveToken; })"),
    );
    set(&js_sys::global().into(), "imageRemoveToken", &token);
    let service = live.terminal.render_service_handle().unwrap();
    let backend = service.current_renderer().unwrap();
    let error = service.set_renderer(Rc::clone(&backend)).unwrap_err();
    assert!(error == token);
    assert!(!renderer.has_layer(ImageLayer::Bottom));
    assert!(get(&bottom, "parentElement").is_null());
    assert!(renderer.has_layer(ImageLayer::Top));
    assert!(Rc::ptr_eq(&backend, &service.current_renderer().unwrap()));
    js_sys::Reflect::delete_property(
        &top.clone().unchecked_into::<js_sys::Object>(),
        &"remove".into(),
    )
    .unwrap();
    service.set_renderer(backend).unwrap();
    assert!(!renderer.has_layer(ImageLayer::Top));
    owner.dispose().unwrap();
}

#[wasm_bindgen_test]
fn newest_interceptor_runs_first_and_errors_stop_older_wrappers() {
    let live = Live::new();
    live.open();
    let service = live.terminal.render_service_handle().unwrap();
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    let first = service.intercept_renderer_change(move || {
        observed.set(observed.get() + 1);
        Ok(RendererChangeAction::Continue)
    });
    let token = eval("({identity:'intercept failure'})");
    let thrown = token.clone();
    let second = service.intercept_renderer_change(move || Err(thrown.clone()));
    assert!(
        service
            .set_renderer(service.current_renderer().unwrap())
            .unwrap_err()
            == token
    );
    assert_eq!(calls.get(), 0);
    second.dispose();
    service
        .set_renderer(service.current_renderer().unwrap())
        .unwrap();
    assert_eq!(calls.get(), 1);
    first.dispose();
}

#[wasm_bindgen_test]
fn source_lifecycle_traces_match_font_filter_and_replacement_failure_order() {
    let source = eval(
        r#"(() => {
        const trace = [];
        const service = {setRenderer(value){trace.push(['backend', value])}, refreshRows(a,b){trace.push(['refresh',a,b])}};
        const terminal = {rows:4, _core:{open(){trace.push(['open'])}, _renderService:service,
            optionsService:{onOptionChange(fn){terminal.change=fn; return {dispose(){trace.push(['unsubscribe'])}}}}}};
        const owner = new SourceImageRenderer(terminal);
        terminal._core.open(null);
        owner.rescaleCanvas=()=>trace.push(['rescale']);
        terminal.change('fontFamily'); terminal.change('fontSize');
        owner.showPlaceholder(false);
        owner._layers.set('bottom',{canvas:{remove(){trace.push(['remove','bottom'])}}});
        owner._layers.set('top',{canvas:{remove(){trace.push(['remove','top'])}}});
        service.setRenderer('next');
        owner.dispose(); terminal._core.open(null); service.setRenderer('restored');
        return trace;
    })()"#,
    );
    assert_eq!(
        js_sys::JSON::stringify(&source)
            .unwrap()
            .as_string()
            .unwrap(),
        r#"[["open"],["rescale"],["refresh",0,4],["refresh",0,4],["remove","bottom"],["remove","top"],["backend","next"],["unsubscribe"],["open"],["backend","restored"]]"#
    );
}
