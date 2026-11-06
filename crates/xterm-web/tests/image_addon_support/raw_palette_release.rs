// Copyright (c) 2020, 2023 The xterm.js authors. MIT license.
use crate::{
    raw_kitty_limit::json,
    raw_palette::{palette_pair, set, value},
    raw_sixel_limit::parse,
    support::*,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test(async)]
async fn raw_palette_memory_and_abort_releases_preserve_warning_and_throw_order() {
    // Memory rejection releases inside the decoder and again after the warning.
    // A size rejection only releases after the warning.
    for (size, fail_at) in [(false, 0), (false, 4), (false, 5), (true, 0), (true, 4)] {
        let pair = palette_pair(&format!(
            r#"({{showPlaceholder:false,pixelLimit:{},
          sixelSizeLimit:{},sixelPaletteLimit:{{
            log:[],failAt:0,token:{{toString(){{return 'release token'}}}},
            [Symbol.toPrimitive](hint){{this.log.push(hint);
              if(this.log.length===this.failAt)throw this.token;return 32}}
          }}
        }})"#,
            if size { 16777216 } else { 1 },
            if size { 1 } else { 33554432 }
        ));
        pair.ready().await;
        for rust in [true, false] {
            set(&value(&pair, rust), "failAt", &JsValue::from(fail_at));
        }
        let patch = eval("(()=>{const old=console.warn,log=[];console.warn=(...args)=>log.push(args);return {log,restore(){console.warn=old}}})()");
        let actual = pair
            .terminal
            .shared_native()
            .write(sixel())
            .map_err(|error| xterm_web::js_error::to_js_error(&error));
        let actual_log = json(&get(&patch, "log"));
        call(&get(&patch, "log"), "splice", &[0.into()]);
        let expected =
            js_sys::Function::new_with_args("t,text", "return t._core._inputHandler.parse(text)")
                .call2(
                    &JsValue::UNDEFINED,
                    &get(&pair.source, "terminal"),
                    &sixel().into(),
                );
        let expected_log = json(&get(&patch, "log"));
        call(&patch, "restore", &[]);
        assert_eq!(actual_log, expected_log, "size={size} fail={fail_at}");
        assert_eq!(actual.is_ok(), expected.is_ok());
        if let (Err(actual), Err(expected)) = (actual, expected) {
            assert_eq!(actual, get(&value(&pair, true), "token"));
            assert_eq!(expected, get(&value(&pair, false), "token"));
        }
        assert_eq!(
            json(&get(&value(&pair, true), "log")),
            json(&get(&value(&pair, false), "log"))
        );
        pair.compare();
    }
}

#[wasm_bindgen_test(async)]
async fn raw_palette_large_image_copy_releases_after_upload_and_keeps_pixels() {
    let pair = palette_pair(
        r#"({showPlaceholder:false,sixelPaletteLimit:{
      log:[],[Symbol.toPrimitive](hint){this.log.push(hint);return 32}
    }})"#,
    );
    pair.ready().await;
    let (actual, expected) = parse(&pair, "\x1bP0;1q\"1;1;1024;1024#1!1024~\x1b\\");
    actual.unwrap();
    expected.unwrap();
    pair.compare();
    // Copy precedes the >4MiB release, so the uploaded pixels survive.
    pair.compare_image(0.0, 0.0);
    assert_eq!(
        json(&get(&value(&pair, true), "log")),
        "[\"number\",\"number\",\"number\",\"number\"]"
    );
    assert_eq!(
        json(&get(&value(&pair, true), "log")),
        json(&get(&value(&pair, false), "log"))
    );
}
