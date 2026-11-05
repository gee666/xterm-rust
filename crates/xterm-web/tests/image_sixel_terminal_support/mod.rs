// Copyright (c) 2020, 2023 The xterm.js authors. MIT license.
#[path = "../image_terminal_support/mod.rs"]
mod common;
pub use common::*;
use std::{cell::Cell, rc::Rc};
use xterm_addons::image::{
    options::ImageOptions,
    sixel::{SixelDecoder, SixelHandler},
};
use xterm_web::image::BrowserSixelProtocol;

pub type Live = common::Live<BrowserSixelProtocol>;
impl Live {
    pub fn new(open: bool, options: Rc<Cell<ImageOptions>>) -> Self {
        Self::with_protocol(open, |terminal, renderer, storage| {
            let protocol = BrowserSixelProtocol::new(terminal, renderer, storage, options);
            let registration = protocol.register().unwrap();
            (protocol, registration)
        })
    }
    pub fn load(&self, options: ImageOptions) {
        self.protocol.complete_loading(
            SixelDecoder::new(SixelHandler::decoder_options(
                options.pixel_limit,
                options.sixel_palette_limit,
            ))
            .unwrap(),
        );
    }
}
impl Source {
    pub fn new(open: bool) -> Self {
        Self::with_protocol(
            open,
            "({sixelScrolling:true,sixelPaletteLimit:4096,sixelSizeLimit:33554432})",
            r#"
            const h=new SourceSixelHandler(opts,new SourceSixelImageStorage(store,opts,renderer,t),t);
            let dec=h._dec;
            const ready=dec?Promise.resolve():new Promise(resolve=>Object.defineProperty(h,'_dec',{
                configurable:true,get(){return dec},set(value){dec=value;resolve()}
            }));
            const registration=t._core._inputHandler._parser.registerDcsHandler({final:'q'},h);
            return {h,registration,ready};"#,
        )
    }
    pub async fn ready(&self) {
        promise(get(&self.0, "ready")).await;
    }
}
pub async fn ready(open: bool) -> (Live, Source, Rc<Cell<ImageOptions>>) {
    let options = Rc::new(Cell::new(ImageOptions::default()));
    let live = Live::new(open, options.clone());
    let source = Source::new(open);
    source.ready().await;
    live.load(options.get());
    (live, source, options)
}
pub fn sequence(data: &str, transparent: bool) -> String {
    format!("\x1bP0;{}q{data}\x1b\\", u8::from(transparent))
}
pub fn source_option(source: &Source, name: &str, value: wasm_bindgen::JsValue) {
    js_sys::Reflect::set(&get(&source.0, "opts"), &name.into(), &value).unwrap();
}
