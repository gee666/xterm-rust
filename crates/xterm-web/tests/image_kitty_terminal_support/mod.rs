// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
#[path = "../image_terminal_support/mod.rs"]
mod common;
pub use common::*;
use wasm_bindgen::JsValue;
use xterm_addons::image::kitty::handler::KittyOptions;
use xterm_web::image::BrowserKittyProtocol;

pub type Live = common::Live<BrowserKittyProtocol>;
impl Live {
    pub fn new(open: bool) -> Self {
        Self::options(open, KittyOptions::default())
    }
    pub fn options(open: bool, options: KittyOptions) -> Self {
        Self::with_protocol(open, |terminal, renderer, storage| {
            js_sys::Reflect::set(&terminal.options(), &"logLevel".into(), &"off".into()).unwrap();
            let protocol = BrowserKittyProtocol::new(terminal, renderer, storage, options);
            let registration = protocol.register().unwrap();
            (protocol, registration)
        })
    }
}
impl Source {
    pub fn new(open: bool) -> Self {
        Self::options(open, "({kittySizeLimit:33554432})")
    }
    pub fn options(open: bool, options: &str) -> Self {
        Self::with_protocol(open, options,
            "t.options.logLevel='off'; const h=new SourceKittyGraphicsHandler(opts,renderer,new SourceKittyImageStorage(store),t); const registration=t._core._inputHandler._parser.registerApcHandler({final:'G'},h); return {h,registration};")
    }
}
pub const PNG: &[u8] =
    include_bytes!("../../../xterm-addons/tests/fixtures/image-metrics/w3c_home.png");
pub fn payload(bytes: &[u8]) -> String {
    let binary: String = bytes.iter().map(|&b| char::from(b)).collect();
    call(&js_sys::global(), "btoa", &[binary.into()])
        .as_string()
        .unwrap()
}
pub fn sequence(control: &str, bytes: &[u8]) -> String {
    format!("\x1b_G{control};{}\x1b\\", payload(bytes))
}
pub fn raw() -> Vec<u8> {
    (0..6 * 9)
        .flat_map(|n| {
            [
                ((n * 37) % 256) as u8,
                ((n * 17) % 256) as u8,
                ((n * 13) % 256) as u8,
                255,
            ]
        })
        .collect()
}
pub async fn same_maps(live: &Live, source: &Source) {
    let h = get(&source.0, "h");
    let images = call(&get(&h, "images"), "entries", &[]);
    let images = call(
        &js_sys::Array::from(&images),
        "map",
        &[eval(
            "e=>[e[0],e[1].data.size,e[1].width,e[1].height,e[1].format,e[1].compression]",
        )],
    );
    let rust = js_sys::Array::new();
    for entry in live.protocol.handler().images().iter() {
        let (id, record) = entry.unwrap();
        let m = record.bytes.borrow();
        let row = js_sys::Array::new();
        for value in [
            JsValue::from_f64(id),
            (m.data.len() as f64).into(),
            m.width.into(),
            m.height.into(),
            m.format.into(),
            js_sys::JsString::from_char_code(&m.compression).into(),
        ] {
            row.push(&value);
        }
        rust.push(&row);
    }
    assert_eq!(
        js_sys::JSON::stringify(&rust).unwrap(),
        js_sys::JSON::stringify(&images).unwrap()
    );
    for entry in live.protocol.handler().images().iter() {
        let (id, record) = entry.unwrap();
        let data = record.bytes.borrow().data.clone();
        let bytes = data.read().await.unwrap();
        let source_record = call(&get(&h, "images"), "get", &[id.into()]);
        let buffer = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(call(
            &get(&source_record, "data"),
            "arrayBuffer",
            &[],
        )))
        .await
        .unwrap();
        assert_eq!(bytes, js_sys::Uint8Array::new(&buffer).to_vec());
    }
    let placements = js_sys::Array::from(&call(&get(&h, "_kittyIdToStorageId"), "entries", &[]));
    let rust = js_sys::Array::new();
    for entry in live.protocol.handler().placements().iter() {
        let (id, storage_id) = entry.unwrap();
        rust.push(&js_sys::Array::of2(&id.into(), &(storage_id as f64).into()));
    }
    assert_eq!(
        js_sys::JSON::stringify(&rust).unwrap(),
        js_sys::JSON::stringify(&placements).unwrap()
    );
}
pub async fn compressed(bytes: &[u8], format: &str) -> Vec<u8> {
    let result = js_sys::Function::new_with_args("data,format", "return (async()=>{const stream=new CompressionStream(format);const writer=stream.writable.getWriter();writer.write(data);writer.close();return new Uint8Array(await new Response(stream.readable).arrayBuffer())})()")
        .call2(&JsValue::UNDEFINED, &js_sys::Uint8Array::from(bytes).into(), &format.into()).unwrap();
    let output = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(result))
        .await
        .unwrap();
    js_sys::Uint8Array::new(&output).to_vec()
}
