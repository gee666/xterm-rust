// Copyright (c) 2023 The xterm.js authors. MIT license.
#[path = "../image_terminal_support/mod.rs"]
mod common;
pub use common::*;
use xterm_addons::image::iip::types::IipOptions;
use xterm_web::image::BrowserIipProtocol;

pub type Live = common::Live<BrowserIipProtocol>;
impl Live {
    pub fn new(open: bool) -> Self {
        Self::with_protocol(open, |terminal, renderer, storage| {
            let protocol =
                BrowserIipProtocol::new(terminal, renderer, storage, IipOptions::default())
                    .unwrap();
            let token = protocol.register().unwrap();
            (protocol, token)
        })
    }
}
impl Source {
    pub fn new(open: bool) -> Self {
        Self::with_protocol(open, "({iipSizeLimit:33554432})",
            "const h=new SourceIIPHandler(opts,renderer,new SourceIIPImageStorage(store),t); const registration=t._core._inputHandler._parser.registerOscHandler(1337,h); return {h,registration};")
    }
}
pub const PNG: &[u8] =
    include_bytes!("../../../xterm-addons/tests/fixtures/image-metrics/w3c_home.png");
pub fn sequence(bytes: &[u8], dimensions: &str) -> String {
    let binary: String = bytes.iter().map(|&b| char::from(b)).collect();
    let encoded = call(&js_sys::global(), "btoa", &[binary.into()])
        .as_string()
        .unwrap();
    format!("\x1b]1337;File=inline=1;{dimensions}:{encoded}\x1b\\")
}
