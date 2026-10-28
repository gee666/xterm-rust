// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from browser/services/KeyboardService.ts.

use xterm_core::input::keyboard::{evaluate_keyboard_event, KeyboardEvent, KeyboardResult};
use xterm_core::input::kitty_keyboard::{self, flags, EventType};
use xterm_core::input::win32_input_mode::Win32InputMode;

/// A live mode snapshot. Extension options and enabled terminal modes are
/// separate gates; enabling one without the other must not select a protocol.
#[derive(Clone, Copy, Debug, Default)]
pub struct KeyboardModes {
    pub win32_extension: bool,
    pub win32_mode: bool,
    pub kitty_extension: bool,
    pub kitty_flags: u32,
    pub application_cursor_keys: bool,
    pub is_mac: bool,
    pub mac_option_is_meta: bool,
}

impl KeyboardModes {
    pub fn use_win32(self) -> bool {
        self.win32_extension && self.win32_mode
    }
    pub fn use_kitty(self) -> bool {
        self.kitty_extension && kitty_keyboard::should_use_protocol(self.kitty_flags)
    }

    /// Source-ordered, fallible counterpart for mutable browser event fields.
    pub fn try_key_down<E: super::keyboard_dispatch_types::KeyEventSource + ?Sized>(
        self,
        event: &E,
    ) -> Result<KeyboardResult, E::Error> {
        if self.use_win32() {
            Win32InputMode.try_evaluate_keyboard_event(event, true)
        } else if self.use_kitty() {
            kitty_keyboard::try_evaluate(
                event,
                self.kitty_flags,
                if event.repeat()? {
                    EventType::Repeat
                } else {
                    EventType::Press
                },
                self.is_mac && self.mac_option_is_meta,
            )
        } else {
            xterm_core::input::keyboard::try_evaluate_keyboard_event(
                event,
                self.application_cursor_keys,
                self.is_mac,
                self.mac_option_is_meta,
            )
        }
    }

    pub fn try_key_up<E: super::keyboard_dispatch_types::KeyEventSource + ?Sized>(
        self,
        event: &E,
    ) -> Result<Option<KeyboardResult>, E::Error> {
        if self.use_win32() {
            Win32InputMode
                .try_evaluate_keyboard_event(event, false)
                .map(Some)
        } else if self.use_kitty() && self.kitty_flags & flags::REPORT_EVENT_TYPES != 0 {
            kitty_keyboard::try_evaluate(
                event,
                self.kitty_flags,
                EventType::Release,
                self.is_mac && self.mac_option_is_meta,
            )
            .map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn key_down(self, event: &KeyboardEvent<'_>, repeat: bool) -> KeyboardResult {
        if self.use_win32() {
            Win32InputMode.evaluate_keyboard_event(event, true)
        } else if self.use_kitty() {
            kitty_keyboard::evaluate(
                event,
                self.kitty_flags,
                if repeat {
                    EventType::Repeat
                } else {
                    EventType::Press
                },
                self.is_mac && self.mac_option_is_meta,
            )
        } else {
            evaluate_keyboard_event(
                event,
                self.application_cursor_keys,
                self.is_mac,
                self.mac_option_is_meta,
            )
        }
    }

    pub fn key_up(self, event: &KeyboardEvent<'_>) -> Option<KeyboardResult> {
        if self.use_win32() {
            Some(Win32InputMode.evaluate_keyboard_event(event, false))
        } else if self.use_kitty() && self.kitty_flags & flags::REPORT_EVENT_TYPES != 0 {
            Some(kitty_keyboard::evaluate(
                event,
                self.kitty_flags,
                EventType::Release,
                self.is_mac && self.mac_option_is_meta,
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event<'a>(key: &'a [u16], code: &'a [u16]) -> KeyboardEvent<'a> {
        KeyboardEvent {
            key_code: 65,
            key,
            code,
            ..KeyboardEvent::default()
        }
    }
    #[test]
    fn win32_priority_and_double_gating_matrix() {
        let key = vec![97];
        let code: Vec<u16> = "KeyA".encode_utf16().collect();
        let event = event(&key, &code);
        for extension in [false, true] {
            for mode in [false, true] {
                let config = KeyboardModes {
                    win32_extension: extension,
                    win32_mode: mode,
                    kitty_extension: true,
                    kitty_flags: flags::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
                    ..KeyboardModes::default()
                };
                assert_eq!(config.use_win32(), extension && mode);
                let expected = if extension && mode {
                    Win32InputMode.evaluate_keyboard_event(&event, true)
                } else {
                    kitty_keyboard::evaluate(&event, config.kitty_flags, EventType::Press, false)
                };
                assert_eq!(config.key_down(&event, false), expected);
                if extension && mode {
                    assert_eq!(
                        config.key_up(&event),
                        Some(Win32InputMode.evaluate_keyboard_event(&event, false))
                    );
                }
            }
        }
    }
    #[test]
    fn kitty_repeat_release_and_legacy_fallback() {
        let key: Vec<u16> = "ArrowUp".encode_utf16().collect();
        let event = event(&key, &key);
        for extension in [false, true] {
            for flags_value in [
                0,
                flags::DISAMBIGUATE_ESCAPE_CODES,
                flags::REPORT_EVENT_TYPES,
            ] {
                let config = KeyboardModes {
                    kitty_extension: extension,
                    kitty_flags: flags_value,
                    application_cursor_keys: true,
                    ..KeyboardModes::default()
                };
                assert_eq!(config.use_kitty(), extension && flags_value != 0);
                let expected = if config.use_kitty() {
                    kitty_keyboard::evaluate(&event, flags_value, EventType::Repeat, false)
                } else {
                    evaluate_keyboard_event(&event, true, false, false)
                };
                assert_eq!(config.key_down(&event, true), expected);
                assert_eq!(
                    config.key_up(&event).is_some(),
                    extension && flags_value & flags::REPORT_EVENT_TYPES != 0
                );
            }
        }
    }
    #[test]
    fn mac_option_is_meta_passed_only_on_mac_for_kitty() {
        let key: Vec<u16> = "å".encode_utf16().collect();
        let code: Vec<u16> = "KeyA".encode_utf16().collect();
        let event = KeyboardEvent {
            alt_key: true,
            ..event(&key, &code)
        };
        for mac in [false, true] {
            let config = KeyboardModes {
                is_mac: mac,
                mac_option_is_meta: true,
                kitty_extension: true,
                kitty_flags: flags::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
                ..KeyboardModes::default()
            };
            assert_eq!(
                config.key_down(&event, false),
                kitty_keyboard::evaluate(&event, config.kitty_flags, EventType::Press, mac)
            );
        }
    }
}
