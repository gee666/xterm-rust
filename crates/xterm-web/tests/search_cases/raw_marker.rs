//! Search overview deduplication must use raw marker Set keys, not integer casts.
use super::Harness;
use std::{cell::Cell, rc::Rc};
use xterm_addons::search::{
    DecorationManager, SearchColor, SearchDecorationOptions, SearchDecorationOptionsHandle,
    SearchDecorationSource,
};
use xterm_core::{event::EmitError, headless::Terminal, search::SearchResult};

struct RetypeMarker {
    terminal: Rc<Terminal>,
    calls: Cell<usize>,
    nan: bool,
}
impl SearchDecorationSource for RetypeMarker {
    fn color(&self, kind: SearchColor) -> Result<Option<String>, EmitError> {
        if kind == SearchColor::MatchBackground {
            let call = self.calls.get();
            self.calls.set(call + 1);
            // A live option getter can mutate the marker created just before it.
            // The manager must retain the resulting primitive without coercion.
            let marker = self.terminal.markers().unwrap().last().unwrap().clone();
            if self.nan {
                marker.set_line(f64::NAN);
            } else if call == 0 {
                marker.set_line("0");
            }
        }
        SearchDecorationOptions::default().color(kind)
    }
}

#[test]
fn overview_dedup_distinguishes_string_numbers_and_coalesces_nan() {
    for (nan, expected) in [(false, 2), (true, 1)] {
        let harness = Harness::new(20, 3, None);
        let manager = DecorationManager::new(harness.host.clone());
        let source: SearchDecorationOptionsHandle = Rc::new(RetypeMarker {
            terminal: harness.host.terminal.native.clone(),
            calls: Cell::new(0),
            nan,
        });
        manager
            .create_highlights(
                &[
                    SearchResult {
                        row: 0,
                        col: 0,
                        size: 1,
                        term: vec![0x61],
                    },
                    SearchResult {
                        row: 0,
                        col: 2,
                        size: 1,
                        term: vec![0x61],
                    },
                ],
                &source,
            )
            .unwrap();
        let decorations = harness.decorations();
        assert_eq!(decorations.len(), 2);
        assert_eq!(
            decorations
                .iter()
                .filter(|decoration| {
                    decoration
                        .options()
                        .borrow()
                        .overview_ruler_options
                        .is_some()
                })
                .count(),
            expected
        );
        if !nan {
            assert!(decorations.iter().any(|decoration| decoration
                .marker()
                .line_value()
                .as_string()
                == Some(&[0x30])));
            assert!(decorations
                .iter()
                .any(|decoration| decoration.marker().line_value().strict_eq_number(0.0)));
        }
    }
}
