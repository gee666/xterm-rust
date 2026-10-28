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

#[wasm_bindgen_test]
fn failed_disposal_then_owner_drop_cannot_leave_a_dead_renderer_wrapper() {
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
    set(
        &canvas,
        "remove",
        &eval("(function(){throw 'drop regression'})"),
    );
    assert!(owner.dispose().is_err());
    drop(owner);
    // Dropping a failed owner does not retry removal or hide the partial map.
    assert!(renderer.has_layer(ImageLayer::Top));
    js_sys::Reflect::delete_property(&canvas.unchecked_into::<js_sys::Object>(), &"remove".into())
        .unwrap();
    service
        .set_renderer(service.current_renderer().unwrap())
        .unwrap();
    assert_eq!(count.get(), 1);
    renderer.dispose().unwrap();
    backend.dispose();
    let source = eval(
        r#"(() => {
        let calls=0;
        const terminal={_core:{open(){},screenElement:{},optionsService:{onOptionChange(){return {dispose(){}}}},
            _renderService:{setRenderer(){calls++}}}};
        let owner=new SourceImageRenderer(terminal);
        const canvas={remove(){throw 'drop regression'}};
        owner._layers.set('top',{canvas});
        try{owner.dispose()}catch(error){if(error!=='drop regression')throw error}
        owner=undefined; canvas.remove=()=>{};
        terminal._core._renderService.setRenderer(null);
        return calls;
    })()"#,
    );
    assert_eq!(source.as_f64(), Some(1.0));
}

#[wasm_bindgen_test]
fn disposal_final_map_clear_forgets_layer_reinserted_by_bottom_removal() {
    let live = Live::new();
    live.open();
    let owner = ImageRendererLifecycle::new(&live.terminal).unwrap();
    let renderer = owner.renderer();
    renderer.insert_layer(ImageLayer::Top).unwrap();
    renderer.insert_layer(ImageLayer::Bottom).unwrap();
    let bottom = renderer.canvas(ImageLayer::Bottom).unwrap().unwrap();
    let observed = Rc::clone(&renderer);
    let insert = Closure::<dyn Fn()>::new(move || observed.insert_layer(ImageLayer::Top).unwrap());
    let wrap = js_sys::Function::new_with_args(
        "insert",
        "return function(){delete this.remove;insert();this.remove()}",
    );
    set(
        &bottom,
        "remove",
        &wrap.call1(&JsValue::NULL, insert.as_ref()).unwrap(),
    );
    owner.dispose().unwrap();
    assert!(!renderer.has_layer(ImageLayer::Top));
    assert!(!renderer.has_layer(ImageLayer::Bottom));
    // Source clears registration without revisiting the newly inserted DOM node.
    let screen = live.terminal.screen_element().unwrap();
    assert_eq!(
        screen
            .query_selector_all(".xterm-image-layer-top")
            .unwrap()
            .length(),
        1
    );
    let source = eval(
        r#"(() => {
        const terminal={_core:{open(){},screenElement:{},optionsService:{onOptionChange(){return {dispose(){}}}},
            _renderService:{setRenderer(){}}}};
        const owner=new SourceImageRenderer(terminal);
        owner._layers.set('bottom',{canvas:{remove(){owner._layers.set('top',{canvas:{remove(){}}})}}});
        owner.dispose();return owner.hasLayer('top');
    })()"#,
    );
    assert_eq!(source.as_bool(), Some(false));
}
