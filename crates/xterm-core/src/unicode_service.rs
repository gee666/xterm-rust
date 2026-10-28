// Copyright (c) 2019, 2021 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of UnicodeService.ts and UnicodeApi.ts.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::event::{EmitError, Emitter, Event};
use crate::lifecycle::Disposable;
use crate::unicode::{UnicodeCharProperties, UnicodeCharWidth, UnicodeVersionProvider};

/// Recoverable failures from provider selection, callbacks or width accumulation.
#[derive(Debug, Eq, PartialEq)]
pub enum UnicodeServiceError {
    NoActiveProvider,
    UnknownVersion(String),
    Event(EmitError),
    WidthOverflow,
}

impl fmt::Display for UnicodeServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoActiveProvider => f.write_str("no active Unicode provider"),
            Self::UnknownVersion(version) => write!(f, "unknown Unicode version \"{version}\""),
            Self::Event(error) => error.fmt(f),
            Self::WidthOverflow => {
                f.write_str("string cell width exceeds the signed integer range")
            }
        }
    }
}

impl std::error::Error for UnicodeServiceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Event(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Default)]
struct State {
    providers: Vec<(String, Rc<dyn UnicodeVersionProvider>)>,
    active_version: String,
    active_provider: Option<Rc<dyn UnicodeVersionProvider>>,
}

/// Unicode provider registry and UTF-16 string-width calculation.
///
/// Starts empty, as upstream. Registering the first provider selects it. The
/// terminal, rather than this service, is responsible for registering V6.
/// Methods release internal borrows before calling providers or event listeners.
/// Wrap the service in `Rc` and capture `Weak` in reentrant callbacks to avoid
/// reference cycles. Disposal clears listeners, not providers or active state.
#[derive(Default)]
pub struct UnicodeService {
    state: RefCell<State>,
    on_change: Emitter<String>,
}

impl UnicodeService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_change(&self) -> Event<String> {
        self.on_change.event()
    }

    /// Numeric array-index version names sort first, matching `Object.keys`.
    pub fn versions(&self) -> Vec<String> {
        let state = self.state.borrow();
        let mut indexed = Vec::new();
        let mut named = Vec::new();
        for (version, _) in &state.providers {
            if let Some(index) = array_index(version) {
                indexed.push((index, version.clone()));
            } else {
                named.push(version.clone());
            }
        }
        indexed.sort_unstable_by_key(|entry| entry.0);
        indexed
            .into_iter()
            .map(|entry| entry.1)
            .chain(named)
            .collect()
    }

    pub fn active_version(&self) -> String {
        self.state.borrow().active_version.clone()
    }

    pub fn set_active_version(&self, version: &str) -> Result<(), UnicodeServiceError> {
        {
            let mut state = self.state.borrow_mut();
            let provider = state
                .providers
                .iter()
                .find(|entry| entry.0 == version)
                .map(|entry| Rc::clone(&entry.1))
                .ok_or_else(|| UnicodeServiceError::UnknownVersion(version.to_owned()))?;
            state.active_provider = Some(provider);
            state.active_version = version.to_owned();
        }
        // Reassigning the same version still fires. State is committed first.
        self.on_change
            .fire(&version.to_owned())
            .map_err(UnicodeServiceError::Event)
    }

    pub fn register(
        &self,
        provider: Rc<dyn UnicodeVersionProvider>,
    ) -> Result<(), UnicodeServiceError> {
        let version = provider.version().to_owned();
        let activate = {
            let mut state = self.state.borrow_mut();
            if let Some(entry) = state.providers.iter_mut().find(|entry| entry.0 == version) {
                entry.1 = provider;
            } else {
                state.providers.push((version.clone(), provider));
            }
            state.active_version.is_empty()
        };
        // Replacing an active name does not replace the selected provider object
        // until the caller selects that name again. The reference stores both.
        if activate {
            self.set_active_version(&version)?;
        }
        Ok(())
    }

    pub fn wcwidth(&self, codepoint: u32) -> Result<UnicodeCharWidth, UnicodeServiceError> {
        Ok(self.active_provider()?.wcwidth(codepoint))
    }

    pub fn char_properties(
        &self,
        codepoint: u32,
        preceding: UnicodeCharProperties,
    ) -> Result<UnicodeCharProperties, UnicodeServiceError> {
        Ok(self
            .active_provider()?
            .char_properties(codepoint, preceding))
    }

    pub fn get_string_cell_width(&self, text: &str) -> Result<i64, UnicodeServiceError> {
        self.string_width(text.encode_utf16())
    }

    /// Unlike `str`, this overload represents lone UTF-16 surrogates losslessly.
    pub fn get_string_cell_width_utf16(&self, text: &[u16]) -> Result<i64, UnicodeServiceError> {
        self.string_width(text.iter().copied())
    }

    fn string_width(
        &self,
        mut units: impl Iterator<Item = u16>,
    ) -> Result<i64, UnicodeServiceError> {
        let mut result = 0_i64;
        let mut preceding = UnicodeCharProperties::default();
        while let Some(unit) = units.next() {
            let mut codepoint = u32::from(unit);
            if (0xd800..=0xdbff).contains(&unit) {
                let Some(second) = units.next() else {
                    return add_width(result, i64::from(self.wcwidth(codepoint)?.columns()));
                };
                if (0xdc00..=0xdfff).contains(&second) {
                    codepoint =
                        (u32::from(unit) - 0xd800) * 0x400 + u32::from(second) - 0xdc00 + 0x10000;
                } else {
                    // Upstream accounts for the second unit's standalone width
                    // before the first unit's joining properties, without retry.
                    result = add_width(
                        result,
                        i64::from(self.wcwidth(u32::from(second))?.columns()),
                    )?;
                }
            }
            let current = self.char_properties(codepoint, preceding)?;
            let mut width = i64::from(current.width());
            if current.should_join() {
                width -= i64::from(preceding.width());
            }
            result = add_width(result, width)?;
            preceding = current;
        }
        Ok(result)
    }

    fn active_provider(&self) -> Result<Rc<dyn UnicodeVersionProvider>, UnicodeServiceError> {
        self.state
            .borrow()
            .active_provider
            .clone()
            .ok_or(UnicodeServiceError::NoActiveProvider)
    }
}

impl Disposable for UnicodeService {
    fn dispose(&self) {
        self.on_change.dispose();
    }
}

impl Drop for UnicodeService {
    fn drop(&mut self) {
        self.on_change.dispose();
    }
}

fn add_width(total: i64, width: i64) -> Result<i64, UnicodeServiceError> {
    total
        .checked_add(width)
        .ok_or(UnicodeServiceError::WidthOverflow)
}

#[cfg(test)]
mod tests {
    use super::{add_width, UnicodeServiceError};

    #[test]
    fn signed_accumulation_reports_overflow_without_wrapping() {
        assert_eq!(add_width(i64::MAX, 0), Ok(i64::MAX));
        assert_eq!(add_width(i64::MIN, 0), Ok(i64::MIN));
        assert_eq!(add_width(3, -3), Ok(0));
        assert_eq!(
            add_width(i64::MAX, 1),
            Err(UnicodeServiceError::WidthOverflow)
        );
        assert_eq!(
            add_width(i64::MIN, -1),
            Err(UnicodeServiceError::WidthOverflow)
        );
    }
}

fn array_index(value: &str) -> Option<u32> {
    let index: u32 = value.parse().ok()?;
    (index != u32::MAX && index.to_string() == value).then_some(index)
}

/// Public Unicode API view. Proposed-API access gating belongs to the terminal.
/// This borrowed view cannot outlive its terminal's Unicode service.
#[derive(Clone, Copy)]
pub struct UnicodeApi<'a> {
    service: &'a UnicodeService,
}

impl<'a> UnicodeApi<'a> {
    pub fn new(service: &'a UnicodeService) -> Self {
        Self { service }
    }

    pub fn register(
        &self,
        provider: Rc<dyn UnicodeVersionProvider>,
    ) -> Result<(), UnicodeServiceError> {
        self.service.register(provider)
    }

    pub fn versions(&self) -> Vec<String> {
        self.service.versions()
    }

    pub fn active_version(&self) -> String {
        self.service.active_version()
    }

    pub fn set_active_version(&self, version: &str) -> Result<(), UnicodeServiceError> {
        self.service.set_active_version(version)
    }
}
