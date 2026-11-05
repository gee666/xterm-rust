# Omitted HTML start columns during resize callbacks

Task 437 repairs one remaining part of serialization task 346. Full search/serialization family 322 remains open. Full equivalence is NOT PROVEN.

The original HTML range passes `range.startCol` unchanged into `BaseSerializeHandler.serialize`. When it is omitted or explicitly undefined, `col < endLineColumn` is false, regardless of the current line length. The first row still emits an empty fragment. Later rows start at column zero.

The browser adapter previously substituted the captured terminal column count for undefined. That usually skips the first row by accident. A callback can grow the line after that count is captured, so a nonfinal first row then visits its new trailing cells. For a 12-column line grown to 18, Rust emitted six extra spaces where source emitted none. An `endLine` getter and a `fontFamily` header getter can both cause this defect.

The adapter now uses the same empty-loop sentinel already used for numeric NaN and positive Infinity columns. It changes neither native integer ranges nor the shared Rust serializer. It preserves the originally captured end column, later rows, header execution and thrown-value identity. No dependency or JavaScript terminal fallback was added.

## Directed regressions

Two shared source/Rust browser probes grow a nonfinal first row from 12 to 18 columns, once in the `endLine` getter and once in the font-family header getter. Each captures exact HTML, getter order, explicit undefined and zero controls, thrown header identity and complete public-cell state. The shared fixture stays in one function because Playwright serializes it into each browser realm.

The negative control is `532551a2`, built source hash `ce0d94da9022620413978edc7748c5fa5127bf20668c0d7424ca08b24354ad5c`. The repair is `e7a7be6a`, built source hash `266acb0cbcc240f0ea9e1a5b8cd42f6d602d1258c1523b293ba43a8f7bc16c72`. Both build reports record Windows Rust 1.85.1, Cargo 1.85.1 and wasm-bindgen 0.2.100. The private branch merges accepted 427 ancestry without replaying the already-integrated 416/421 patches. Only the new probe, repair and evidence commits need subsequent canonical pickup after 427.

The negative control matches 63/66 cases in each of Chromium, Firefox and WebKit. Both new omitted-column probes fail. The repaired run matches 65/66 cases in every engine; both probes now match, including getter order, later rows, thrown header identity and full cell state. The remaining mismatch is the previously retained ANSI raw-numeric traversal gap. The runner still exits 1 and reports it as failed.

The unchanged original SerializeAddon suite passes 120/120 source cases and 120/120 Rust cases, with zero failed, skipped or unrun cases. Before/after reference integrity checks match all 776 manifest files. The 32 seeded restoration cases retain seed `0x05e21a11`, with 12 operations per trace. Raw traces, browser versions, screenshots, suite reports and exact runner/package/source identities are under `results/serialize-omitted-column/baseline/` and `fixed/`. Both runs shut down their servers without cleanup errors.

## Completed checks

- All 15 native serialization test binaries pass 64 functions, with none ignored.
- Warning-denied WASM library Clippy passes for xterm-web and its dependencies.
- `cargo fmt -p xterm-web -- --check` passes.
- Exact `cargo fmt --all -- --check` fails with Windows OS206. The raw failure remains in `results/serialize-omitted-column/fmt-workspace.log`.
- Baseline and fixed debug WASM builds pass. The browser package hashes and full build input identities are retained in their reports under `results/demos/build/`.

Commands use private O: targets, `CARGO_BUILD_JOBS=2` and the current team allocation. Compiler allocation 2636 was released in post 409 comment 2657. Browser allocation 2671 covered the sequential negative-control and final matrices and has been released. No new full workspace, all-feature, release or native Linux/macOS pass is claimed.

For runtime reproduction, use the documented `SOURCE_ROOT`, wasm-bindgen and O: temporary environment from `docs/serialize-finite-rows.md`, then run `node scripts/build-demo.mjs --debug` and `node scripts/serialize-browser-check.mjs --suite` after obtaining a browser allocation. The known ANSI raw-coordinate mismatch remains an expected nonzero runner result, not an ignored or passing case.
