# ANSI invalid row-count failure ordering

Task421 repairs part of346. The source reads the range start marker twice, captures terminal columns, then reads the end marker twice. Only after those reads does StringSerializeHandler create `new Array(endRow - startRow)`. Rust previously converted the start into an unsigned native index immediately, rejecting NaN before observing either end getter.

The browser adapter now retains each raw marker value until both markers are read. For numeric primitive results, it calculates the source row count difference and checks the JavaScript Array length domain, zero through 2^32-1 inclusive. Nonfinite, negative, fractional and oversized differences call a minimal browser bridge that invokes the Array constructor with that invalid count. This preserves the engine's actual RangeError text. The bridge never allocates a valid large array, parses terminal input or substitutes a JavaScript engine. Terminal serialization remains in shared Rust. Valid integral ranges continue through the unchanged native serializer.

## Regression evidence

`ansi-invalid-row-count-phases` compares eleven bound pairs. These include NaN at either or both endpoints, both signs of infinity, reversed integral ranges, fractional differences and a count of2^32. It compares exact exception name/message and range/start/end/late-option getter logs. `ansi-invalid-start-end-getter-reentry` checks that the first or second end getter can throw the exact sentinel object despite the invalid start. Its no-throw variant resets/resizes the host before the Array failure. Full public-cell snapshots and dimensions verify those side effects.

The negative control is current maine8011592 merged with accepted task416 ancestry, before this production change. Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 each match56/59 corpus cases. Both new cases fail, alongside the retained raw-numeric-boundaries mismatch. Fixed each matches58/59. The sole remaining corpus mismatch stays visible and the runner exits1. Existing raw-numeric-boundaries now matches ANSI NaN and both infinity errors, but its negative/fractional/large-coordinate traversal mismatches remain.

The unchanged original SerializeAddon Playwright suites pass120/120 source and120/120 Rust, with0 failed/skipped/unrun. All776 reference-manifest files match before and after. The seeded32-case full public-cell restoration corpus retains seed `0x05e21a11` and12 operations per trace. No source files, assertions, source timeouts or retries changed. Both servers shut down without cleanup errors.

`results/serialize-array-count/baseline/` retains baseline traces for both added cases and the complete baseline report. `results/serialize-browser/` contains full fixed traces, unchanged suite reports, screenshots and input/artifact hashes. `results/demos/build/` retains baseline and fixed WASM build identities. Focused quality receipts are under `results/serialize-array-count/`.

Windows Rust1.85.1 WASM-library Clippy with `-D warnings` passes. Exact workspace `cargo fmt --all -- --check` fails with Windows OS206; all five package-level format checks pass. All14 native serialization test binaries pass58 tests, with no ignored tests, as recorded in `native-all.log`. Full workspace/all-feature/release and native Linux/macOS gates were not rerun for this browser-only change. Canonical integration still requires its gates.

## Reproduction and limits

Set `CARGO_BUILD_JOBS=2`, `TEMP`, `TMP` and `TMPDIR` to an owned O: worktree tmp directory, `SOURCE_ROOT` to the unchanged supplied reference and `WASM_BINDGEN` to CLI0.2.100.

```sh
node scripts/build-demo.mjs --debug
node scripts/serialize-browser-check.mjs --suite
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
cargo fmt --all -- --check
for p in xterm-core xterm-addons xterm-web xterm-wasm-support xterm-font-addons; do
  cargo fmt -p "$p" -- --check || exit
done
```

This does not close arbitrary object/string coordinate coercion, valid row-count ranges with unrepresentable native endpoints, raw scrollback values, replaced public methods, palette-array mutation, all-browser GC or the full search/serialization family. Equal infinite endpoints throw before ANSI traversal, but can make HTML source traversal non-progressing. Those hanging HTML probes remain unrun, not approved divergences. Tasks346/322 remain open. Full equivalence is NOT PROVEN.
