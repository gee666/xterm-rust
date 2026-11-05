# Empty HTML traversal for reversed numeric rows

Task 446 repairs a scoped part of serialization task 346. Full search/serialization family 322 remains open. Full equivalence is NOT PROVEN.

## Source contract and repair

`BaseSerializeHandler.serialize` reads the bounds, calls the HTML header handler and then tests `row <= endRow`. Primitive numeric bounds with `start > end` never enter the row loop. The start column is captured but never coerced. This remains true for negative, fractional, very large and infinite reversed bounds. Unlike the ANSI handler, the HTML handler does not allocate a row-count array.

The browser adapter previously converted the column before determining that traversal was empty. A Symbol or throwing column object could fail before header getters. Even with a normal column, reversed infinite or out-of-safe-domain row bounds produced a Rust coordinate error. The adapter now uses its existing reversed native range for these empty numeric cases, alongside the previously accepted NaN-row path. It retains header/footer generation and skips cell reads and unused-column coercion.

The change is confined to `crates/xterm-web/src/serialize_addon/html.rs`. The shared Rust serializer and native integer API do not change. There is no new dependency, unsafe operation, source reference edit or JavaScript terminal fallback. Non-numeric bounds still follow the existing path; this repair does not claim late object-coercion parity. Equal infinite bounds and other non-progressing source loops remain unresolved, not silently accepted.

## Regression evidence

Two shared source/Rust probes cover eight reversed bound pairs and normal, Symbol and throwing-object columns. They compare exact HTML, getters and their order, header reentry that resets/resizes the terminal and mutates the original range values, complete public-cell snapshots, and exact thrown-header identity. The fixture remains cohesive because Playwright serializes the whole function into each browser realm, as documented in the file.

The private branch starts at main `3aa04692`. It includes only the accepted 437 probe/source prerequisites before the new work. Those prerequisites must not be replayed into canonical main. Task 437 remains separately queued for integration.

- Negative control commit `da02b9e8`, actual source SHA-256 `25c812cd72b3a747b4d784567d7350348b915535b549b465a93db6a56ce885f9`.
- Repair commit `630eb7f9`, actual source SHA-256 `55b0246711036f9bcd428c3af0f7575e9851351d8bde9766269a8487239590f3`.
- Baseline build report `results/demos/build/2026-11-05T18-59-29-800Z/report.json`.
- Fixed build report `results/demos/build/2026-11-05T19-01-10-472Z/report.json`.

The negative control matches 65/68 cases in each engine. Both new probes fail, alongside the retained ANSI raw-coordinate case. Fixed matches 67/68 in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Both new probes match exactly. The sole remaining mismatch is `raw-numeric-boundaries`, in its ANSI results. The runner deliberately exits 1 for that failure. Neither run is reported as a fully passing corpus.

The unchanged original SerializeAddon suite passes 120/120 source cases and 120/120 Rust cases across the three browsers, with zero failed, skipped or unrun cases. Both runs verify all 776 reference manifest files before and after execution without mismatches. The 32 seeded restoration traces retain seed `0x05e21a11`, with 12 operations each. Raw traces, screenshots, source/build/runner hashes and original-suite reports are under `results/serialize-reversed-rows/{baseline,fixed}/`; `summary.json` records exact totals. Neither run reports server cleanup errors.

The negative runner uses the already-built baseline package, source bytes and exact latest-build pointer. It verifies their source/package identities, swaps only owned temporary files, and restores fixed bytes in `finally`. It verifies the restored fixed source/package identities before finishing. The helper is retained as `results/serialize-reversed-rows/negative-swap.mjs.txt`. The baseline report's HEAD revision remains the fixed commit during this byte-level negative control, while its actual source hash and recorded build revision correctly identify the baseline. No source or package was rebuilt for that run. A reporting-only inline summary command had a syntax error after the final browser runner finished; only the copy/summary step was corrected, without rerunning browsers.

## Quality checks and reproduction

Windows Rust 1.85.1, Cargo 1.85.1, Node 24.15.0 and wasm-bindgen 0.2.100 were used. All 15 native serialization binaries pass 64 tests, with none failed or ignored. Warning-denied WASM library Clippy and `cargo fmt -p xterm-web -- --check` pass. Exact `cargo fmt --all -- --check` fails with Windows OS206; its complete output is retained. No full workspace/all-feature/release or native Linux/macOS pass is claimed here.

Set `SOURCE_ROOT` to the unchanged reference with its installed Playwright dependencies and generated original demo bundle. Set `WASM_BINDGEN` to the locked CLI. Keep `TEMP`, `TMP`, `TMPDIR` and private targets inside the owned O: worktree. Obtain current team allocation before running compilers or browsers.

```sh
CARGO_BUILD_JOBS=2 node scripts/build-demo.mjs --debug
node scripts/serialize-browser-check.mjs --suite
cargo test --locked -j2 -p xterm-core --test serialize_html --test serialize_html_numeric_rows
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
cargo fmt -p xterm-web -- --check
cargo fmt --all -- --check
```

Compiler allocation 2767 was released in post 409 comment 2792. Browser allocation 2806 was released in comment 2818. Review is source/light only unless separately authorized. Canonical integration gates and the complete 346/322 raw traversal, palette, public-host replacement and all-browser lifetime obligations remain open.
