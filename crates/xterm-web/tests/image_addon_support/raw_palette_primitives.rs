// Copyright (c) 2020, 2023 The xterm.js authors. MIT license.
use crate::{
    raw_kitty_limit::json,
    raw_palette::{palette_pair, value},
    support::*,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test(async)]
async fn raw_palette_ready_primitives_and_undefined_default_use_constructor_fallback() {
    for expression in [
        "undefined",
        "null",
        "false",
        "true",
        "0",
        "-0",
        "NaN",
        "-Infinity",
        "-1",
        "''",
        "'32'",
        "'no'",
        "[]",
        "new Number(32)",
    ] {
        let pair = palette_pair(&format!(
            "({{showPlaceholder:false,sixelPaletteLimit:{expression}}})"
        ));
        pair.ready().await;
        for text in [
            sixel(),
            "\x1b[?1;1S",
            "\x1b[?1;3;16S",
            sixel(),
            "\x1b[?1;2S",
        ] {
            pair.write(text).await;
            pair.compare();
        }
        if get(&pair.events, "count").as_f64().unwrap() > 0.0 {
            pair.compare_image(0.0, 0.0);
        }
    }
    let pair = palette_pair(
        r#"(()=>{
      let reads=0;return {showPlaceholder:false,get sixelPaletteLimit(){
        return ++reads===1?{log:[],[Symbol.toPrimitive](hint){this.log.push(hint);return 16}}:undefined
      }};
    })()"#,
    );
    let initial = value(&pair, true);
    let source_initial = value(&pair, false);
    pair.ready().await;
    assert_eq!(json(&get(&initial, "log")), "[\"number\",\"number\"]");
    pair.addon.reset().unwrap();
    call(&get(&pair.source, "addon"), "reset", &[]);
    assert!(value(&pair, true).is_undefined());
    assert!(value(&pair, false).is_undefined());
    assert_eq!(
        json(&get(&initial, "log")),
        "[\"number\",\"number\",\"number\",\"number\"]"
    );
    pair.write(sixel()).await;
    pair.compare();
    pair.compare_image(0.0, 0.0);
    assert_eq!(
        json(&get(&initial, "log")),
        json(&get(&source_initial, "log"))
    );
    assert_eq!(get(&pair.events, "count"), JsValue::from(1));
}
