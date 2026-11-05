// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use crate::support::{call, eval, get};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_web::{image::ImageAddon, Terminal};

fn json(value: &JsValue) -> String {
    js_sys::JSON::stringify(value).unwrap().as_string().unwrap()
}
fn values() -> js_sys::Array {
    eval("[undefined,null,false,true,0,-0,NaN,Infinity,-1,0.5,1000,'','8',0n,2n,Symbol('limit'),{},[],[16],new Number(4),{valueOf(){throw Error('deferred')}}]").unchecked_into()
}
fn same(actual: &JsValue, expected: &JsValue) {
    assert!(js_sys::Object::is(actual, expected));
}
fn set(addon: &JsValue, value: &JsValue) {
    assert!(js_sys::Reflect::set(addon, &"storageLimit".into(), value).unwrap());
}
fn options(value: &JsValue) -> JsValue {
    let result = eval("({showPlaceholder:false,sixelSupport:false,iipSupport:false,kittySupport:false,enableSizeReports:false})");
    js_sys::Reflect::set(&result, &"storageLimit".into(), value).unwrap();
    result
}
struct Owned {
    addon: JsValue,
    terminal: JsValue,
}
impl Owned {
    fn new(rust: bool, options: &JsValue, active: bool) -> Self {
        let addon = if rust {
            ImageAddon::new(options.clone()).unwrap().into()
        } else {
            js_sys::Function::new_with_args("opts", "return new SourceImageAddon(opts)")
                .call1(&JsValue::UNDEFINED, options)
                .unwrap()
        };
        let terminal = if rust {
            Terminal::new(eval("({allowProposedApi:true,logLevel:'off'})"))
                .unwrap()
                .into()
        } else {
            eval("new SourceTerminal({allowProposedApi:true,logLevel:'off'})")
        };
        if active {
            call(&addon, "activate", std::slice::from_ref(&terminal));
        }
        Self { addon, terminal }
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        call(&self.addon, "dispose", &[]);
        call(&self.terminal, "dispose", &[]);
    }
}

#[wasm_bindgen_test]
fn storage_constructor_and_inactive_setters_keep_raw_identity_without_conversion() {
    for value in values() {
        for rust in [false, true] {
            let owned = Owned::new(rust, &options(&value), false);
            let addon = &owned.addon;
            same(&get(&get(addon, "_opts"), "storageLimit"), &value);
            assert_eq!(get(addon, "storageLimit"), JsValue::from(-1));
            for replacement in values() {
                set(addon, &replacement);
                same(&get(&get(addon, "_opts"), "storageLimit"), &replacement);
                call(addon, "reset", &[]);
                same(&get(&get(addon, "_opts"), "storageLimit"), &replacement);
            }
            call(addon, "dispose", &[]);
            set(addon, &value);
            same(&get(&get(addon, "_opts"), "storageLimit"), &value);
        }
    }
}

fn activate_probe(owned: &Owned, mode: &str) -> JsValue {
    js_sys::Function::new_with_args("addon,terminal,mode", r#"
      const trace=[], sentinel={failure:true}, descriptors={}; let calls=0,caught;
      const original=addon._opts.storageLimit;
      for(const key of ['error','warn']) {
        descriptors[key]=Object.getOwnPropertyDescriptor(console,key);
        Object.defineProperty(console,key,{...descriptors[key],value(message){
          trace.push([key,message,addon.storageLimit,Object.is(addon._opts.storageLimit,original)]);
          if(mode===key)throw sentinel;
          if(mode==='nested' && key==='warn')addon.storageLimit=16;
        }});
      }
      try { addon.activate(terminal); } catch(error) { caught=error; }
      finally { for(const key of ['error','warn'])Object.defineProperty(console,key,descriptors[key]); }
      return {trace,limit:addon.storageLimit,rawSame:Object.is(addon._opts.storageLimit,original),raw16:addon._opts.storageLimit===16,
        failed:caught!==undefined,exact:caught===sentinel,calls};
    "#).apply(&JsValue::UNDEFINED, &js_sys::Array::of3(&owned.addon, &owned.terminal, &mode.into())).unwrap()
}

#[wasm_bindgen_test]
fn raw_constructor_limit_is_deferred_to_activation_with_source_fallback_diagnostics() {
    for expression in [
        "8",
        "'16'",
        "undefined",
        "NaN",
        "0",
        "Symbol('bad')",
        "2n",
        "({valueOf(){throw {nonError:true}}})",
        "({valueOf(){throw Error('limit failure')}})",
    ] {
        for mode in ["normal", "error", "warn", "nested"] {
            let source = Owned::new(false, &options(&eval(expression)), false);
            let rust = Owned::new(true, &options(&eval(expression)), false);
            assert_eq!(
                json(&activate_probe(&rust, mode)),
                json(&activate_probe(&source, mode)),
                "{expression}/{mode}"
            );
        }
    }
    for rust in [false, true] {
        let fixture = eval("(()=>{let reads=0;const first={valueOf(){return 12}},second={valueOf(){throw Error('default must remain unused')}};return {first,get reads(){return reads},options:{sixelSupport:false,iipSupport:false,kittySupport:false,get storageLimit(){return ++reads===1?first:second}}}})()");
        let owned = Owned::new(rust, &get(&fixture, "options"), false);
        assert_eq!(get(&fixture, "reads"), JsValue::from(2));
        same(
            &get(&get(&owned.addon, "_opts"), "storageLimit"),
            &get(&fixture, "first"),
        );
        call(
            &owned.addon,
            "activate",
            std::slice::from_ref(&owned.terminal),
        );
        assert_eq!(get(&owned.addon, "storageLimit"), JsValue::from(12));
        call(&owned.addon, "reset", &[]);
        assert_eq!(get(&fixture, "reads"), JsValue::from(2));
    }
}

#[wasm_bindgen_test]
fn active_primitive_and_object_limits_match_raw_identity_and_validation() {
    let source = Owned::new(false, &options(&128.into()), true);
    let rust = Owned::new(true, &options(&128.into()), true);
    let probe = js_sys::Function::new_with_args(
        "addon,value",
        r#"
      addon.storageLimit=64; let failed=false, error;
      try { addon.storageLimit=value; } catch(e) { failed=true;error=[e.name,e.message]; }
      return {failed,error,limit:addon.storageLimit,same:Object.is(addon._opts.storageLimit,value),previous:addon._opts.storageLimit===64};
    "#,
    );
    for value in values() {
        let actual = probe
            .call2(&JsValue::UNDEFINED, &rust.addon, &value)
            .unwrap();
        let expected = probe
            .call2(&JsValue::UNDEFINED, &source.addon, &value)
            .unwrap();
        assert_eq!(json(&actual), json(&expected));
    }
}

fn setter_probe(addon: &JsValue, expression: &str, action: &str, step: u32) -> JsValue {
    js_sys::Function::new_with_args("addon,values,action,step", r#"
      addon.storageLimit=64;
      const trace=[], sentinel={failure:true}, previous=addon._opts.storageLimit;let count=0,caught,failed=false;
      const raw={ [Symbol.toPrimitive](hint) {
        trace.push([++count,hint,addon.storageLimit,Object.is(addon._opts.storageLimit,previous)]);
        if(count===step){
          if(action.startsWith('nested'))addon.storageLimit=16;
          if(action==='dispose')addon.dispose();
          if(action.endsWith('throw'))throw sentinel;
        }
        return values[Math.min(count-1,values.length-1)];
      }};
      try { addon.storageLimit=raw; } catch(error) { caught=error;failed=true; }
      return {trace,limit:addon.storageLimit,rawSame:Object.is(addon._opts.storageLimit,raw),rawPrevious:Object.is(addon._opts.storageLimit,previous),
        raw16:addon._opts.storageLimit===16,failed,exact:caught===sentinel,error:failed&&caught!==sentinel?[caught.name,caught.message]:null};
    "#).apply(&JsValue::UNDEFINED, &js_sys::Array::of4(addon, &eval(expression), &action.into(), &step.into())).unwrap()
}

#[wasm_bindgen_test]
fn active_limit_repeats_coercion_and_orders_nested_assignment_throw_and_disposal() {
    for expression in [
        "[4,4,16]",
        "[0]",
        "[1,2001]",
        "[1,1,2001]",
        "[1,1,2**40]",
        "[NaN]",
        "[2n]",
        "[Symbol('bad')]",
    ] {
        for action in ["normal", "throw", "nested", "nested-throw", "dispose"] {
            for step in 1..=3 {
                let source = Owned::new(false, &options(&128.into()), true);
                let rust = Owned::new(true, &options(&128.into()), true);
                assert_eq!(
                    json(&setter_probe(&rust.addon, expression, action, step)),
                    json(&setter_probe(&source.addon, expression, action, step)),
                    "{expression}/{action}/{step}"
                );
                // Source retains its disposed storage object and still validates.
                call(&rust.addon, "dispose", &[]);
                call(&source.addon, "dispose", &[]);
                assert_eq!(
                    json(&setter_probe(&rust.addon, expression, "normal", 1)),
                    json(&setter_probe(&source.addon, expression, "normal", 1))
                );
            }
        }
    }
}

#[wasm_bindgen_test(async)]
async fn raw_limit_eviction_commits_pixels_before_image_deletion_and_raw_after_success() {
    let pair = crate::support::Pair::new(false, "({showPlaceholder:false})");
    pair.ready().await;
    pair.write(crate::support::iip()).await;
    let source = get(&pair.source, "addon");
    let rust: JsValue = ImageAddon::new(eval("({showPlaceholder:false})"))
        .unwrap()
        .into();
    let terminal: JsValue = Terminal::new(eval("({allowProposedApi:true,logLevel:'off'})"))
        .unwrap()
        .into();
    call(&rust, "activate", std::slice::from_ref(&terminal));
    let owned = Owned {
        addon: rust,
        terminal,
    };
    // Feed the same real QOI via the public queued writer, not storage shims.
    let text = crate::support::iip();
    crate::support::promise(
        js_sys::Function::new_with_args(
            "terminal,text",
            "return new Promise(resolve=>terminal.write(text,resolve))",
        )
        .call2(&JsValue::UNDEFINED, &owned.terminal, &text.into())
        .unwrap(),
    )
    .await;
    for addon in [&owned.addon, &source] {
        assert!(get(addon, "storageUsage").as_f64().unwrap() > 0.0);
        let raw = eval("({valueOf(){return NaN}})");
        set(addon, &raw);
        assert_eq!(get(addon, "storageLimit"), JsValue::from(-1));
        assert_eq!(get(addon, "storageUsage"), JsValue::from(0));
        same(&get(&get(addon, "_opts"), "storageLimit"), &raw);
    }
}
