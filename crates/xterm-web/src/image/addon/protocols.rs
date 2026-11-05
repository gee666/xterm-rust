// Copyright (c) 2020 The xterm.js authors. MIT license.
use super::{hooks::token, Cleanup, Protocol, State};
use crate::{
    image::{
        BrowserIipProtocol, BrowserKittyProtocol, BrowserSixelProtocol, ImageRendererLifecycle,
    },
    js_error::to_js_error,
    terminal::Terminal,
};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use xterm_addons::image::{
    iip::types::IipOptions,
    kitty::handler::KittyOptions,
    sixel::{SixelDecoder, SixelHandler},
    storage::ImageStorage,
};
impl State {
    pub(super) fn register_protocols(
        self: &Rc<Self>,
        terminal: &Terminal,
        renderer: Rc<ImageRendererLifecycle>,
        storage: ImageStorage,
    ) -> Result<(), JsValue> {
        let options = self.options.get();
        if options.sixel_support {
            let protocol = BrowserSixelProtocol::new(
                terminal,
                renderer.renderer(),
                storage.clone(),
                Rc::clone(&self.options),
            );
            // Native decoder initialization still crosses a promise boundary.
            // Data arriving before completion is discarded, never replayed.
            let loading = protocol.clone();
            let _promise = wasm_bindgen_futures::future_to_promise(async move {
                wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::UNDEFINED))
                    .await?;
                let decoder = SixelDecoder::new(SixelHandler::decoder_options(
                    options.pixel_limit,
                    options.sixel_palette_limit,
                ))
                .map_err(|e| to_js_error(&e))?;
                loading.complete_loading(decoder);
                Ok(JsValue::UNDEFINED)
            });
            self.add_handler(Protocol::Sixel(protocol.clone()))?;
            self.cleanups.borrow_mut().push(token(protocol.register()?));
        }
        if self.options.get().iip_support {
            let options = self.options.get();
            let protocol = BrowserIipProtocol::new(
                terminal,
                renderer.renderer(),
                storage.clone(),
                IipOptions {
                    pixel_limit: options.pixel_limit,
                    size_limit: options.iip_size_limit,
                },
            )?;
            self.add_handler(Protocol::Iip(protocol.clone()))?;
            self.cleanups.borrow_mut().push(token(protocol.register()?));
        }
        if self.options.get().kitty_support {
            let options = self.options.get();
            let protocol = BrowserKittyProtocol::new(
                terminal,
                renderer.renderer(),
                storage,
                KittyOptions {
                    pixel_limit: options.pixel_limit,
                    size_limit: options.kitty_size_limit,
                },
            );
            self.add_handler(Protocol::Kitty(protocol.clone()))?;
            let cleanup: Cleanup = Rc::new({
                let protocol = protocol.clone();
                move || protocol.dispose()
            });
            let registration = protocol.register()?;
            self.cleanups
                .borrow_mut()
                .extend([cleanup, token(registration)]);
        }
        Ok(())
    }
}
