use crate::support::*;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

#[wasm_bindgen_test(async)]
async fn bitmap_rejections_and_host_constructor_throws_send_render_failure() {
    for key in ["createImageBitmap", "ImageData"] {
        let live = Live::new(true);
        let source = Source::new(true);
        let marker = eval("({tag:'host'})");
        let _patch = Patch::new(js_sys::global().into(), key, thrower(&marker));
        let text = sequence("a=T,i=9,s=6,v=9", &raw());
        live.write(&text).await;
        source.write(&text).await;
        same_state(&live, &source);
        same_maps(&live, &source).await;
        assert_eq!(live.added.get(), 0);
        assert!(String::from_utf16(&live.replies.borrow()[0])
            .unwrap()
            .contains("image rendering failed"));
    }
    let live = Live::new(true);
    let source = Source::new(true);
    let _patch = Patch::new(
        js_sys::global().into(),
        "createImageBitmap",
        eval("()=>Promise.reject({tag:'bitmap'})"),
    );
    let text = sequence("a=T,i=9,s=6,v=9", &raw());
    live.write(&text).await;
    source.write(&text).await;
    same_state(&live, &source);
    assert_eq!(live.added.get(), 0);
}
#[wasm_bindgen_test(async)]
async fn blob_throw_is_synchronous_and_preserves_exception_identity() {
    let live = Live::new(true);
    let source = Source::new(true);
    let marker = eval("({tag:'blob'})");
    let _patch = Patch::new(js_sys::global().into(), "Blob", thrower(&marker));
    let text = sequence("a=T,i=9,s=6,v=9", &raw());
    let native = live.terminal.shared_native().write(&text).unwrap_err();
    assert!(xterm_web::js_error::to_js_error(&native) == marker);
    assert_eq!(
        try_call(&source.0, "begin", &[text.into()]).unwrap_err(),
        marker
    );
    same_state(&live, &source);
}
#[wasm_bindgen_test(async)]
async fn throwing_storage_callback_closes_bitmap_and_preserves_partial_cells() {
    let live = Live::new(true);
    let source = Source::new(true);
    let marker = eval("({tag:'insert'})");
    let token = marker.clone();
    live.storage.set_on_image_added(Some(Rc::new(move || {
        Err(xterm_addons::image::storage::StorageError::Event(
            xterm_web::js_error::callback_error(token.clone()),
        ))
    })));
    js_sys::Reflect::set(
        &get(&source.0, "store"),
        &"onImageAdded".into(),
        &thrower(&marker),
    )
    .unwrap();
    let closes = eval("(()=>{const old=ImageBitmap.prototype.close,log=[];ImageBitmap.prototype.close=function(){log.push([this.width,this.height]);return old.call(this)};return {log,restore(){ImageBitmap.prototype.close=old}}})()");
    let text = sequence("a=T,i=9,s=6,v=9", &raw());
    live.write(&text).await;
    let before = js_sys::JSON::stringify(&get(&closes, "log")).unwrap();
    call(&get(&closes, "log"), "splice", &[0.into()]);
    source.write(&text).await;
    assert_eq!(
        before,
        js_sys::JSON::stringify(&get(&closes, "log")).unwrap()
    );
    call(&closes, "restore", &[]);
    // The fixture added counter is replaced, so compare all other state directly.
    assert_eq!(
        js_sys::JSON::stringify(&live.snapshot()).unwrap(),
        js_sys::JSON::stringify(&call(&source.0, "snapshot", &[])).unwrap()
    );
    same_maps(&live, &source).await;
}
#[wasm_bindgen_test(async)]
async fn reentrant_reset_from_image_data_preserves_late_placement_like_source() {
    let live = Live::new(true);
    let source = Source::new(true);
    let reset = live.protocol.clone();
    let count = Rc::new(Cell::new(0));
    let seen = count.clone();
    let callback = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
        reset.reset().unwrap();
        seen.set(seen.get() + 1);
    });
    let wrapper = js_sys::Function::new_with_args(
        "old,callback",
        "return function(...args){callback();return Reflect.construct(old,args)}",
    )
    .call2(
        &JsValue::UNDEFINED,
        &get(&js_sys::global(), "ImageData"),
        callback.as_ref(),
    )
    .unwrap();
    let text = sequence("a=T,i=9,s=6,v=9", &raw());
    {
        let _patch = Patch::new(js_sys::global().into(), "ImageData", wrapper);
        live.write(&text).await;
    }
    assert_eq!(count.get(), 1);
    let wrapper = js_sys::Function::new_with_args(
        "old,s",
        "return function(...args){s.h.reset();return Reflect.construct(old,args)}",
    )
    .call2(
        &JsValue::UNDEFINED,
        &get(&js_sys::global(), "ImageData"),
        &source.0,
    )
    .unwrap();
    {
        let _patch = Patch::new(js_sys::global().into(), "ImageData", wrapper);
        source.write(&text).await;
    }
    same_state(&live, &source);
    same_maps(&live, &source).await;
}
#[wasm_bindgen_test(async)]
async fn response_callback_throw_rejects_pending_handler_with_original_value() {
    let live = Live::new(true);
    let source = Source::new(true);
    let marker = eval("({tag:'response'})");
    let token = marker.clone();
    let listener = live
        .terminal
        .shared_native()
        .on_data()
        .try_subscribe(move |_| Err(xterm_web::js_error::callback_error(token.clone())));
    let subscription = call(&get(&source.0, "t"), "onData", &[thrower(&marker)]);
    let text = sequence("a=T,i=9,s=6,v=9", &raw());
    assert!(live.begin(&text));
    assert!(source.begin(&text));
    let result = live
        .terminal
        .shared_native()
        .take_pending()
        .unwrap()
        .unwrap()
        .resolve()
        .await
        .unwrap_err();
    assert_eq!(xterm_web::js_error::to_js_error(&result), marker);
    let result = wasm_bindgen_futures::JsFuture::from(
        call(&source.0, "finish", &[]).unchecked_into::<js_sys::Promise>(),
    )
    .await
    .unwrap_err();
    assert_eq!(result, marker);
    same_state(&live, &source);
    same_maps(&live, &source).await;
    listener.dispose();
    call(&subscription, "dispose", &[]);
}
