# OSC outer-parser independent review

Reviewer: Sargon. Native host: Windows x64.

Reviewed `tests/escape_osc.rs`, `scripts/escape-osc-oracle.mjs`, `scripts/escape-reference.mjs`, shared callback/removal/future helpers and the integration checklist in `docs/osc.md`. The loader preserves module-private declarations and hashes source bytes. The JS and Rust traces use the same executor turns, including queued cleanup before the current pending callback. The native test invokes the actual outer parser and compares each callback/state observation rather than a helper substitute. No weakened original assertions or ignored cases were found in this scoped integration work.

Executed the oracle, native integration tests and owned formatting checks. The oracle matched 6,726 traces, including 3,149 pending callbacks and 253 resets. All four native tests passed. Formatting passed. Command details and output are in this directory.

Focused Clippy failed on shared headless `with_queued_parse` dead code and a comparison chain in `input/move_to_cell.rs`. Those failures are retained and reported to owners. They are not claimed as an OSC quality pass. Task91 acceptance covers its native integration assertions and source correspondence, not the overall workspace quality criterion or terminal/browser integration.
