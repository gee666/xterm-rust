// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Shared browser atlas ownership. The binding runtime, not each renderer, owns
//! this pool so compatible terminals share canvas pages and upload versions.
use super::{
    cache::{AtlasCache, CacheError},
    config::{AtlasDimensions, AtlasOptions},
    texture_atlas::{AtlasRuntime, TextureAtlas},
};
use crate::theme::ColorSet;
use std::rc::{Rc, Weak};
use wasm_bindgen::JsValue;
use web_sys::Document;
use xterm_core::{log_service::LogService, unicode_service::UnicodeService};

/// Validated terminal values used to generate a fresh configuration on every
/// acquisition. Color/cache identities remain shared, while the ANSI list is
/// copied by ConfigFactory as in CharAtlasUtils.generateConfig.
pub struct AtlasRequest<'a> {
    pub dimensions: AtlasDimensions,
    pub options: &'a AtlasOptions,
    pub colors: &'a ColorSet,
    pub custom_glyphs: bool,
}

/// Services are resolved only on a cache miss, after any previous atlas release.
/// Supplying a closure preserves that phase and permits fallible host lookup or
/// callback reentry without holding the pool's RefCell borrow.
pub struct AtlasServices {
    pub unicode: Rc<UnicodeService>,
    pub log: Rc<LogService>,
}

/// One instance per browser binding runtime. T is the terminal's stable Rc
/// identity, not a renderer identity: resizing or replacing a renderer must not
/// create another owner. Each renderer explicitly releases on disposal/context
/// restoration through remove_terminal. Dropping T alone does not dispose an
/// atlas or run user events; a retained Weak identity still permits release.
///
/// The document is the binding's source-global document, not the terminal's
/// current parent-window document. This matches CharAtlasCache's construction
/// of TextureAtlas with `document`, including terminals hosted in iframes.
pub struct BrowserAtlasCache<T: ?Sized> {
    document: Document,
    runtime: Rc<AtlasRuntime>,
    cache: AtlasCache<T, TextureAtlas>,
}
impl<T: ?Sized> BrowserAtlasCache<T> {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            runtime: Rc::new(AtlasRuntime::default()),
            cache: AtlasCache::new(),
        }
    }

    /// Shared limits, NULL identities and monotonically increasing upload clock.
    /// A renderer initializes the limits on this instance, never a private copy.
    pub fn runtime(&self) -> &Rc<AtlasRuntime> {
        &self.runtime
    }

    pub fn acquire(
        &self,
        terminal: &Rc<T>,
        request: AtlasRequest<'_>,
        services: impl FnOnce() -> Result<AtlasServices, JsValue>,
    ) -> Result<Rc<TextureAtlas>, CacheError<JsValue>> {
        let config = self.runtime.config.generate(
            request.dimensions,
            request.options,
            request.colors,
            request.custom_glyphs,
        );
        self.cache.acquire(terminal, config, |config| {
            let services = services()?;
            TextureAtlas::new(
                &self.document,
                config,
                self.runtime.clone(),
                services.unicode,
                services.log,
            )
        })
    }

    /// Only the final owner disposes the actual atlas. Disposal errors propagate
    /// and leave the entry owned, including mutations from reentrant callbacks.
    pub fn remove_terminal(&self, terminal: &Weak<T>) -> Result<(), CacheError<JsValue>> {
        self.cache.remove_terminal(terminal)
    }
}
