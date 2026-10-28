// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowReport {
    TextAreaPixels,
    CellPixels,
}

/// Ordered, owned notifications. UTF-16 payloads retain unpaired surrogates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalEvent {
    Bell,
    RequestReset,
    SyncScrollBar,
    SendFocus,
    ColorSchemeQuery,
    Color(Vec<super::ColorRequest>),
    WindowReport(WindowReport),
    LineFeed,
    CursorMove,
    Scroll(usize),
    /// InputHandler-only scroll notification. Browser selection/viewport refresh
    /// consumes this; it is not a public headless onScroll event.
    InputScroll(usize),
    Resize {
        cols: usize,
        rows: usize,
    },
    Render {
        start: usize,
        end: usize,
    },
    WriteParsed,
    Data(Vec<u16>),
    Binary(Vec<u16>),
    TitleChange(Vec<u16>),
    AccessibilityChar(Vec<u16>),
    AccessibilityTab(usize),
}
