// Copyright (c) 2020, 2023 The xterm.js authors. MIT license.
use crate::support::*;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_web::{image::ImageAddon, js_error::to_js_error};

fn values() -> js_sys::Array {
    eval("[undefined,null,false,true,0,-0,NaN,Infinity,-Infinity,1,-1,'','1000','no',0n,1000n,Symbol('size'),{},[],new Number(1000)]").unchecked_into()
}
fn pair(open: bool, options: &str) -> Pair {
    // Check the constructor before allocating a terminal. Expected old-code
    // failures must not leave an opened terminal or pending renderer callbacks.
    let input = eval(options);
    let addon = ImageAddon::new(input.clone()).unwrap();
    let actual = get(&addon.option_inspection(), "sixelSizeLimit");
    addon.dispose().unwrap();
    assert!(js_sys::Object::is(&actual, &get(&input, "sixelSizeLimit")));
    Pair::new(open, options)
}
fn json(value: &JsValue) -> String {
    js_sys::JSON::stringify(value).unwrap().as_string().unwrap()
}
fn limit(pair: &Pair, rust: bool) -> JsValue {
    let opts = if rust {
        pair.addon.option_inspection()
    } else {
        get(&get(&pair.source, "addon"), "_opts")
    };
    get(&opts, "sixelSizeLimit")
}
fn source_parse(pair: &Pair, text: &str) -> Result<JsValue, JsValue> {
    js_sys::Function::new_with_args("t,text", "return t._core._inputHandler.parse(text)").call2(
        &JsValue::UNDEFINED,
        &get(&pair.source, "terminal"),
        &text.into(),
    )
}
fn parse(pair: &Pair, text: &str) -> (Result<(), JsValue>, Result<JsValue, JsValue>) {
    let actual = pair
        .terminal
        .shared_native()
        .write(text)
        .map_err(|e| to_js_error(&e));
    let expected = source_parse(pair, text);
    (actual, expected)
}

#[wasm_bindgen_test]
fn raw_sixel_limit_constructor_keeps_first_copy_without_coercion() {
    for value in values() {
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"sixelSizeLimit".into(), &value).unwrap();
        let actual = ImageAddon::new(options.clone().into()).unwrap();
        let source = js_sys::Function::new_with_args("opts", "return new SourceImageAddon(opts)")
            .call1(&JsValue::UNDEFINED, &options)
            .unwrap();
        assert!(js_sys::Object::is(
            &get(&actual.option_inspection(), "sixelSizeLimit"),
            &value
        ));
        assert!(js_sys::Object::is(
            &get(&get(&source, "_opts"), "sixelSizeLimit"),
            &value
        ));
        actual.reset().unwrap();
        actual.dispose().unwrap();
        assert!(js_sys::Object::is(
            &get(&actual.option_inspection(), "sixelSizeLimit"),
            &value
        ));
        call(&source, "dispose", &[]);
    }
    for rust in [false, true] {
        let input = eval("(()=>{let reads=0;const first={[Symbol.toPrimitive](){throw Error('eager conversion')}},second={};return {first,second,get reads(){return reads},options:{get sixelSizeLimit(){return ++reads===1?first:second}}}})()");
        let options = get(&input, "options");
        let addon: JsValue = if rust {
            ImageAddon::new(options).unwrap().into()
        } else {
            js_sys::Function::new_with_args("opts", "return new SourceImageAddon(opts)")
                .call1(&JsValue::UNDEFINED, &options)
                .unwrap()
        };
        assert_eq!(get(&input, "reads"), JsValue::from(2));
        assert_eq!(
            get(&get(&addon, "_opts"), "sixelSizeLimit"),
            get(&input, "first")
        );
        call(&addon, "reset", &[]);
        call(&addon, "dispose", &[]);
    }
}

#[wasm_bindgen_test(async)]
async fn raw_sixel_limit_skips_disabled_pending_and_aborted_packets() {
    let disabled = pair(false, "({sixelSupport:false,showPlaceholder:false,sixelSizeLimit:{[Symbol.toPrimitive](){throw Error('disabled')}}})");
    disabled.write(sixel()).await;
    disabled.compare();
    let pair = pair(false, "({showPlaceholder:false,sixelSizeLimit:{log:[],[Symbol.toPrimitive](hint){this.log.push(hint);return -1}}})");
    // Both initialization promises are still pending. This must not coerce.
    let (actual, expected) = parse(&pair, "\x1bP0;1q~\x1b\\");
    actual.unwrap();
    expected.unwrap();
    for rust in [false, true] {
        assert_eq!(json(&get(&limit(&pair, rust), "log")), "[]");
    }
    pair.ready().await;
    let patch = eval(
        "(()=>{const old=console.warn;console.warn=()=>{};return {restore(){console.warn=old}}})()",
    );
    let (actual, expected) = parse(&pair, "\x1bP0;1q~");
    actual.unwrap();
    expected.unwrap();
    let (actual, expected) = parse(&pair, "~~\x1b\\");
    call(&patch, "restore", &[]);
    actual.unwrap();
    expected.unwrap();
    for rust in [false, true] {
        assert_eq!(json(&get(&limit(&pair, rust), "log")), "[\"number\"]");
    }
    pair.compare();
    pair.addon.dispose().unwrap();
    call(&get(&pair.source, "addon"), "dispose", &[]);
    let (actual, expected) = parse(&pair, sixel());
    actual.unwrap();
    expected.unwrap();
    for rust in [false, true] {
        assert_eq!(json(&get(&limit(&pair, rust), "log")), "[\"number\"]");
    }
}

#[wasm_bindgen_test(async)]
async fn raw_sixel_limit_relational_primitives_match_real_image_pixels() {
    for expression in [
        "undefined",
        "null",
        "false",
        "true",
        "0",
        "-0",
        "NaN",
        "Infinity",
        "-Infinity",
        "-1",
        "''",
        "'1000'",
        "'no'",
        "0n",
        "1000n",
        "Symbol('size')",
        "{}",
        "[]",
        "new Number(1000)",
    ] {
        let pair = pair(
            true,
            &format!("({{showPlaceholder:false,sixelSizeLimit:{expression}}})"),
        );
        pair.ready().await;
        let patch = eval("(()=>{const old=console.warn,log=[];console.warn=(...args)=>log.push(args);return {log,restore(){console.warn=old}}})()");
        let actual = pair
            .terminal
            .shared_native()
            .write(sixel())
            .map_err(|e| to_js_error(&e));
        let actual_log = json(&get(&patch, "log"));
        call(&get(&patch, "log"), "splice", &[0.into()]);
        let expected = source_parse(&pair, sixel());
        let expected_log = json(&get(&patch, "log"));
        call(&patch, "restore", &[]);
        assert_eq!(actual_log, expected_log, "value={expression}");
        assert_eq!(actual.is_ok(), expected.is_ok(), "value={expression}");
        if let (Err(actual), Err(expected)) = (actual, expected) {
            assert_eq!(get(&actual, "name"), get(&expected, "name"));
            assert_eq!(get(&actual, "message"), get(&expected, "message"));
        }
        pair.compare();
        if get(&pair.events, "count") == 1 {
            pair.compare_image(0.0, 0.0);
        }
    }
}

#[wasm_bindgen_test(async)]
async fn raw_sixel_limit_throw_keeps_committed_size_and_exact_value() {
    let pair = pair(true, "({showPlaceholder:false,sixelSizeLimit:{log:[],token:{},[Symbol.toPrimitive](hint){this.log.push(hint);throw this.token}}})");
    pair.ready().await;
    let (actual, expected) = parse(&pair, "\x1bP0;1q~");
    assert_eq!(actual.unwrap_err(), get(&limit(&pair, true), "token"));
    assert_eq!(expected.unwrap_err(), get(&limit(&pair, false), "token"));
    for rust in [false, true] {
        assert_eq!(json(&get(&limit(&pair, rust), "log")), "[\"number\"]");
    }
    // The unchanged source handler confirms the increment survives the throw.
    let source_handler = call(
        &get(&get(&pair.source, "addon"), "_handlers"),
        "get",
        &["sixel".into()],
    );
    assert_eq!(get(&source_handler, "_size"), JsValue::from(1));
    pair.compare();
    for rust in [false, true] {
        js_sys::Reflect::set(
            &limit(&pair, rust),
            &js_sys::Symbol::to_primitive(),
            &js_sys::Function::new_no_args("return 1"),
        )
        .unwrap();
    }
    // The second byte exceeds 1 only if the first increment survived the throw.
    let patch = eval("(()=>{const old=console.warn,log=[];console.warn=(...args)=>log.push(args);return {log,restore(){console.warn=old}}})()");
    let actual = pair.terminal.shared_native().write("~\x1b\\");
    let actual_log = json(&get(&patch, "log"));
    call(&get(&patch, "log"), "splice", &[0.into()]);
    let expected = source_parse(&pair, "~\x1b\\");
    let expected_log = json(&get(&patch, "log"));
    call(&patch, "restore", &[]);
    actual.unwrap();
    expected.unwrap();
    assert_eq!(actual_log, "[[\"SIXEL: too much data, aborting\"]]");
    assert_eq!(actual_log, expected_log);
    pair.compare();
}

#[wasm_bindgen_test(async)]
async fn raw_sixel_limit_coercion_reset_and_warning_failures_use_live_state() {
    for reset in [false, true] {
        let pair = Rc::new(pair(true, "({showPlaceholder:false,sixelSizeLimit:{log:[],callback:null,[Symbol.toPrimitive](hint){this.log.push(hint);this.callback?.();return 1000}}})"));
        pair.ready().await;
        let weak = Rc::downgrade(&pair);
        let callback = Closure::<dyn Fn()>::new(move || {
            if reset {
                weak.upgrade().unwrap().addon.reset().unwrap();
            }
        });
        js_sys::Reflect::set(&limit(&pair, true), &"callback".into(), callback.as_ref()).unwrap();
        let source_callback =
            js_sys::Function::new_with_args("addon,reset", "return ()=>{if(reset)addon.reset()}")
                .call2(
                    &JsValue::UNDEFINED,
                    &get(&pair.source, "addon"),
                    &reset.into(),
                )
                .unwrap();
        js_sys::Reflect::set(&limit(&pair, false), &"callback".into(), &source_callback).unwrap();
        for text in [
            "\x1bP0;1q\"1;1;14;12#1;2;100;0;0!14~",
            "-#2;2;0;100;0!14~\x1b\\",
        ] {
            let (actual, expected) = parse(&pair, text);
            actual.unwrap();
            expected.unwrap();
        }
        pair.compare();
        pair.compare_image(0.0, 0.0);
        assert_eq!(
            json(&get(&limit(&pair, true), "log")),
            json(&get(&limit(&pair, false), "log"))
        );
        js_sys::Reflect::set(&limit(&pair, true), &"callback".into(), &JsValue::NULL).unwrap();
    }
    let pair = pair(
        true,
        "({showPlaceholder:false,sixelSizeLimit:{[Symbol.toPrimitive](){return -1}}})",
    );
    pair.ready().await;
    let patch = eval("(()=>{const old=console.warn,token={};console.warn=()=>{throw token};return {token,restore(){console.warn=old}}})()");
    let (actual, expected) = parse(&pair, "\x1bP0;1q~");
    call(&patch, "restore", &[]);
    assert_eq!(actual.unwrap_err(), get(&patch, "token"));
    assert_eq!(expected.unwrap_err(), get(&patch, "token"));
    pair.compare();
}
