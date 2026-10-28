// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of addon-program-status/ProgramStatus.ts.

use super::{StatusRecord, StatusState};
use std::rc::Rc;

const RECORD_LIMIT: usize = 256;

/// At most 256 records. A vector preserves the Map update order directly.
/// Shared slices are immutable cached snapshots, matching frozen JS arrays.
#[derive(Default)]
pub struct ProgramStatusStore {
    records: Vec<StatusRecord>,
    snapshot: Option<Rc<[StatusRecord]>>,
}

impl ProgramStatusStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn records(&mut self) -> Rc<[StatusRecord]> {
        if let Some(snapshot) = &self.snapshot {
            return Rc::clone(snapshot);
        }
        let records: Vec<_> = self
            .records
            .iter()
            .map(|record| {
                let mut record = record.clone();
                if let Some(app) = self.effective_app(&record.id) {
                    record.effective_app = Some(app.to_owned());
                    record.refresh_identity();
                }
                record
            })
            .collect();
        let snapshot: Rc<[StatusRecord]> = records.into();
        self.snapshot = Some(Rc::clone(&snapshot));
        snapshot
    }

    pub fn replace(&mut self, record: StatusRecord) {
        if let Some(index) = self.records.iter().position(|r| r.id == record.id) {
            self.records.remove(index);
        } else if self.records.len() >= RECORD_LIMIT {
            self.records.remove(0);
        }
        self.records.push(record);
        self.snapshot = None;
    }

    /// Empty id clears all records, not just the root record.
    pub fn clear(&mut self, id: &str) -> bool {
        let prefix = format!("{id}/");
        self.remove(|r| id.is_empty() || r.id == id || r.id.starts_with(&prefix))
    }

    pub fn remove_active_records(&mut self) -> bool {
        self.remove(|r| matches!(r.state, StatusState::Working | StatusState::Blocked))
    }

    fn remove(&mut self, predicate: impl Fn(&StatusRecord) -> bool) -> bool {
        let before = self.records.len();
        self.records.retain(|r| !predicate(r));
        let changed = self.records.len() != before;
        if changed {
            self.snapshot = None;
        }
        changed
    }

    fn effective_app<'a>(&'a self, mut id: &str) -> Option<&'a str> {
        loop {
            if let Some(app) = self
                .records
                .iter()
                .find(|r| r.id == id)
                .and_then(|r| r.app.as_deref())
            {
                return Some(app);
            }
            if id.is_empty() {
                return None;
            }
            id = id.rsplit_once('/').map_or("", |(parent, _)| parent);
        }
    }
}
