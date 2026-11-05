// Copyright (c) 2023 The xterm.js authors. MIT license.
use super::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use xterm_addons::image::storage::StorageError;
use xterm_web::js_error::{callback_error, to_js_error};

struct Patch {
    object: JsValue,
    key: &'static str,
    previous: JsValue,
}
impl Patch {
    fn new(object: JsValue, key: &'static str, value: JsValue) -> Self {
        let previous = get(&object, key);
        js_sys::Reflect::set(&object, &key.into(), &value).unwrap();
        Self {
            object,
            key,
            previous,
        }
    }
}
impl Drop for Patch {
    fn drop(&mut self) {
        js_sys::Reflect::set(&self.object, &self.key.into(), &self.previous).unwrap();
    }
}
fn try_call(value: &JsValue, key: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
    get(value, key)
        .unchecked_into::<js_sys::Function>()
        .apply(value, &args.iter().cloned().collect::<js_sys::Array>())
}
fn thrower(token: &JsValue) -> JsValue {
    js_sys::Function::new_with_args("token", "return function(){throw token}")
        .call1(&JsValue::UNDEFINED, token)
        .unwrap()
}
fn warnings() -> (JsValue, Patch) {
    let values = js_sys::Array::new();
    let capture =
        js_sys::Function::new_with_args("values", "return function(...args){values.push(args)}")
            .call1(&JsValue::UNDEFINED, &values)
            .unwrap();
    let patch = Patch::new(get(&js_sys::global(), "console"), "warn", capture);
    (values.into(), patch)
}

#[wasm_bindgen_test]
fn create_bitmap_synchronous_exception_preserves_original_identity_and_parser_tail() {
    let live = Live::new(true);
    let source = Source::new(true);
    let token = eval("({token:'synchronous bitmap call'})");
    let _patch = Patch::new(
        js_sys::global().into(),
        "createImageBitmap",
        thrower(&token),
    );
    let text = format!(
        "{}NEVER",
        sequence(PNG, "width=14px;height=10px;preserveAspectRatio=0")
    );
    let error = live.terminal.shared_native().write(&text).unwrap_err();
    assert!(to_js_error(&error) == token);
    assert!(try_call(&source.0, "begin", &[text.into()]).unwrap_err() == token);
    same_state(&live, &source);
    assert_eq!(live.added.get(), 0);
}

#[wasm_bindgen_test(async)]
async fn bitmap_rejection_is_warned_with_cause_identity_and_parser_resumes() {
    let live = Live::new(true);
    let source = Source::new(true);
    let token = eval("({token:'rejected bitmap'})");
    let function =
        js_sys::Function::new_with_args("token", "return function(){return Promise.reject(token)}")
            .call1(&JsValue::UNDEFINED, &token)
            .unwrap();
    let _patch = Patch::new(js_sys::global().into(), "createImageBitmap", function);
    let (record, _warning) = warnings();
    let text = format!(
        "{}Z",
        sequence(PNG, "width=14px;height=10px;preserveAspectRatio=0")
    );
    assert!(live.begin(&text));
    assert!(source.begin(&text));
    live.finish(true).await;
    source.finish().await;
    same_state(&live, &source);
    let warnings = js_sys::Array::from(&record);
    assert_eq!(warnings.length(), 2);
    for value in warnings.iter() {
        let args = js_sys::Array::from(&value);
        assert!(args
            .get(0)
            .as_string()
            .unwrap()
            .starts_with("IIP: decoding error image/png "));
        assert!(args.get(1) == token);
    }
}

#[wasm_bindgen_test(async)]
async fn throwing_async_warning_rejects_parser_continuation_with_original_value() {
    let live = Live::new(true);
    let source = Source::new(true);
    let token = eval("({token:'throwing async warning'})");
    // Isolate the protocol rejection from InputHandler's optional slow-handler
    // logger. Source's default logger also creates an unhandled diagnostic
    // rejection; that separate logger contract is not suppressed by this test.
    js_sys::Reflect::set(&live.terminal.options(), &"logLevel".into(), &"off".into()).unwrap();
    js_sys::Reflect::set(
        &get(&get(&source.0, "t"), "options"),
        &"logLevel".into(),
        &"off".into(),
    )
    .unwrap();
    let _bitmap = Patch::new(
        js_sys::global().into(),
        "createImageBitmap",
        eval("(()=>Promise.reject('bad bitmap'))"),
    );
    let _warning = Patch::new(get(&js_sys::global(), "console"), "warn", thrower(&token));
    let text = format!(
        "{}NEVER",
        sequence(PNG, "width=14px;height=10px;preserveAspectRatio=0")
    );
    assert!(live.begin(&text));
    assert!(source.begin(&text));
    let pending = live
        .terminal
        .shared_native()
        .take_pending()
        .unwrap()
        .unwrap();
    assert!(to_js_error(&pending.resolve().await.unwrap_err()) == token);
    let promise = call(&source.0, "finish", &[]).unchecked_into::<js_sys::Promise>();
    assert!(
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .unwrap_err()
            == token
    );
    same_state(&live, &source);
    assert_eq!(live.added.get(), 0);
}

#[wasm_bindgen_test(async)]
async fn immediate_storage_error_propagates_but_bitmap_storage_error_is_caught() {
    for asynchronous in [false, true] {
        let live = Live::new(true);
        let source = Source::new(true);
        let token = eval("({token:'onImageAdded failure'})");
        let cause = token.clone();
        live.storage
            .set_on_image_added(Some(std::rc::Rc::new(move || {
                Err(StorageError::Event(callback_error(cause.clone())))
            })));
        js_sys::Reflect::set(
            &get(&source.0, "store"),
            &"onImageAdded".into(),
            &thrower(&token),
        )
        .unwrap();
        let (record, _warning) = warnings();
        let (bytes, dimensions) = if asynchronous {
            (PNG.to_vec(), "width=14px;height=10px;preserveAspectRatio=0")
        } else {
            (tiny_qoi(), "width=2px;height=2px;preserveAspectRatio=0")
        };
        let text = format!("{}Z", sequence(&bytes, dimensions));
        if asynchronous {
            assert!(live.begin(&text));
            assert!(source.begin(&text));
            live.finish(true).await;
            source.finish().await;
            let warnings = js_sys::Array::from(&record);
            assert_eq!(warnings.length(), 2);
            for value in warnings.iter() {
                assert!(js_sys::Array::from(&value).get(1) == token);
            }
        } else {
            assert!(to_js_error(&live.terminal.shared_native().write(&text).unwrap_err()) == token);
            assert!(try_call(&source.0, "begin", &[text.into()]).unwrap_err() == token);
            assert_eq!(js_sys::Array::from(&record).length(), 0);
        }
        same_state(&live, &source);
        assert_eq!(
            live.storage.image_count(),
            1,
            "insertion precedes throwing event"
        );
    }
}

#[wasm_bindgen_test(async)]
async fn pending_real_bitmap_inserts_after_protocol_reset_storage_disposal_and_unregister() {
    let live = Live::new(true);
    let source = Source::new(true);
    // Save the real browser function. Each controller gets its own real bitmap.
    let original = get(&js_sys::global(), "createImageBitmap");
    let deferred =
        eval("({resolvers:[],call(){return new Promise(resolve=>this.resolvers.push(resolve))}})");
    let replacement = js_sys::Function::new_with_args("d", "return (...args)=>d.call(...args)")
        .call1(&JsValue::UNDEFINED, &deferred)
        .unwrap();
    let patch = Patch::new(js_sys::global().into(), "createImageBitmap", replacement);
    let text = format!(
        "{}Z",
        sequence(PNG, "width=14px;height=10px;preserveAspectRatio=0")
    );
    assert!(live.begin(&text));
    assert!(source.begin(&text));
    live.protocol.reset();
    call(&source.0, "reset", &[]);
    live.storage.dispose().unwrap();
    call(&get(&source.0, "store"), "dispose", &[]);
    live.registration.dispose();
    call(&get(&source.0, "registration"), "dispose", &[]);
    drop(patch);
    let factory = js_sys::Function::new_with_args(
        "original,bytes",
        r#"return original(new Blob([bytes],{type:'image/png'}),{resizeWidth:14,resizeHeight:10})"#,
    );
    let resolvers = js_sys::Array::from(&get(&deferred, "resolvers"));
    assert_eq!(resolvers.length(), 2);
    for index in 0..2 {
        let image = wasm_bindgen_futures::JsFuture::from(
            factory
                .call2(
                    &JsValue::UNDEFINED,
                    &original,
                    &js_sys::Uint8Array::from(PNG),
                )
                .unwrap()
                .unchecked_into::<js_sys::Promise>(),
        )
        .await
        .unwrap();
        resolvers
            .get(index)
            .unchecked_into::<js_sys::Function>()
            .call1(&JsValue::UNDEFINED, &image)
            .unwrap();
    }
    live.finish(true).await;
    source.finish().await;
    same_state(&live, &source);
    assert_eq!(live.added.get(), 1);
    assert_eq!(pixels(&live.image(0, 0)), pixels(&source.image(0, 0)));
}

#[wasm_bindgen_test(async)]
async fn pending_bitmap_keeps_terminal_alive_until_storage_commit_without_parser_cycle() {
    use std::rc::Rc;
    use xterm_addons::image::{
        iip::types::IipOptions,
        storage::{ImageStorage, ImageStorageOptions, TerminalStorageHost},
    };
    use xterm_web::{
        image::{BrowserIipProtocol, ImageRendererLifecycle},
        Terminal,
    };
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
    let protocol = BrowserIipProtocol::new(
        &terminal,
        owner.renderer(),
        storage.clone(),
        IipOptions::default(),
    )
    .unwrap();
    let registration = protocol.register().unwrap();
    assert!(matches!(
        native.write(&sequence(
            PNG,
            "width=14px;height=10px;preserveAspectRatio=0"
        )),
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
        .expect("promise must retain terminal context")
        .dispose();
    assert!(pending.resolve().await.unwrap());
    assert_eq!(
        storage.image_count(),
        1,
        "pending bitmap must commit after owner drops"
    );
    // Public buffer/parser JS wrappers can retain the native engine until GC.
    // The protocol's weak browser-terminal context must nevertheless be gone.
    assert!(
        protocol.register().is_err(),
        "no browser terminal cycle may remain"
    );
    storage.dispose().unwrap();
    owner.dispose().unwrap();
}
