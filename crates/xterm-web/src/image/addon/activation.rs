// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{controls, State};
use crate::{
    image::{js, ImageRendererLifecycle},
    js_error::to_js_error,
    terminal::Terminal,
};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use xterm_addons::image::storage::{ImageStorage, StorageDiagnostic, TerminalStorageHost};
impl State {
    pub(super) fn activate(self: &Rc<Self>, terminal: &Terminal) -> Result<(), JsValue> {
        *self.terminal.borrow_mut() = Rc::downgrade(&terminal.shared);
        let renderer = Rc::new(ImageRendererLifecycle::new(terminal)?);
        *self.renderer.borrow_mut() = Some(Rc::clone(&renderer));
        let host = TerminalStorageHost::from_terminal(
            &terminal.shared.native,
            Rc::new(|level, message| {
                let console = js::get(&js_sys::global(), "console").map_err(js::error)?;
                js::call(
                    &console,
                    match level {
                        StorageDiagnostic::Error => "error",
                        StorageDiagnostic::Warning => "warn",
                    },
                    &[message.into()],
                )
                .map_err(js::error)?;
                Ok(())
            }),
        )
        .map_err(|e| to_js_error(&e))?;
        let limit = self.storage_limit.borrow().clone();
        let pixels = super::storage_limit::initial_pixels(&limit)?;
        let storage = ImageStorage::new_with_pixel_limit(
            Rc::new(host),
            renderer.renderer(),
            Rc::clone(&self.placeholder),
            pixels,
        );
        *self.storage.borrow_mut() = Some(storage.clone());
        self.inspect_storage(storage.clone())?;
        let weak = Rc::downgrade(self);
        storage.set_on_image_added(Some(Rc::new(move || {
            if let Some(state) = weak.upgrade() {
                state
                    .added
                    .fire(&())
                    .map_err(xterm_addons::image::storage::StorageError::Event)?;
            }
            Ok(())
        })));
        if self.options.get().enable_size_reports {
            controls::enable_reports(terminal)?;
        }
        let cleanups = self.parent_registrations(terminal, renderer.clone(), storage.clone())?;
        self.cleanups.borrow_mut().extend(cleanups);
        self.register_protocols(terminal, renderer, storage)
    }
}
