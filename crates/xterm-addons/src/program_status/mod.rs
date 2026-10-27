// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of addon-program-status.

//! OSC 7501 report parsing and immutable, insertion-ordered status snapshots.

mod addon;
mod parse;
mod store;
pub use addon::ProgramStatusAddon;
pub use parse::{parse_report, parse_report_utf16};
pub use store::ProgramStatusStore;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusState {
    Idle,
    Working,
    Done,
    Blocked,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Permission,
    Question,
    Auth,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusRecord {
    pub id: String,
    pub state: StatusState,
    pub kind: Option<StatusKind>,
    pub progress: Option<u8>,
    pub app: Option<String>,
    pub effective_app: Option<String>,
    pub title: Option<String>,
    pub message: Option<String>,
}

impl StatusRecord {
    pub fn new(id: impl Into<String>, state: StatusState) -> Self {
        Self {
            id: id.into(),
            state,
            kind: None,
            progress: None,
            app: None,
            effective_app: None,
            title: None,
            message: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatusReport {
    Clear(String),
    Record(StatusRecord),
}
