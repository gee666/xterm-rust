// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
use super::*;
enum LookupSource {
    Cache(DecorationLineCache),
    Service(DecorationService),
}
/// Defers lookup until first next and retains its live bucket through callbacks.
/// Once exhausted it remains done, like a JavaScript generator.
pub struct DecorationCellIterator {
    source: LookupSource,
    line: i64,
    initialized: bool,
    finished: bool,
    bucket: Option<Bucket>,
    index: usize,
    x: f64,
    layer: Option<DecorationLayer>,
}
impl DecorationCellIterator {
    pub(super) fn new(
        cache: DecorationLineCache,
        x: f64,
        line: i64,
        layer: Option<DecorationLayer>,
    ) -> Self {
        Self {
            source: LookupSource::Cache(cache),
            line,
            initialized: false,
            finished: false,
            bucket: None,
            index: 0,
            x,
            layer,
        }
    }
    pub(crate) fn retain_service(mut self, service: DecorationService) -> Self {
        self.source = LookupSource::Service(service);
        self
    }
}
impl Iterator for DecorationCellIterator {
    type Item = Decoration;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        if !self.initialized {
            self.bucket = match &self.source {
                LookupSource::Cache(cache) => cache.bucket(self.line),
                LookupSource::Service(service) => service.line_cache().bucket(self.line),
            };
            self.initialized = true;
        }
        let Some(bucket) = self.bucket.as_ref() else {
            self.finished = true;
            return None;
        };
        loop {
            let decoration = bucket.borrow().get(self.index).cloned();
            let Some(decoration) = decoration else {
                self.finished = true;
                return None;
            };
            self.index += 1;
            if decoration.matches(self.x, self.layer) {
                return Some(decoration);
            }
        }
    }
}
impl std::iter::FusedIterator for DecorationCellIterator {}
