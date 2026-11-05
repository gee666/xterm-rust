// Copyright (c) 2020 The xterm.js authors. MIT license.
use super::{call, eval, get, iip, Pair};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

struct Gate {
    original: JsValue,
    state: JsValue,
}
impl Gate {
    fn new() -> Self {
        let original = get(&js_sys::global(), "createImageBitmap");
        let state = eval("({requests:[]})");
        let replacement=js_sys::Function::new_with_args("original,state", "return function(...args){return new Promise(resolve=>state.requests.push(()=>resolve(original.apply(this,args))))}").call2(&JsValue::UNDEFINED,&original,&state).unwrap();
        js_sys::Reflect::set(&js_sys::global(), &"createImageBitmap".into(), &replacement).unwrap();
        Self { original, state }
    }
    fn release(&self) {
        js_sys::Function::new_with_args("state","if(state.requests.length!==2)throw Error('expected both real pending bitmaps');for(const release of state.requests)release();").call1(&JsValue::UNDEFINED,&self.state).unwrap();
    }
}
impl Drop for Gate {
    fn drop(&mut self) {
        js_sys::Reflect::set(
            &js_sys::global(),
            &"createImageBitmap".into(),
            &self.original,
        )
        .unwrap();
    }
}
pub async fn late_completion() {
    let pair = Pair::new(true, "({showPlaceholder:false})");
    pair.ready().await;
    let gate = Gate::new();
    let text = iip().replace("width=2px;height=2px", "width=14px;height=10px");
    let native = pair.terminal.shared_native();
    assert!(matches!(
        native.write(&text),
        Err(xterm_core::headless::HeadlessError::Engine(
            xterm_core::terminal::TerminalError::Pending
        ))
    ));
    let parse = js_sys::Function::new_with_args(
        "terminal,text,result",
        "return terminal._core._inputHandler.parse(text,result)",
    );
    let terminal = get(&pair.source, "terminal");
    let pending = parse
        .call2(&JsValue::UNDEFINED, &terminal, &text.into())
        .unwrap();
    assert!(pending.is_instance_of::<js_sys::Promise>());
    pair.addon.dispose().unwrap();
    call(&get(&pair.source, "addon"), "dispose", &[]);
    assert_eq!(pair.addon.storage_usage().unwrap(), 0.0);
    gate.release();
    let done = native
        .take_pending()
        .unwrap()
        .unwrap()
        .resolve()
        .await
        .unwrap();
    native.resume(done).unwrap();
    let done = JsFuture::from(js_sys::Promise::resolve(&pending))
        .await
        .unwrap();
    parse
        .call3(&JsValue::UNDEFINED, &terminal, &"".into(), &done)
        .unwrap();
    pair.frame().await;
    pair.compare();
    pair.compare_image(0.0, 0.0);
    assert!(pair.addon.storage_usage().unwrap() > 0.0);
    assert_eq!(get(&pair.events, "count"), JsValue::from(0));
}
use wasm_bindgen::JsCast;

struct RemovalFailure {
    prototype: JsValue,
    original: JsValue,
}
impl Drop for RemovalFailure {
    fn drop(&mut self) {
        js_sys::Reflect::set(&self.prototype, &"remove".into(), &self.original).unwrap();
    }
}
pub async fn failed_disposal() {
    let pair = Pair::new(true, "({showPlaceholder:false})");
    pair.ready().await;
    pair.write(super::sixel()).await;
    pair.compare();
    let prototype = eval("HTMLCanvasElement.prototype");
    let original = get(&prototype, "remove");
    let token = eval("({disposalFailure:true})");
    let method=js_sys::Function::new_with_args("original,token","return function(){if(this.classList.contains('xterm-image-layer-top'))throw token;return original.call(this);}").call2(&JsValue::UNDEFINED,&original,&token).unwrap();
    let patch = RemovalFailure {
        prototype: prototype.clone(),
        original,
    };
    js_sys::Reflect::set(&prototype, &"remove".into(), &method).unwrap();
    let error = pair.addon.dispose().unwrap_err();
    assert_eq!(error, token);
    let source_error =
        js_sys::Function::new_with_args("addon", "try {addon.dispose()}catch(error){return error}")
            .call1(&JsValue::UNDEFINED, &get(&pair.source, "addon"))
            .unwrap();
    assert_eq!(source_error, token);
    drop(patch);
    pair.compare();
    pair.write(iip()).await;
    pair.compare();
    assert_eq!(get(&pair.events, "count"), JsValue::from(2));
    pair.addon.dispose().unwrap();
    call(&get(&pair.source, "addon"), "dispose", &[]);
    pair.compare();
    assert_eq!(pair.addon.storage_usage().unwrap(), 0.0);
}
