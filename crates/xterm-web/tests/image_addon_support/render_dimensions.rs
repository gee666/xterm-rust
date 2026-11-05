// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::support::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

fn service(pair: &Pair) -> JsValue {
    get(&pair.terminal.core_services(), "_renderService")
}
fn source_service(pair: &Pair) -> JsValue {
    get(
        &get(&get(&pair.source, "terminal"), "_core"),
        "_renderService",
    )
}
fn dimensions(service: &JsValue) -> JsValue {
    get(service, "dimensions")
}
fn json(value: &JsValue) -> String {
    js_sys::JSON::stringify(value).unwrap().as_string().unwrap()
}
fn compare(pair: &Pair, actual: &JsValue, expected: &JsValue) {
    assert_eq!(json(actual), json(expected));
    assert_eq!(json(actual), json(&pair.terminal.dimensions().unwrap()));
    let native = pair
        .terminal
        .render_service_handle()
        .unwrap()
        .dimensions()
        .unwrap();
    assert!(!get(&get(actual, "css"), "cell").is_undefined());
    assert_eq!(
        get(&get(&get(actual, "css"), "cell"), "width").as_f64(),
        Some(native.css.cell.width)
    );
}

#[wasm_bindgen_test(async)]
async fn internal_dimensions_are_absent_before_open_and_live_after_open() {
    let pair = Pair::new(
        false,
        "({sixelSupport:false,iipSupport:false,kittySupport:false})",
    );
    assert!(service(&pair).is_undefined());
    assert!(source_service(&pair).is_undefined());
    let document = web_sys::window().unwrap().document().unwrap();
    let parent = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
    pair.terminal.open(parent.clone().into()).unwrap();
    call(
        &get(&pair.source, "terminal"),
        "open",
        &[get(&pair.source, "parent")],
    );
    pair.frame().await;
    let render = service(&pair);
    let actual = dimensions(&render);
    let expected = dimensions(&source_service(&pair));
    assert_eq!(render, service(&pair));
    assert_eq!(actual, dimensions(&render));
    for group in ["css", "device"] {
        let part = get(&actual, group);
        assert_eq!(part, get(&dimensions(&render), group));
        for kind in ["canvas", "cell"] {
            assert_eq!(
                get(&part, kind),
                get(&get(&dimensions(&render), group), kind)
            );
        }
    }
    let initial = json(&actual);
    compare(&pair, &actual, &expected);
    for (cols, rows, size) in [(31, 7, 18), (9, 16, 11), (20, 10, 14)] {
        pair.terminal.resize(cols as f64, rows as f64).unwrap();
        call(
            &get(&pair.source, "terminal"),
            "resize",
            &[cols.into(), rows.into()],
        );
        js_sys::Reflect::set(&pair.terminal.options(), &"fontSize".into(), &size.into()).unwrap();
        js_sys::Reflect::set(
            &get(&get(&pair.source, "terminal"), "options"),
            &"fontSize".into(),
            &size.into(),
        )
        .unwrap();
        pair.frame().await;
        assert_eq!(actual, dimensions(&render));
        assert_eq!(expected, dimensions(&source_service(&pair)));
        compare(&pair, &actual, &expected);
    }
    assert_eq!(initial, json(&actual));
    pair.terminal.reset().unwrap();
    call(&get(&pair.source, "terminal"), "reset", &[]);
    compare(&pair, &actual, &expected);
    pair.terminal.dispose().unwrap();
    call(&get(&pair.source, "terminal"), "dispose", &[]);
    assert_eq!(render, service(&pair));
    assert!(js_sys::Reflect::get(&render, &"dimensions".into()).is_err());
    assert!(js_sys::Reflect::get(&source_service(&pair), &"dimensions".into()).is_err());
    assert_eq!(json(&actual), json(&expected));
    parent.remove();
}

#[wasm_bindgen_test(async)]
async fn renderer_replacement_changes_dimension_identity_and_retains_old_views() {
    let first = Pair::new(
        true,
        "({sixelSupport:false,iipSupport:false,kittySupport:false})",
    );
    let second = Pair::new(
        true,
        "({sixelSupport:false,iipSupport:false,kittySupport:false})",
    );
    second.terminal.resize(33.0, 8.0).unwrap();
    call(
        &get(&second.source, "terminal"),
        "resize",
        &[33.into(), 8.into()],
    );
    first.frame().await;
    let old = dimensions(&service(&first));
    let old_source = dimensions(&source_service(&first));
    let before = json(&old);
    let second_renderer = second
        .terminal
        .render_service_handle()
        .unwrap()
        .current_renderer()
        .unwrap();
    first
        .terminal
        .render_service_handle()
        .unwrap()
        .set_renderer(second_renderer)
        .unwrap();
    let replace = js_sys::Function::new_with_args(
        "first,second",
        "first.setRenderer(second._renderer.value)",
    );
    replace
        .call2(
            &JsValue::UNDEFINED,
            &source_service(&first),
            &source_service(&second),
        )
        .unwrap();
    first.frame().await;
    let current = dimensions(&service(&first));
    assert_ne!(old, current);
    assert_ne!(old_source, dimensions(&source_service(&first)));
    compare(&first, &current, &dimensions(&source_service(&first)));
    assert_eq!(before, json(&old));
    assert_eq!(json(&old), json(&old_source));
    assert!(!js_sys::Reflect::set(
        &get(&get(&current, "css"), "cell"),
        &"width".into(),
        &1.into()
    )
    .unwrap());
    assert!(!js_sys::Reflect::set(&current, &"extra".into(), &1.into()).unwrap());
    compare(&first, &current, &dimensions(&source_service(&first)));
}
