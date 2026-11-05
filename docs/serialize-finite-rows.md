# Finite numeric HTML row traversal

Task 427 repairs part of 346. The source HTML handler increments numeric rows without converting them to unsigned buffer indexes. Each iteration emits a row fragment, even when there is no line. A fractional final bound does not make the preceding integral row the final row, so that row uses its actual line length rather than the explicit end column.

Row lookup must preserve CircularList arithmetic. The source adds its current ring start index to the raw row before taking the remainder. Negative and large keys can alias real slots. Even the smallest positive fractional key can round to an integral slot during that addition. Treating every negative, fractional or large row as missing is wrong.

The shared Rust HTML serializer now has `HtmlNumericRange` and `serialize_html_numeric_rows`. The existing integral native API keeps its exact integer iterator. Both entry points share header, style, cell, warning and footer logic. Numeric buffer hooks preserve raw keys through the actual native Buffer and PublicBufferView ring lookups. The browser uses the existing `BufferApiView::get_line_number`, captures that line through warning reentry, and interpolates the original numeric coordinate in warnings. There is no JavaScript terminal fallback or new dependency.

## Negative controls and review repair

The first negative control is private commit `9b262a53`, main `926d8355` merged with accepted 421 ancestry. That merge preserves current main's public-cell documentation and includes the accepted 416 column repair. No previously accepted patch was replayed into canonical main. Main subsequently integrated 416 and 421 through task 429.

Three new shared source/Rust probes cover 24 finite bound/column combinations, header reset/resize with complete public-cell snapshots, and exact thrown header identity. Cases include negative zero, negative and fractional bounds, reversed ranges, 2^32 and the largest safe JavaScript integer. Baseline Chromium, Firefox and WebKit each match 58/62 cases. All three new probes fail, alongside the retained raw-numeric-boundaries mismatch. The first fixed run matches 61/62 per engine, but that implementation was not accepted.

The first independent reviewer found that the initial fix incorrectly filtered raw row keys before ring lookup. With a full, rotated two-row ring containing `def` and `ghi`, source row keys -1, Number.MIN_VALUE, 2^32 and Number.MAX_SAFE_INTEGER resolve to real lines. The reviewer reproduced the defect using the existing source build without compilation or browsers, and ran all five then-built native numeric tests. Those tests passed because their plain Vec adapter did not model a ring.

Two new browser probes exercise the real full, rotated ring and aliased-row warnings that reset the host. The initial implementation at `c13e7b10`, source hash `632d8eaf`, matches 61/64 in each browser and fails both new probes. The repaired implementation retains numeric coordinates through capture, cell access and diagnostics. Its source hash `fed4f597` matches 63/64 per engine. All five new directed probes match, including warnings and full public-cell snapshots. A sixth native regression verifies the same four aliases through actual Buffer and PublicBufferView implementations.

The remaining corpus mismatch is ANSI traversal of negative, fractional and large coordinates. The HTML portions now match. The runner still exits 1 rather than hiding that failure. One attempted alias negative control stopped before launching browsers because the generated latest-build pointer had been restored to an older report. The exact final-build report was selected and artifact hashes verified before the recorded 61/64 control. The failed attempt is recorded in `alias-first-attempt.txt`.

## Verification

The unchanged original SerializeAddon Playwright suites pass 120/120 source and 120/120 Rust on the repaired code, with 0 failed/skipped/unrun. Both before/after reference checks match all 776 manifest files. The 32 seeded restoration traces retain seed `0x05e21a11`, 12 operations per trace. Browser versions, full traces, source/build/runner hashes, suite logs and screenshots are retained under `results/serialize-finite-rows/fixed/`. Servers shut down with no cleanup errors.

All 15 native serialization test binaries pass 64 functions, with no ignored tests. Focused native and WASM library Clippy with `-D warnings` pass on Windows Rust 1.85.1. Exact `cargo fmt --all -- --check` still fails with Windows OS206, while all five package-level checks pass. Final native/Clippy/fmt logs use the `-final` suffix. Full workspace/all-feature/release and native Linux/macOS gates were not rerun for this scoped repair. Canonical integration still requires its normal gates.

The initial source and first browser evidence remain on the branch so the rejected implementation and repair can be audited. `baseline/` records the original 58/62 negative control, `fixed-first/` the initial 61/62 run, `alias-baseline/` the rejected implementation's 61/64 control, and `fixed/` the repaired 63/64 run. Independent resumed-review and cleanup receipts follow separately.

## Reproduction and limits

Use the locked wasm-bindgen CLI 0.2.100 and unchanged reference with its original generated browser bundle and installed Playwright dependencies. Set `CARGO_BUILD_JOBS=2`, `TEMP`, `TMP` and `TMPDIR` to an owned O: worktree tmp directory. Set `SOURCE_ROOT` to the supplied reference and `WASM_BINDGEN` to that CLI. Obtain the current team compiler/browser allocation before running these commands.

```sh
node scripts/build-demo.mjs --debug
node scripts/serialize-browser-check.mjs --suite
cargo test --locked -j2 -p xterm-core --test serialize_html_numeric_rows --test serialize_html
cargo clippy --locked -j2 -p xterm-core --lib --test serialize_html_numeric_rows -- -D warnings
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
cargo fmt --all -- --check
```

This entry point accepts finite row bounds within JavaScript's safe-integer domain. Values outside it can make row++ stop advancing; they return an explicit error rather than reproducing a source hang. This restriction is not a source-approved divergence. NaN row behavior remains the accepted 410 path. Infinite bounds, arbitrary objects/strings, late coordinate coercion, nonintegral columns, omitted-column resize reentry, raw scrollback, replaced public methods, palette-array mutation, all-browser GC and ANSI numeric traversal remain 346/322 work. Huge but progressing ranges still require source-compatible proportional work; no performance bound is claimed. Full equivalence remains NOT PROVEN.
