// Copyright (c) 2020 The xterm.js authors. MIT license.
use crate::support::{call, eval, get};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_web::image::ImageAddon;

fn json(value: &JsValue) -> String {
    js_sys::JSON::stringify(value).unwrap().as_string().unwrap()
}

fn failing_options(mode: &str, thrown: &str) -> JsValue {
    js_sys::Function::new_with_args(
        "mode,expected",
        r#"
        const trace=[], extra=Symbol('extra'); let pass=0;
        const target={
          pixelLimit: { valueOf() { trace.push('coerce'); throw Error('premature coercion'); } },
          sixelSupport: false,
          [extra]: 7
        };
        Object.defineProperty(target, 'hidden', {
          get() { throw Error('nonenumerable getter must not run'); }
        });
        const options=new Proxy(target, {
          ownKeys(target) {
            trace.push(`keys:${++pass}`);
            if(mode==='keys' && pass===2) throw expected;
            return Reflect.ownKeys(target);
          },
          getOwnPropertyDescriptor(target,key) {
            trace.push(`descriptor:${pass}:${String(key)}`);
            if(mode==='descriptor' && pass===2 && key==='sixelSupport') throw expected;
            return Reflect.getOwnPropertyDescriptor(target,key);
          },
          get(target,key,receiver) {
            trace.push(`get:${pass}:${String(key)}`);
            if(pass===2 && (mode==='getter' && key==='pixelLimit' || mode==='symbol' && key===extra)) throw expected;
            return Reflect.get(target,key,receiver);
          }
        });
        return { options, trace, expected };
        "#,
    )
    .call2(&JsValue::UNDEFINED, &mode.into(), &eval(thrown))
    .unwrap()
}

#[wasm_bindgen_test]
fn second_option_copy_exceptions_precede_numeric_coercion() {
    for mode in ["keys", "descriptor", "getter", "symbol"] {
        for thrown in [
            "({sentinel:true})",
            "undefined",
            "null",
            "0",
            "Symbol('failure')",
        ] {
            let source = failing_options(mode, thrown);
            let error = js_sys::Function::new_with_args(
                "fixture",
                "try { new SourceImageAddon(fixture.options); throw Error('missing failure'); } catch(error) { return error; }",
            )
            .call1(&JsValue::UNDEFINED, &source)
            .unwrap();
            assert_eq!(error, get(&source, "expected"), "source {mode}/{thrown}");
            let expected_trace = json(&get(&source, "trace"));
            assert!(!expected_trace.contains("coerce"));

            let rust = failing_options(mode, thrown);
            let error = ImageAddon::new(get(&rust, "options"))
                .err()
                .expect("second option copy must fail");
            assert_eq!(error, get(&rust, "expected"), "Rust {mode}/{thrown}");
            assert_eq!(
                json(&get(&rust, "trace")),
                expected_trace,
                "{mode}/{thrown}"
            );
        }
    }
}

fn successful_options() -> JsValue {
    eval(
        r#"(()=>{
          const trace=[]; let pass=0;
          const options=new Proxy({pixelLimit:0,sixelPaletteLimit:0,sixelSupport:false}, {
            ownKeys(target) { trace.push(`keys:${++pass}`); return Reflect.ownKeys(target); },
            getOwnPropertyDescriptor(target,key) {
              trace.push(`descriptor:${pass}:${key}`);
              return Reflect.getOwnPropertyDescriptor(target,key);
            },
            get(target,key) {
              trace.push(`get:${pass}:${key}`);
              return key==='pixelLimit' ? 1024*pass : key==='sixelPaletteLimit' ? 128*pass : false;
            }
          });
          return {options,trace};
        })()"#,
    )
}

#[wasm_bindgen_test]
fn option_copy_passes_keep_current_and_reset_defaults_distinct() {
    let rust = successful_options();
    let addon = ImageAddon::new(get(&rust, "options")).unwrap();
    let source = successful_options();
    let original =
        js_sys::Function::new_with_args("options", "return new SourceImageAddon(options)")
            .call1(&JsValue::UNDEFINED, &get(&source, "options"))
            .unwrap();
    assert_eq!(json(&get(&rust, "trace")), json(&get(&source, "trace")));
    assert_eq!(
        get(&addon.option_inspection(), "pixelLimit"),
        JsValue::from(1024)
    );
    assert_eq!(
        get(&addon.option_inspection(), "sixelPaletteLimit"),
        JsValue::from(128)
    );
    assert_eq!(
        json(&addon.option_inspection()),
        json(&get(&original, "_opts"))
    );
    addon.reset().unwrap();
    call(&original, "reset", &[]);
    assert_eq!(
        get(&addon.option_inspection(), "sixelPaletteLimit"),
        JsValue::from(256)
    );
    assert_eq!(
        json(&addon.option_inspection()),
        json(&get(&original, "_opts"))
    );
    // Reset consumes the copied defaults, not the caller's getters again.
    assert_eq!(json(&get(&rust, "trace")), json(&get(&source, "trace")));
    addon.dispose().unwrap();
    call(&original, "dispose", &[]);
}
