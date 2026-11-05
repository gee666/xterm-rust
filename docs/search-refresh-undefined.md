# Delayed search refresh with no cached term

The full user intent remains `docs/intent.md` and `PORT_REQUIREMENTS.md`. Task422 continues the same 16-agent swarm and fixes one native search defect, not full search equivalence. Overall equivalence is NOT PROVEN.

## Behavior

Source captures the current cached term when its queued write/resize refresh fires, clears the cache, then copies current options before calling findPrevious. If clearDecorations or an intervening undefined-term search has already cleared the cache, that captured term is undefined. The native addon instead created a UTF16 empty string. A later decorated native empty-string search could then compare matching flags, including throwing callbacks, even though source bypasses comparisons with an undefined cache.

The private native term now distinguishes `Text` from `Undefined`. The existing completion path recognizes undefined and leaves the cache absent. Empty text stays a real cached value. Two native constructor calls change mechanically; matching and accepted412 failure ordering do not change. No addon/cache/buffer borrow spans option copying or reentry. An option-copy callback may cache a nested empty search, but the outer refresh still completes with its previously captured undefined term.

No dependency, unsafe code, lint suppression or JS terminal-engine delegation was added. Production files remain below350 lines. Reference source, upstream assertions, AGENTS.md and credentials are unchanged. All new work and temporary builds use owned `O:/xterm-rust-swarm-16/tmp/Q422`.

## Verification

Run with Node24.15.0, Rust/Cargo1.85.1 and wasm-bindgen0.2.100. Set TEMP, TMP and TMPDIR to owned O: scratch, CARGO_BUILD_JOBS=2, SOURCE_DEPENDENCIES to the unchanged source dependency checkout, PLAYWRIGHT_MODULE to its Playwright index.mjs and WASM_BINDGEN to the matching installed CLI. Existing runners bound their processes and close browsers/servers in finally blocks. No source junction or dependency installation was needed.

```sh
SEARCH_CASES=search-refresh-undefined node scripts/search-call-args-source.mjs
cargo test --locked -j2 -p xterm-web --test search_addon
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon -- -D warnings
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
cargo fmt --all -- --check
```

`fixtures/search-refresh-undefined.js` runs the same38 directed cases per engine against unchanged source and actual Rust bindings. It covers write/resize scheduling, both follow-up directions, clear/retain behavior, intervening undefined/empty/null/false/zero searches, selection clearing, event and live option-copy order, nested search from copying, skipped throwing flag getters and preserved thrown identity for a truly cached empty string. Three native virtual-clock test functions cover the actual regression, explicit-empty control, both scheduling events/directions and copy reentry. No sleeps or new random cases are used.

| Windows check | Executed | Passed | Failed | Ignored/skipped/unrun |
| --- | ---: | ---: | ---: | ---: |
| Shared unchanged-source cases, three engines | 114 | 114 | 0 | 0 |
| Fixed native test functions | 40 | 40 | 0 | 0 |
| Fixed Rust WASM functions, three engines | 36 | 36 | 0 | 0 |
| New directed cases inside those WASM functions | 114 | 114 | 0 | 0 |
| Exact old native producers, three new functions | 3 | 1 | 2 | 37 filtered |
| Exact old WASM producers, Chromium control | 12 | 12 | 0 | 0 |

The114 directed WASM cases are inside three of the36 functions, not114 additional Rust functions. Browsers are Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. All776 reference-manifest entries match before and after source execution. Final native/WASM and scoped warning-denied Clippy receipts are on source fdb71129 atop main013bf319, including accepted406/407/408 and412. Initial native evidence predates that font union and is retained separately.

The negative controls restore all three old search producers from3fe9a262, keeping the new assertions. Native catches the unwanted comparison and the reentrant-copy error. The browser control unexpectedly passes because old fallback text has native Utf16Term identity, while actual browser inputs use JsTerm identity. They cannot compare equal, so the browser adapter already bypassed this comparison. The temporary controller's expectation therefore failed. Its raw successful browser receipt and `expectedRegressionObserved:false` remain, not relabeled as a failing browser regression detector. Producer bytes were restored in finally and all three hashes verified. A subsequent fixed native40 and WASM36 matrix passes.

`results/search-refresh-undefined/summary.json` pins source, fixtures and raw receipts. Text logs normalize CRLF to LF and retain one final newline; embedded raw browser output in JSON is unchanged. Compiler, WASM and binding paths in runner receipts are temporary verification artifacts, not delivered distributions. Full `cargo fmt --all -- --check` still fails with Windows OS206; affected Rust-file formatting passes. This is not a full quality waiver. The unchanged original81 browser suite was not rerun for422. Task412 retains its earlier81-pass evidence, not evidence for this later source. Native Linux/macOS runtime, full raw matching, randomized raw-option traces, source-suite mapping and complete347/322 remain open.

## Review and integration

Task422 accepted source/evidence2d6da430 against base013bf319. The isolated reviewer independently ran native40/40, scoped warning-denied Clippy, affected-file rustfmt and diff checking. It verified all37 then-pinned files, both retained browser artifacts and all776 reference-manifest entries. It inspected source/WASM and control receipts without launching browsers. No files changed and no findings were raised. `review.txt` preserves the outcome; the final summary adds that receipt as its38th pin.

Canonical integration is delegated to task309, not claimed complete. No main reference has been changed by this lane. Accepted source and concise raw evidence are preserved in Git before owned temporary cleanup.

The cleanup preflight scans10746 files/2444 directories, finds zero links, redirects or owned tool processes, verifies all38 final pins and individually whitelists212 generated untracked browser paths under two known roots. `cleanup-preflight.json` retains the checks. Root deletion is reported separately through task309 and post557 after the command completes; do not infer it from this preflight.
