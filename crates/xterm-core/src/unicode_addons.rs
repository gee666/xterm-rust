// Copyright (c) 2019-2023 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ports Unicode11Addon.ts and UnicodeGraphemesAddon.ts.

use crate::public_api::TerminalAddon;
use crate::unicode11::UnicodeV11;
use crate::unicode_graphemes::UnicodeGraphemeProvider;
use crate::unicode_service::{UnicodeService, UnicodeServiceError};
use std::cell::RefCell;
use std::rc::Rc;

/// A terminal supplies its live Unicode service after enforcing API access gates.
/// Error conversion keeps terminal access failures and service failures visible.
pub trait UnicodeAddonHost {
    type Error: From<UnicodeServiceError>;
    fn unicode_service_for_addon(&self) -> Result<Rc<UnicodeService>, Self::Error>;
}

impl UnicodeAddonHost for Rc<UnicodeService> {
    type Error = UnicodeServiceError;

    fn unicode_service_for_addon(&self) -> Result<Rc<UnicodeService>, Self::Error> {
        Ok(Rc::clone(self))
    }
}

/// Unicode 11 activation registers a provider without selecting it.
#[derive(Debug, Default)]
pub struct Unicode11Addon;

impl Unicode11Addon {
    pub fn new() -> Self {
        Self
    }

    pub fn activate(&self, unicode: &UnicodeService) -> Result<(), UnicodeServiceError> {
        unicode.register(Rc::new(UnicodeV11::new()))
    }

    /// Upstream intentionally leaves the registered provider in the registry.
    pub fn dispose(&self) {}
}

impl<T: UnicodeAddonHost> TerminalAddon<T> for Unicode11Addon {
    type Error = T::Error;

    fn activate(&self, terminal: &T) -> Result<(), Self::Error> {
        let unicode = terminal.unicode_service_for_addon()?;
        Unicode11Addon::activate(self, unicode.as_ref()).map_err(Into::into)
    }

    fn dispose(&self) -> Result<(), Self::Error> {
        Unicode11Addon::dispose(self);
        Ok(())
    }
}

#[derive(Default)]
struct GraphemesState {
    plain: Option<Rc<UnicodeGraphemeProvider>>,
    graphemes: Option<Rc<UnicodeGraphemeProvider>>,
    unicode: Option<Rc<UnicodeService>>,
    old_version: String,
}

/// Registers both Unicode 15 providers and restores the old version on disposal.
///
/// Providers are cached across activation. The service is owned while attached,
/// as the JavaScript addon retains its terminal's Unicode API. Callbacks must
/// capture a Weak addon/service reference to avoid reference cycles.
#[derive(Default)]
pub struct UnicodeGraphemesAddon {
    state: RefCell<GraphemesState>,
}

impl<T: UnicodeAddonHost> TerminalAddon<T> for UnicodeGraphemesAddon {
    type Error = T::Error;

    fn activate(&self, terminal: &T) -> Result<(), Self::Error> {
        UnicodeGraphemesAddon::activate(self, terminal.unicode_service_for_addon()?)
            .map_err(Into::into)
    }

    fn dispose(&self) -> Result<(), Self::Error> {
        UnicodeGraphemesAddon::dispose(self).map_err(Into::into)
    }
}

impl UnicodeGraphemesAddon {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn activate(&self, unicode: Rc<UnicodeService>) -> Result<(), UnicodeServiceError> {
        let (plain, graphemes) = {
            let mut state = self.state.borrow_mut();
            let plain = Rc::clone(
                state
                    .plain
                    .get_or_insert_with(|| Rc::new(UnicodeGraphemeProvider::new(false))),
            );
            let graphemes = Rc::clone(
                state
                    .graphemes
                    .get_or_insert_with(|| Rc::new(UnicodeGraphemeProvider::new(true))),
            );
            state.unicode = Some(Rc::clone(&unicode));
            (plain, graphemes)
        };
        unicode.register(plain)?;
        unicode.register(graphemes)?;
        self.state.borrow_mut().old_version = unicode.active_version();
        unicode.set_active_version("15-graphemes")
    }

    /// Restore the remembered version, retaining registry entries and attachment.
    /// Repeated disposal re-selects and emits again, matching the source.
    pub fn dispose(&self) -> Result<(), UnicodeServiceError> {
        let restore = {
            let state = self.state.borrow();
            state
                .unicode
                .as_ref()
                .map(|service| (Rc::clone(service), state.old_version.clone()))
        };
        if let Some((unicode, version)) = restore {
            unicode.set_active_version(&version)?;
        }
        Ok(())
    }
}
