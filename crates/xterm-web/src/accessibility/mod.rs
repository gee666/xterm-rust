// Copyright (c) 2017-2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. AccessibilityManager and LocalizableStrings contracts.

mod announcements;
pub mod localization;
mod selection;
pub use announcements::{AnnouncementError, Announcements, CharacterAction, MAX_ROWS_TO_READ};
pub use localization::{LocalizableStrings, LocalizationError};
pub use selection::{map_point, selection_range, CellPoint, InvalidRange, SelectionRange};

#[cfg(target_arch = "wasm32")]
pub mod browser;
#[cfg(target_arch = "wasm32")]
mod browser_rows;
#[cfg(target_arch = "wasm32")]
mod browser_selection;
#[cfg(target_arch = "wasm32")]
pub use browser::{AccessibilityEvent, AccessibilityHost, AccessibilityManager, Dimensions, Row};
