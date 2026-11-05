# Finite numeric HTML row traversal

Task427 repairs part of346. The source HTML handler increments numeric rows without converting them to unsigned buffer indexes. Negative and fractional rows usually have no line, but each iteration still emits `<div><span></span></div>`. Later integral rows remain visible. A fractional final bound does not make the preceding integral row the final row, so that row uses its actual line length rather than the explicit end column.

The shared Rust HTML serializer now has `HtmlNumericRange` and `serialize_html_numeric_rows`. The existing integral native API keeps its exact integer iterator. Both entry points share header, style, cell, warning and footer logic. The browser adapter uses numeric traversal for finite primitive row endpoints and keeps captured endpoint values through header callbacks. Only valid integral row keys become native buffer indexes. There is no JavaScript terminal fallback or new dependency.

## Evidence

The negative control is private commit9b262a53, main926d8355 merged with accepted421 ancestry. That merge preserves current main's public-cell documentation and includes the accepted416 column repair. No previously accepted patch was replayed into canonical main.

Three new shared source/Rust probes cover24 finite bound/column combinations, header reset/resize with complete public-cell snapshots, and exact thrown header identity. Cases include negative zero, negative and fractional bounds, reversed ranges,2^32 and the largest safe JavaScript integer. Baseline Chromium, Firefox and WebKit each match58/62 corpus cases. All three new probes fail, alongside the retained raw-numeric-boundaries mismatch. The first fixed run matches61/62 per engine. All new probes match. The remaining mismatch is ANSI traversal of negative, fractional and large coordinates; the HTML portions now match. The corpus still exits1 rather than hiding that failure.

The unchanged original SerializeAddon Playwright suites pass120/120 source and120/120 Rust, with0 failed/skipped/unrun. Both before/after reference checks match all776 manifest files. The32 seeded restoration traces retain seed `0x05e21a11`,12 operations per trace. Browser versions, full traces, source/build/runner hashes, suite logs and screenshots are retained under `results/serialize-finite-rows/`. Servers shut down with no cleanup errors.

Five native regression functions test missing-row output, row stepping and endpoint equality, integral API equivalence, large indexes, and explicit rejection of nonfinite/nonprogressing coordinates. Focused native and WASM library Clippy with `-D warnings` passed on Windows Rust1.85.1. Final quality/review receipts are recorded separately after source freeze.

## Reproduction and limits

Use the locked wasm-bindgen CLI0.2.100 and unchanged reference with its original generated browser bundle and installed Playwright dependencies. Set `CARGO_BUILD_JOBS=2`, `TEMP`, `TMP` and `TMPDIR` to an owned O: worktree tmp directory. Set `SOURCE_ROOT` to the supplied reference and `WASM_BINDGEN` to that CLI. Obtain the current team compiler/browser allocation before running these commands.

```sh
node scripts/build-demo.mjs --debug
node scripts/serialize-browser-check.mjs --suite
cargo test --locked -j2 -p xterm-core --test serialize_html_numeric_rows --test serialize_html
cargo clippy --locked -j2 -p xterm-core --lib --test serialize_html_numeric_rows -- -D warnings
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
cargo fmt --all -- --check
```

This entry point accepts finite row bounds within JavaScript's safe-integer domain. Values outside it can make row++ stop advancing; they return an explicit error rather than reproducing a source hang. This restriction is not a source-approved divergence. NaN row behavior remains the accepted410 path. Infinite bounds, arbitrary objects/strings, late coordinate coercion, nonintegral columns, omitted-column resize reentry, raw scrollback, replaced public methods, palette-array mutation, all-browser GC and ANSI numeric traversal remain346/322 work. Huge but progressing ranges still require source-compatible proportional work; no performance bound is claimed. Full equivalence remains NOT PROVEN.
