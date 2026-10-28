// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Contracts for CoreBrowserTerminal keyboard dispatch.

use super::keyboard::KeyboardModes;
use xterm_core::input::keyboard::{KeyboardEvent, KeyboardResult};
#[path = "keyboard_event.rs"]
mod event;
pub use event::{EventNumber, KeyEventSource, StaticKeyEvent};

/// DOM event category. The adapter supplies the actual event's category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyEventKind {
    Down,
    Up,
    Press,
}

/// DOM fields beyond the shared protocol encoder's input. Optional character
/// codes retain the distinction between an absent property and numeric zero.
#[derive(Clone, Copy, Debug)]
pub struct DispatchKeyEvent<'a> {
    pub key: KeyboardEvent<'a>,
    pub kind: KeyEventKind,
    pub repeat: bool,
    pub alt_graph: bool,
    pub char_code: Option<u32>,
    pub which: Option<u32>,
}

/// Live values are sampled again after callbacks, not cached across dispatch.
#[derive(Clone, Copy, Debug, Default)]
pub struct KeyboardSnapshot {
    pub modes: KeyboardModes,
    /// Browser-service override, distinct from KeyboardModes' platform encoder.
    pub is_mac: bool,
    pub is_windows: bool,
    pub screen_reader_mode: bool,
    pub scroll_on_user_input: bool,
    pub rows: u32,
    pub ybase: u32,
    pub ydisp: u32,
}

/// Effects execute synchronously and in order. In particular, OnKey precedes
/// ShowCursor and Data, and callback errors stop subsequent effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyboardAction<'a> {
    UpdateCursorStyle,
    ScrollLines(i64),
    ScrollToBottom,
    SelectAll,
    Cancel,
    ClearTextarea,
    OnKey(&'a [u16]),
    ShowCursor,
    Data { key: &'a [u16], user_input: bool },
    Focus,
}

/// Fallible reads shared by the dispatcher and its DOM host. Defaults suit
/// immutable test models; DOM adapters override individual live source reads.
pub trait KeyboardReadHost {
    type Error;

    fn snapshot(&self) -> Result<KeyboardSnapshot, Self::Error>;
    // Defaults suit pure models. DOM adapters override these individual reads
    // so getters are not evaluated early or on short-circuited source paths.
    fn is_mac(&self) -> Result<bool, Self::Error> {
        Ok(self.snapshot()?.is_mac)
    }
    fn is_windows(&self) -> Result<bool, Self::Error> {
        Ok(self.snapshot()?.is_windows)
    }
    fn mac_option_is_meta(&self) -> Result<bool, Self::Error> {
        Ok(self.snapshot()?.modes.mac_option_is_meta)
    }
    fn scroll_on_user_input(&self) -> Result<bool, Self::Error> {
        Ok(self.snapshot()?.scroll_on_user_input)
    }
    fn scroll_position(&self) -> Result<(u32, u32), Self::Error> {
        let snapshot = self.snapshot()?;
        Ok((snapshot.ybase, snapshot.ydisp))
    }
    fn rows(&self) -> Result<u32, Self::Error> {
        Ok(self.snapshot()?.rows)
    }
    fn screen_reader_mode(&self) -> Result<bool, Self::Error> {
        Ok(self.snapshot()?.screen_reader_mode)
    }
    fn use_kitty(&self) -> Result<bool, Self::Error> {
        Ok(self.snapshot()?.modes.use_kitty())
    }
    fn use_win32(&self) -> Result<bool, Self::Error> {
        Ok(self.snapshot()?.modes.use_win32())
    }
    fn alt_graph_state(
        &self,
        event: &dyn KeyEventSource<Error = Self::Error>,
    ) -> Result<bool, Self::Error> {
        event.alt_graph()
    }
    fn evaluate_key_down(
        &self,
        event: &dyn KeyEventSource<Error = Self::Error>,
    ) -> Result<KeyboardResult, Self::Error> {
        self.snapshot()?.modes.try_key_down(event)
    }
    fn evaluate_key_up(
        &self,
        event: &dyn KeyEventSource<Error = Self::Error>,
    ) -> Result<Option<KeyboardResult>, Self::Error> {
        self.snapshot()?.modes.try_key_up(event)
    }
}

/// The DOM adapter must release RefCell borrows before invoking callbacks.
/// Shared-reference hooks allow synchronous reentry through an Rc.
pub trait KeyboardHost: KeyboardReadHost {
    /// Returns true when no custom handler exists or it does not veto.
    fn custom_key_handler(
        &self,
        event: &dyn KeyEventSource<Error = Self::Error>,
    ) -> Result<bool, Self::Error>;
    /// Called only when Mac Option-as-Meta does not bypass composition.
    fn composition_key_down(
        &self,
        event: &dyn KeyEventSource<Error = Self::Error>,
    ) -> Result<bool, Self::Error>;
    /// Cancel means preventDefault followed by stopPropagation. The original
    /// event is absent only for input-event text, which has no keyboard event.
    fn perform(
        &self,
        event: Option<&dyn KeyEventSource<Error = Self::Error>>,
        action: KeyboardAction<'_>,
    ) -> Result<(), Self::Error>;
}

/// Fields read by CoreBrowserTerminal's input-event path.
#[derive(Clone, Copy, Debug)]
pub struct TextInputEvent<'a> {
    pub data: Option<&'a [u16]>,
    pub insert_text: bool,
    pub composed: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DispatchState {
    pub key_down_handled: bool,
    pub key_down_seen: bool,
    pub key_press_handled: bool,
    pub unprocessed_dead_key: bool,
}
