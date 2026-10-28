mod reentrancy;
use super::*;
fn pixels(canvas: &JsValue) -> Vec<u8> {
    let ctx = call(canvas, "getContext", &["2d".into()]);
    let data = call(
        &ctx,
        "getImageData",
        &[
            0.into(),
            0.into(),
            get(canvas, "width"),
            get(canvas, "height"),
        ],
    );
    js_sys::Uint8Array::new(&get(&get(&data, "data"), "buffer")).to_vec()
}

#[wasm_bindgen_test]
fn seeded_live_font_layer_replacement_and_placeholder_pixels_match_source() {
    let live = Live::new();
    live.open();
    let owner = ImageRendererLifecycle::new(&live.terminal).unwrap();
    let renderer = owner.renderer();
    let source = eval(
        r#"(() => {
        const document = globalThis.document;
        const terminal = {rows:4, _core:{open(){}, screenElement:document.createElement('div'),
            _coreBrowserService:{window:globalThis},
            _renderService:{setRenderer(){},refreshRows(){}},
            optionsService:{onOptionChange(fn){terminal.change=fn;return {dispose(){}}}}}};
        document.body.appendChild(terminal._core.screenElement);
        return new SourceImageRenderer(terminal);
    })()"#,
    );
    let terminal = get(&source, "_terminal");
    let service = live.terminal.render_service_handle().unwrap();
    set(
        &terminal,
        "dimensions",
        &live.terminal.dimensions().unwrap(),
    );
    let mut seed = 0x127389u32;
    for step in 0..64 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let layer = if seed & 1 == 0 {
            ImageLayer::Top
        } else {
            ImageLayer::Bottom
        };
        let name = if layer == ImageLayer::Top {
            "top"
        } else {
            "bottom"
        };
        match (seed >> 8) % 6 {
            0 => {
                renderer.insert_layer(layer).unwrap();
                call(&source, "insertLayerToDom", &[name.into()]);
            }
            1 => {
                renderer.remove_layer(layer).unwrap();
                call(&source, "removeLayerFromDom", &[name.into()]);
            }
            2 => {
                service
                    .set_renderer(service.current_renderer().unwrap())
                    .unwrap();
                call(
                    &get(&get(&terminal, "_core"), "_renderService"),
                    "setRenderer",
                    &[JsValue::NULL],
                );
            }
            3 => {
                let size = 12 + (seed >> 16) % 12;
                let old = get(&live.terminal.options().into(), "fontSize");
                set(&live.terminal.options().into(), "fontSize", &size.into());
                set(
                    &terminal,
                    "dimensions",
                    &live.terminal.dimensions().unwrap(),
                );
                if old != JsValue::from(size) {
                    call(&terminal, "change", &["fontSize".into()]);
                }
            }
            4 => {
                let show = seed & 2 == 0;
                owner.show_placeholder(show).unwrap();
                call(&source, "showPlaceholder", &[show.into()]);
            }
            _ => {
                renderer.draw_placeholder(0.0, 0.0, 2.0).unwrap();
                call(&source, "drawPlaceholder", &[0.into(), 0.into(), 2.into()]);
            }
        }
        for layer in [ImageLayer::Top, ImageLayer::Bottom] {
            let name = if layer == ImageLayer::Top {
                "top"
            } else {
                "bottom"
            };
            let context = call(&get(&source, "_layers"), "get", &[name.into()]);
            let rust = renderer.canvas(layer).unwrap();
            assert_eq!(
                rust.is_some(),
                !context.is_undefined(),
                "seed0x127389 step{step} layer{name}"
            );
            if let Some(rust) = rust {
                let canvas = get(&context, "canvas");
                assert_eq!(get(&rust, "width"), get(&canvas, "width"), "step{step}");
                assert_eq!(get(&rust, "height"), get(&canvas, "height"), "step{step}");
                assert_eq!(
                    pixels(&rust),
                    pixels(&canvas),
                    "seed0x127389 pixels step{step}"
                );
            }
        }
    }
    owner.dispose().unwrap();
    call(&source, "dispose", &[]);
    call(
        &get(&get(&terminal, "_core"), "screenElement"),
        "remove",
        &[],
    );
}

#[wasm_bindgen_test]
fn font_rescale_throw_retains_identity_and_committed_option() {
    let live = Live::new();
    live.open();
    let owner = ImageRendererLifecycle::new(&live.terminal).unwrap();
    let renderer = owner.renderer();
    renderer.insert_layer(ImageLayer::Top).unwrap();
    let canvas = renderer.canvas(ImageLayer::Top).unwrap().unwrap();
    let token = eval("({identity:'font rescale error'})");
    set(&js_sys::global().into(), "imageFontToken", &token);
    let descriptor =
        eval("({configurable:true,get(){return 1},set(){throw globalThis.imageFontToken}})");
    js_sys::Object::define_property(
        &canvas.clone().unchecked_into::<js_sys::Object>(),
        &"width".into(),
        &descriptor.unchecked_into(),
    );
    let options = JsValue::from(live.terminal.options());
    assert!(js_sys::Reflect::set(&options, &"fontSize".into(), &26.into()).unwrap_err() == token);
    assert_eq!(get(&options, "fontSize").as_f64(), Some(26.0));
    assert!(renderer.has_layer(ImageLayer::Top));
    js_sys::Reflect::delete_property(&canvas.unchecked_into::<js_sys::Object>(), &"width".into())
        .unwrap();
    owner.dispose().unwrap();
}

#[wasm_bindgen_test]
fn dropping_before_open_detaches_and_live_terminal_does_not_retain_owner() {
    let live = Live::new();
    let owner = ImageRendererLifecycle::new(&live.terminal).unwrap();
    let renderer = owner.renderer();
    let weak = Rc::downgrade(&renderer);
    drop(owner);
    drop(renderer);
    assert!(weak.upgrade().is_none());
    live.open();
    let service = live.terminal.render_service_handle().unwrap();
    service
        .set_renderer(service.current_renderer().unwrap())
        .unwrap();
}
