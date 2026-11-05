// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use crate::support::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_web::image::ImageAddon;

fn values() -> js_sys::Array {
    eval("[undefined,null,false,true,0,-0,NaN,1,-1,'','yes',0n,1n,Symbol('placeholder'),{},[],new Boolean(false),{[Symbol.toPrimitive](){throw Error('must not coerce')}}]")
        .unchecked_into()
}
fn same(actual: &JsValue, expected: &JsValue) {
    assert!(js_sys::Object::is(actual, expected));
}
fn set(addon: &JsValue, value: &JsValue) {
    assert!(js_sys::Reflect::set(addon, &"showPlaceholder".into(), value).unwrap());
}
fn check(addon: &JsValue, value: &JsValue) {
    same(&get(addon, "showPlaceholder"), value);
    same(&get(&get(addon, "_opts"), "showPlaceholder"), value);
}

#[wasm_bindgen_test]
fn constructor_and_inactive_setters_preserve_raw_placeholder_identity() {
    for value in values() {
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"showPlaceholder".into(), &value).unwrap();
        let actual: JsValue = ImageAddon::new(options.clone().into()).unwrap().into();
        let source =
            js_sys::Function::new_with_args("options", "return new SourceImageAddon(options)")
                .call1(&JsValue::UNDEFINED, &options)
                .unwrap();
        for addon in [&actual, &source] {
            check(addon, &value);
            for replacement in values() {
                set(addon, &replacement);
                check(addon, &replacement);
                call(addon, "reset", &[]);
                check(addon, &replacement);
            }
            let retained = get(addon, "showPlaceholder");
            call(addon, "dispose", &[]);
            check(addon, &retained);
            set(addon, &value);
            check(addon, &value);
        }
    }
}

#[wasm_bindgen_test]
fn constructor_keeps_first_placeholder_copy_and_does_not_reset_it() {
    let make = js_sys::Function::new_with_args(
        "",
        r#"
      const first={}, second={}; let reads=0;
      return {first,second,get reads(){return reads},options:{get showPlaceholder(){return ++reads===1?first:second}}};
    "#,
    );
    for rust in [false, true] {
        let input = make.call0(&JsValue::UNDEFINED).unwrap();
        let options = get(&input, "options");
        let addon: JsValue = if rust {
            ImageAddon::new(options).unwrap().into()
        } else {
            js_sys::Function::new_with_args("options", "return new SourceImageAddon(options)")
                .call1(&JsValue::UNDEFINED, &options)
                .unwrap()
        };
        check(&addon, &get(&input, "first"));
        assert_eq!(get(&input, "reads"), JsValue::from(2));
        call(&addon, "reset", &[]);
        check(&addon, &get(&input, "first"));
        assert_eq!(get(&input, "reads"), JsValue::from(2));
        call(&addon, "dispose", &[]);
    }
}

// This probe changes only the ImageData host constructor, restored in finally.
// Both implementations must store raw values before any rendering callback.
fn probe(addon: &JsValue, value: &JsValue, mode: &str, inner: &JsValue) -> JsValue {
    js_sys::Function::new_with_args(
        "addon,value,mode,inner",
        r#"
      const descriptor=Object.getOwnPropertyDescriptor(globalThis,'ImageData');
      const error={canvasFailure:true}; let calls=0, caught, stored=true;
      Object.defineProperty(globalThis,'ImageData',{...descriptor,value:new Proxy(ImageData,{
        construct(target,args,newTarget){
          calls++;
          stored &&= Object.is(addon.showPlaceholder,value) && Object.is(addon._opts.showPlaceholder,value);
          if(mode==='reenter')addon.showPlaceholder=inner;
          if(mode!=='normal')throw error;
          return Reflect.construct(target,args,newTarget);
        }
      })});
      try{addon.showPlaceholder=value;}catch(e){caught=e;}
      finally{Object.defineProperty(globalThis,'ImageData',descriptor);}
      return {calls,stored,error: caught===error,unexpected: caught!==undefined && caught!==error};
    "#,
    )
    .apply(
        &JsValue::UNDEFINED,
        &js_sys::Array::of4(addon, value, &mode.into(), inner),
    )
    .unwrap()
}

struct OpenAddon {
    addon: JsValue,
    terminal: JsValue,
    parent: JsValue,
}
impl Drop for OpenAddon {
    fn drop(&mut self) {
        call(&self.addon, "dispose", &[]);
        call(&self.terminal, "dispose", &[]);
        call(&self.parent, "remove", &[]);
    }
}

#[wasm_bindgen_test(async)]
async fn active_placeholder_uses_truthiness_and_preserves_values_on_host_failure_and_reentry() {
    let pair = Pair::new(
        true,
        "({showPlaceholder:false,sixelSupport:false,iipSupport:false,kittySupport:false})",
    );
    let source = get(&pair.source, "addon");
    // The wasm-bindgen class itself is exercised, rather than a typed Rust call.
    let actual: JsValue = ImageAddon::new(eval(
        "({showPlaceholder:false,sixelSupport:false,iipSupport:false,kittySupport:false})",
    ))
    .unwrap()
    .into();
    // Use a separate real terminal so two addons do not replace each other's hooks.
    let terminal: JsValue = xterm_web::Terminal::new(eval(
        "({cols:20,rows:10,fontSize:14,allowProposedApi:true,logLevel:'off'})",
    ))
    .unwrap()
    .into();
    let parent = eval("document.body.appendChild(document.createElement('div'))");
    call(&terminal, "open", std::slice::from_ref(&parent));
    call(&actual, "activate", std::slice::from_ref(&terminal));
    let owned = OpenAddon {
        addon: actual,
        terminal,
        parent,
    };
    let actual = &owned.addon;
    for value in values() {
        for addon in [actual, &source] {
            set(addon, &JsValue::FALSE);
            let result = probe(addon, &value, "normal", &JsValue::UNDEFINED);
            assert_eq!(
                get(&result, "calls"),
                JsValue::from(i32::from(value.is_truthy()))
            );
            assert_eq!(get(&result, "stored"), JsValue::TRUE);
            assert_eq!(get(&result, "error"), JsValue::FALSE);
            assert_eq!(get(&result, "unexpected"), JsValue::FALSE);
            check(addon, &value);
            call(addon, "reset", &[]);
            check(addon, &value);
        }
        pair.frame().await;
    }
    let outer = eval("({outer:true})");
    let inner = eval("({inner:true})");
    for mode in ["throw", "reenter"] {
        for addon in [actual, &source] {
            set(addon, &JsValue::FALSE);
            // Falsy reentry takes the real clear-placeholder path without
            // recursively allocating another canvas.
            let nested = if mode == "reenter" {
                JsValue::from("")
            } else {
                inner.clone()
            };
            let result = probe(addon, &outer, mode, &nested);
            assert_eq!(get(&result, "calls"), JsValue::from(1));
            assert_eq!(get(&result, "stored"), JsValue::TRUE);
            assert_eq!(get(&result, "error"), JsValue::TRUE);
            assert_eq!(get(&result, "unexpected"), JsValue::FALSE);
            check(addon, if mode == "reenter" { &nested } else { &outer });
        }
    }
    for addon in [actual, &source] {
        call(addon, "dispose", &[]);
        set(addon, &JsValue::NULL);
        check(addon, &JsValue::NULL);
    }
}
