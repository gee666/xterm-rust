use super::super::*;
use wasm_bindgen::closure::Closure;

#[wasm_bindgen_test]
fn disposal_during_layer_removal_clears_saved_delegate_before_backend_call() {
    let live = Live::new();
    live.open();
    let service = live.terminal.render_service_handle().unwrap();
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    let backend = service.intercept_renderer_change(move || {
        observed.set(observed.get() + 1);
        Ok(RendererChangeAction::Continue)
    });
    let owner = Rc::new(ImageRendererLifecycle::new(&live.terminal).unwrap());
    let renderer = owner.renderer();
    renderer.insert_layer(ImageLayer::Top).unwrap();
    let canvas = renderer.canvas(ImageLayer::Top).unwrap().unwrap();
    let weak = Rc::downgrade(&owner);
    let dispose = Closure::<dyn Fn()>::new(move || weak.upgrade().unwrap().dispose().unwrap());
    let wrap = js_sys::Function::new_with_args(
        "dispose",
        "return function(){delete this.remove; dispose(); this.remove();}",
    );
    set(
        &canvas,
        "remove",
        &wrap.call1(&JsValue::NULL, dispose.as_ref()).unwrap(),
    );
    service
        .set_renderer(service.current_renderer().unwrap())
        .unwrap();
    assert_eq!(calls.get(), 0);
    assert!(!renderer.has_layer(ImageLayer::Top));
    service
        .set_renderer(service.current_renderer().unwrap())
        .unwrap();
    assert_eq!(calls.get(), 1);
    backend.dispose();
    let source = eval(
        r#"(() => {
        let calls=0;
        const terminal={_core:{open(){},screenElement:{}, optionsService:{onOptionChange(){return {dispose(){}}}},
            _renderService:{setRenderer(){calls++}}}};
        const owner=new SourceImageRenderer(terminal);
        owner._layers.set('top',{canvas:{remove(){owner._layers.clear();owner.dispose()}}});
        terminal._core._renderService.setRenderer(null);
        return calls;
    })()"#,
    );
    assert_eq!(source.as_f64(), Some(0.0));
}

#[wasm_bindgen_test]
fn failed_disposal_keeps_saved_delegate_and_is_not_silently_retried() {
    let live = Live::new();
    live.open();
    let service = live.terminal.render_service_handle().unwrap();
    let count = Rc::new(Cell::new(0));
    let observed = Rc::clone(&count);
    let backend = service.intercept_renderer_change(move || {
        observed.set(observed.get() + 1);
        Ok(RendererChangeAction::Continue)
    });
    let owner = ImageRendererLifecycle::new(&live.terminal).unwrap();
    let renderer = owner.renderer();
    renderer.insert_layer(ImageLayer::Top).unwrap();
    let canvas = renderer.canvas(ImageLayer::Top).unwrap().unwrap();
    let token = eval("({identity:'dispose error'})");
    set(&js_sys::global().into(), "imageDisposeToken", &token);
    set(
        &canvas,
        "remove",
        &eval("(function(){throw globalThis.imageDisposeToken})"),
    );
    assert!(owner.dispose().unwrap_err() == token);
    owner.dispose().unwrap();
    assert!(renderer.has_layer(ImageLayer::Top));
    js_sys::Reflect::delete_property(
        &canvas.clone().unchecked_into::<js_sys::Object>(),
        &"remove".into(),
    )
    .unwrap();
    service
        .set_renderer(service.current_renderer().unwrap())
        .unwrap();
    assert!(!renderer.has_layer(ImageLayer::Top));
    assert_eq!(count.get(), 1);
    backend.dispose();
}
