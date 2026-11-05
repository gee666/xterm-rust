// Copyright (c) 2020, 2023 The xterm.js authors. MIT license.
use super::*;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_addons::image::storage::StorageError;
use xterm_web::js_error::{callback_error, to_js_error};

#[wasm_bindgen_test(async)]
async fn synchronous_canvas_and_image_data_failures_keep_error_identity_and_tail() {
    for stage in ["createElement", "getContext", "ImageData", "putImageData"] {
        let (live, source, _) = ready(true).await;
        let token = eval("({token:'SIXEL host failure'})");
        let object = match stage {
            "createElement" => get(&js_sys::global(), "document"),
            "getContext" => eval("HTMLCanvasElement.prototype"),
            "ImageData" => js_sys::global().into(),
            _ => eval("CanvasRenderingContext2D.prototype"),
        };
        let patch = Patch::new(object, stage, thrower(&token));
        let text = format!("{}NEVER", sequence("\"1;1;2;3#1!2~", true));
        let error = live.terminal.shared_native().write(&text).unwrap_err();
        assert!(to_js_error(&error) == token, "Rust stage {stage}");
        assert!(
            try_call(&source.0, "begin", &[text.into()]).unwrap_err() == token,
            "source stage {stage}"
        );
        drop(patch);
        same_state(&live, &source);
        assert_eq!(live.added.get(), 0);
    }
}

#[wasm_bindgen_test(async)]
async fn decoding_warning_and_throwing_warning_preserve_source_state() {
    let (live, source, opts) = ready(true).await;
    let mut options = opts.get();
    options.sixel_size_limit = 1.0;
    opts.set(options);
    source_option(&source, "sixelSizeLimit", 1.into());
    let token = eval("({token:'SIXEL warning threw'})");
    let patch = Patch::new(get(&js_sys::global(), "console"), "warn", thrower(&token));
    let text = format!("{}NEVER", sequence("!3~", true));
    assert!(to_js_error(&live.terminal.shared_native().write(&text).unwrap_err()) == token);
    assert!(try_call(&source.0, "begin", &[text.into()]).unwrap_err() == token);
    drop(patch);
    same_state(&live, &source);
    assert_eq!(live.added.get(), 0);
    // A rejected put leaves source's decoder available, rather than aborting it
    // before console.warn returns. Reset and a fresh sequence remain usable.
    options.sixel_size_limit = 33554432.0;
    opts.set(options);
    source_option(&source, "sixelSizeLimit", 33554432.into());
    live.protocol.reset();
    call(&source.0, "reset", &[]);
    live.write(&sequence("#1!2~", true)).await;
    source.write(&sequence("#1!2~", true)).await;
    same_state(&live, &source);
}

#[wasm_bindgen_test(async)]
async fn put_image_data_reset_reentry_uses_live_decoder_and_keeps_uploaded_pixels() {
    let (live, source, _) = ready(true).await;
    let protocol = live.protocol.clone();
    let reset = Closure::<dyn Fn()>::new(move || protocol.reset());
    let install = js_sys::Function::new_with_args(
        "callback",
        r#"
        const prototype=CanvasRenderingContext2D.prototype,old=prototype.putImageData;
        prototype.putImageData=function(...args){const result=old.apply(this,args);callback();return result};
        return ()=>prototype.putImageData=old;"#,
    );
    let restore = install.call1(&JsValue::UNDEFINED, reset.as_ref()).unwrap();
    let text = sequence("\"1;1;2;3#1;2;100;0;0!2~", true);
    live.write(&text).await;
    restore
        .unchecked_into::<js_sys::Function>()
        .call0(&JsValue::UNDEFINED)
        .unwrap();
    let callback = js_sys::Function::new_with_args("h", "return ()=>h.reset()")
        .call1(&JsValue::UNDEFINED, &get(&source.0, "h"))
        .unwrap();
    let restore = install.call1(&JsValue::UNDEFINED, &callback).unwrap();
    source.write(&text).await;
    restore
        .unchecked_into::<js_sys::Function>()
        .call0(&JsValue::UNDEFINED)
        .unwrap();
    same_state(&live, &source);
    assert_eq!(pixels(&live.image(0, 0)), pixels(&source.image(0, 0)));
    let text = sequence("#1!2~", true);
    live.write(&text).await;
    source.write(&text).await;
    same_state(&live, &source);
    assert_eq!(pixels(&live.image(0, 0)), pixels(&source.image(0, 0)));
}

#[wasm_bindgen_test(async)]
async fn memory_and_indirect_call_traps_use_actual_browser_diagnostics() {
    for body in ["!4294967295~", "#1;4294967295;0;0;0~"] {
        let (live, source, _) = ready(true).await;
        let patch = eval("(()=>{const old=console.warn,log=[];console.warn=(...args)=>log.push(args.map(String));return {log,restore(){console.warn=old}}})()");
        let text = sequence(body, true);
        live.write(&text).await;
        let rust_log = js_sys::JSON::stringify(&get(&patch, "log")).unwrap();
        call(&get(&patch, "log"), "splice", &[0.into()]);
        source.write(&text).await;
        let source_log = js_sys::JSON::stringify(&get(&patch, "log")).unwrap();
        call(&patch, "restore", &[]);
        assert_eq!(rust_log, source_log, "body={body}");
        assert_ne!(rust_log.as_string().unwrap(), "[]");
        same_state(&live, &source);
        assert_eq!(live.added.get(), 0);
    }
}

#[wasm_bindgen_test(async)]
async fn canvas_method_and_constructor_getters_keep_source_evaluation_order() {
    let (live, source, _) = ready(true).await;
    let patch = eval(
        r#"(()=>{
        const log=[],proto=CanvasRenderingContext2D.prototype;
        const method=Object.getOwnPropertyDescriptor(proto,'putImageData');
        const image=Object.getOwnPropertyDescriptor(globalThis,'ImageData');
        const clamped=Object.getOwnPropertyDescriptor(globalThis,'Uint8ClampedArray');
        Object.defineProperty(proto,'putImageData',{configurable:true,get(){log.push('method');return method.value}});
        for(const [name,descriptor] of [['ImageData',image],['Uint8ClampedArray',clamped]]){
            Object.defineProperty(globalThis,name,{configurable:true,get(){log.push(name);return new Proxy(descriptor.value,{construct(target,args){log.push('new '+name);return Reflect.construct(target,args)}})}});
        }
        return {log,restore(){Object.defineProperty(proto,'putImageData',method);Object.defineProperty(globalThis,'ImageData',image);Object.defineProperty(globalThis,'Uint8ClampedArray',clamped)}};
    })()"#,
    );
    let text = sequence("\"1;1;2;3#1!2~", true);
    live.write(&text).await;
    let rust_log = js_sys::JSON::stringify(&get(&patch, "log")).unwrap();
    call(&get(&patch, "log"), "splice", &[0.into()]);
    source.write(&text).await;
    let source_log = js_sys::JSON::stringify(&get(&patch, "log")).unwrap();
    call(&patch, "restore", &[]);
    assert_eq!(rust_log, source_log);
    same_state(&live, &source);
    assert_eq!(pixels(&live.image(0, 0)), pixels(&source.image(0, 0)));
}

#[wasm_bindgen_test(async)]
async fn storage_callback_error_is_synchronous_and_retains_inserted_image() {
    let (live, source, _) = ready(true).await;
    let token = eval("({token:'SIXEL image added failure'})");
    let cause = token.clone();
    live.storage.set_on_image_added(Some(Rc::new(move || {
        Err(StorageError::Event(callback_error(cause.clone())))
    })));
    js_sys::Reflect::set(
        &get(&source.0, "store"),
        &"onImageAdded".into(),
        &thrower(&token),
    )
    .unwrap();
    let text = format!("{}NEVER", sequence("#1!2~", true));
    assert!(to_js_error(&live.terminal.shared_native().write(&text).unwrap_err()) == token);
    assert!(try_call(&source.0, "begin", &[text.into()]).unwrap_err() == token);
    same_state(&live, &source);
    assert_eq!(live.added.get(), 0);
    assert_eq!(pixels(&live.image(0, 0)), pixels(&source.image(0, 0)));
}
