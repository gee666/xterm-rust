use crate::support::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test(async)]
async fn png_image_event_fallback_uses_actual_pixels_without_bitmap_at_entry() {
    let live = Live::new(true);
    let source = Source::new(true);
    let old_bitmap = get(&js_sys::global(), "createImageBitmap");
    let old_image = get(&js_sys::global(), "Image");
    let constructor = js_sys::Function::new_with_args(
        "ImageCtor,bitmap",
        r#"
        return function(){
          const image=new ImageCtor();
          image.addEventListener('load',()=>{globalThis.createImageBitmap=bitmap});
          return image;
        }"#,
    )
    .call2(&JsValue::UNDEFINED, &old_image, &old_bitmap)
    .unwrap();
    let _image = Patch::new(js_sys::global().into(), "Image", constructor);
    let text = sequence("a=T,i=9,f=100", PNG);
    {
        let _bitmap = Patch::new(
            js_sys::global().into(),
            "createImageBitmap",
            JsValue::UNDEFINED,
        );
        live.write(&text).await;
    }
    {
        let _bitmap = Patch::new(
            js_sys::global().into(),
            "createImageBitmap",
            JsValue::UNDEFINED,
        );
        source.write(&text).await;
    }
    same_state(&live, &source);
    same_maps(&live, &source).await;
    assert_eq!(live.added.get(), 1);
    assert_eq!(pixels(&live.image(0, 0)), pixels(&source.image(0, 0)));
}
#[wasm_bindgen_test(async)]
async fn pending_command_remains_live_across_real_blob_read() {
    let live = Live::new(true);
    let source = Source::new(true);
    let encoded = payload(&raw());
    let first = format!("\x1b_Ga=T,i=33,s=6,v=9,m=1;{}\x1b\\", &encoded[..31]);
    live.write(&first).await;
    source.write(&first).await;
    let command = live
        .protocol
        .handler()
        .pending_transmissions()
        .get(33.0)
        .unwrap()
        .unwrap()
        .command
        .borrow()
        .clone();
    let original = call(
        &get(&get(&source.0, "h"), "pendingTransmissions"),
        "get",
        &[33.into()],
    );
    let source_command = get(&original, "cmd");
    let gate = eval("(()=>{const old=Blob.prototype.arrayBuffer,gates=[];Blob.prototype.arrayBuffer=function(){const p=old.call(this);return new Promise(resolve=>gates.push(()=>p.then(resolve)))};return {gates,restore(){Blob.prototype.arrayBuffer=old}}})()");
    let last = format!("\x1b_Gm=0;{}\x1b\\", &encoded[31..]);
    assert!(live.begin(&last));
    assert!(source.begin(&last));
    {
        let mut command = command.borrow_mut();
        command.columns = Some(2.0);
        command.rows = Some(2.0);
        command.x_offset = Some(2.0);
        command.y_offset = Some(3.0);
        command.z_index = Some(-4.0);
        command.cursor_movement = Some(1.0);
    }
    for (key, value) in [
        ("columns", 2),
        ("rows", 2),
        ("xOffset", 2),
        ("yOffset", 3),
        ("zIndex", -4),
        ("cursorMovement", 1),
    ] {
        js_sys::Reflect::set(&source_command, &key.into(), &value.into()).unwrap();
    }
    call(&gate, "restore", &[]);
    call(&get(&gate, "gates"), "forEach", &[eval("f=>f()")]);
    live.finish(true).await;
    source.finish().await;
    same_state(&live, &source);
    same_maps(&live, &source).await;
    assert_eq!(pixels(&live.image(0, 0)), pixels(&source.image(0, 0)));
}
#[wasm_bindgen_test(async)]
async fn rejected_scale_closes_current_bitmap_but_not_storage_owned_image() {
    for fail in [true, false] {
        let live = Live::new(true);
        let source = Source::new(true);
        let text = sequence("a=T,i=7,s=6,v=9,c=2,r=2", &raw());
        let instrument = js_sys::Function::new_with_args("fail",r#"
          const old=createImageBitmap,close=ImageBitmap.prototype.close,log=[];
          globalThis.createImageBitmap=function(image,...args){
            log.push(['bitmap',image.width,image.height,args.length]);
            if(fail&&args.length)return Promise.reject(new Error('scale'));
            return old(image,...args);
          };
          ImageBitmap.prototype.close=function(){log.push(['close',this.width,this.height]);return close.call(this)};
          return {log,restore(){globalThis.createImageBitmap=old;ImageBitmap.prototype.close=close}};
        "#).call1(&JsValue::UNDEFINED,&fail.into()).unwrap();
        live.write(&text).await;
        let before = js_sys::JSON::stringify(&get(&instrument, "log")).unwrap();
        call(&get(&instrument, "log"), "splice", &[0.into()]);
        source.write(&text).await;
        assert_eq!(
            before,
            js_sys::JSON::stringify(&get(&instrument, "log")).unwrap()
        );
        call(&instrument, "restore", &[]);
        same_state(&live, &source);
        same_maps(&live, &source).await;
        assert_eq!(live.added.get(), usize::from(!fail));
    }
}
#[wasm_bindgen_test(async)]
async fn bitmap_promise_can_finish_after_storage_reset_disposal() {
    let live = Live::new(true);
    let source = Source::new(true);
    let gate=eval("(()=>{const old=createImageBitmap,gates=[];let ready;const entered=new Promise(resolve=>ready=resolve);globalThis.createImageBitmap=(...args)=>{const p=old(...args);return new Promise(resolve=>{gates.push(()=>p.then(resolve));if(gates.length===2)ready()})};return {gates,entered,restore(){globalThis.createImageBitmap=old}}})()");
    let text = sequence("a=T,i=7,s=6,v=9", &raw());
    assert!(live.begin(&text));
    assert!(source.begin(&text));
    let pending = live
        .terminal
        .shared_native()
        .take_pending()
        .unwrap()
        .unwrap();
    let completed = wasm_bindgen_futures::future_to_promise(async move {
        Ok(pending.resolve().await.unwrap().into())
    });
    // Wait for the actual API calls, not an assumed number of microtasks.
    promise(get(&gate, "entered")).await;
    assert_eq!(get(&get(&gate, "gates"), "length").as_f64(), Some(2.0));
    live.storage.reset().unwrap();
    live.storage.dispose().unwrap();
    call(&get(&source.0, "store"), "reset", &[]);
    call(&get(&source.0, "store"), "dispose", &[]);
    call(&gate, "restore", &[]);
    call(&get(&gate, "gates"), "forEach", &[eval("f=>f()")]);
    let result = wasm_bindgen_futures::JsFuture::from(completed)
        .await
        .unwrap()
        .as_bool()
        .unwrap();
    live.terminal.shared_native().resume(result).unwrap();
    source.finish().await;
    same_state(&live, &source);
    same_maps(&live, &source).await;
    assert_eq!(live.added.get(), 1);
}
