use crate::support::*;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_addons::image::{
    kitty::handler::KittyOptions,
    storage::{ImageStorage, ImageStorageOptions, TerminalStorageHost},
};
use xterm_web::{
    image::{BrowserKittyProtocol, ImageRendererLifecycle},
    Terminal,
};

#[wasm_bindgen_test(async)]
async fn pending_display_keeps_terminal_until_commit_without_parser_cycle() {
    let terminal = Terminal::new(eval("({cols:20,rows:10,allowProposedApi:true})")).unwrap();
    let native = terminal.shared_native();
    let owner = ImageRendererLifecycle::new(&terminal).unwrap();
    let host = TerminalStorageHost::from_terminal(&native, Rc::new(|_, _| Ok(()))).unwrap();
    let storage = ImageStorage::new(
        Rc::new(host),
        owner.renderer(),
        ImageStorageOptions::default(),
    )
    .unwrap();
    let protocol = BrowserKittyProtocol::new(
        &terminal,
        owner.renderer(),
        storage.clone(),
        KittyOptions::default(),
    );
    let registration = protocol.register().unwrap();
    assert!(matches!(
        native.write(&sequence("a=T,i=9,s=6,v=9", &raw())),
        Err(xterm_core::headless::HeadlessError::Engine(
            xterm_core::terminal::TerminalError::Pending
        ))
    ));
    let pending = native.take_pending().unwrap().unwrap();
    registration.dispose();
    drop(terminal);
    drop(native);
    protocol
        .register()
        .expect("pending display retains terminal context")
        .dispose();
    assert!(pending.resolve().await.unwrap());
    assert_eq!(storage.image_count(), 1);
    assert!(
        protocol.register().is_err(),
        "no browser-terminal cycle remains"
    );
    storage.dispose().unwrap();
    owner.dispose().unwrap();
}
#[wasm_bindgen_test(async)]
async fn synchronous_blob_read_throw_is_pending_render_failure_not_parser_exception() {
    let live = Live::new(true);
    let source = Source::new(true);
    let marker = eval("({tag:'read'})");
    let _patch = Patch::new(
        get(&get(&js_sys::global(), "Blob"), "prototype"),
        "arrayBuffer",
        thrower(&marker),
    );
    let text = sequence("a=T,i=9,s=6,v=9", &raw());
    assert!(live.begin(&text));
    assert!(source.begin(&text));
    live.finish(true).await;
    source.finish().await;
    same_state(&live, &source);
    assert_eq!(live.added.get(), 0);
}
#[wasm_bindgen_test(async)]
async fn throwing_bitmap_close_has_source_repeated_cleanup_order() {
    let live = Live::new(true);
    let source = Source::new(true);
    let marker = eval("({tag:'close'})");
    let wrapper = js_sys::Function::new_with_args(
        "token",
        "return function(){globalThis.kittyCloseLog.push([this.width,this.height]);throw token}",
    )
    .call1(&JsValue::UNDEFINED, &marker)
    .unwrap();
    let _log = Patch::new(
        js_sys::global().into(),
        "kittyCloseLog",
        js_sys::Array::new().into(),
    );
    let _patch = Patch::new(
        get(&get(&js_sys::global(), "ImageBitmap"), "prototype"),
        "close",
        wrapper,
    );
    let text = sequence("a=T,i=9,s=6,v=9,c=2,r=2", &raw());
    live.write(&text).await;
    let before = js_sys::JSON::stringify(&get(&js_sys::global(), "kittyCloseLog")).unwrap();
    call(
        &get(&js_sys::global(), "kittyCloseLog"),
        "splice",
        &[0.into()],
    );
    source.write(&text).await;
    assert_eq!(
        before,
        js_sys::JSON::stringify(&get(&js_sys::global(), "kittyCloseLog")).unwrap()
    );
    assert_eq!(
        get(&get(&js_sys::global(), "kittyCloseLog"), "length").as_f64(),
        Some(2.0)
    );
    same_state(&live, &source);
}
