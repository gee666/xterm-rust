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
    sixel::{DecoderOptions, SixelDecoder, ADDON_PALETTE},
    storage::ImageStorage,
};
impl State {
    pub(super) fn register_protocols(self: &Rc<Self>, terminal: &Terminal) -> Result<(), JsValue> {
        let options = self.options.get();
        if options.sixel_support {
            let (renderer, storage) = self.protocol_context()?;
            // Source evaluates *4 before reading the decoder palette option.
            let memory_limit = super::super::pixel_limit::memory(&self.pixel_limit)?;
            let palette = self.palette.get();
            let current_palette = self.palette.clone();
            let protocol = BrowserSixelProtocol::new_with_raw_options(
                terminal,
                renderer.renderer(),
                storage.clone(),
                Rc::clone(&self.options),
                self.sixel_size_limit.clone(),
                Rc::new(move || current_palette.get()),
            );
            // Native decoder initialization still crosses a promise boundary.
            // Data arriving before completion is discarded, never replayed.
            let loading = protocol.clone();
            let _promise = wasm_bindgen_futures::future_to_promise(async move {
                wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::UNDEFINED))
                    .await?;
                let decoder = SixelDecoder::new(DecoderOptions {
                    memory_limit: Some(memory_limit),
                    palette: ADDON_PALETTE.to_vec(),
                    palette_limit: super::palette::initial(&palette)?,
                    ..DecoderOptions::default()
                })
                .map_err(|e| to_js_error(&e))?;
                loading.complete_loading(decoder);
                Ok(JsValue::UNDEFINED)
            });
            self.add_handler(Protocol::Sixel(protocol.clone()))?;
            self.cleanups.borrow_mut().push(token(protocol.register()?));
        }
        if self.options.get().iip_support {
            let (renderer, storage) = self.protocol_context()?;
            // Conversion may reset/dispose the addon. Read live typed options
            // afterwards, with no state borrow held across the host callback.
            let encoded_limit = super::options::iip_encoded_limit(&self.iip_size_limit)?;
            let options = self.options.get();
            let protocol = BrowserIipProtocol::new_with_raw_pixel_limit(
                terminal,
                renderer.renderer(),
                storage.clone(),
                IipOptions {
                    pixel_limit: options.pixel_limit,
                    size_limit: options.iip_size_limit,
                },
                encoded_limit,
                self.pixel_limit.clone(),
            )?;
            self.add_handler(Protocol::Iip(protocol.clone()))?;
            self.cleanups.borrow_mut().push(token(protocol.register()?));
        }
        if self.options.get().kitty_support {
            let (renderer, storage) = self.protocol_context()?;
            // Source first coerces this raw value in the enabled handler's
            // constructor. Keep state borrows released across user conversion.
            let encoded_limit = super::options::kitty_encoded_limit(&self.kitty_size_limit)?;
            let options = self.options.get();
            let protocol = BrowserKittyProtocol::new_with_raw_pixel_limit(
                terminal,
                renderer.renderer(),
                storage,
                KittyOptions {
                    pixel_limit: options.pixel_limit,
                    size_limit: options.kitty_size_limit,
                },
                encoded_limit,
                self.pixel_limit.clone(),
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
    /// Each source constructor evaluates its renderer/storage arguments before
    /// its option conversion. A prior conversion may have reactivated the addon.
    fn protocol_context(&self) -> Result<(Rc<ImageRendererLifecycle>, ImageStorage), JsValue> {
        let renderer = self
            .renderer()
            .ok_or_else(|| js_sys::Error::new("Image renderer is not initialized"))?;
        let storage = self
            .storage()
            .ok_or_else(|| js_sys::Error::new("Image storage is not initialized"))?;
        Ok((renderer, storage))
    }
}
