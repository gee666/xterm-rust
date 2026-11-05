# HTML serialization with nonfinite start columns

Task416 repairs a narrow part of346. For explicit HTML ranges with integral nonnegative row bounds, a numeric primitive `NaN` or positive `Infinity` start column makes the first row's `col < end` condition false. Source still emits that empty row, runs header getters and visits later rows from column zero. It does not reject the coordinate before the header.

The browser binding represents these two no-cell values with `usize::MAX` in the existing shared integer traversal. This skips only the first row's cell loop and leaves normal header, row and later-cell behavior intact. Native public types and algorithms are unchanged. Task410's numeric NaN row repair is retained through branch ancestry.

## Evidence

The new `html-nonfinite-start-column` probe compares both values across one, two and three rows. The colored later rows expose previous-cell/style mistakes as well as missing content. It compares exact HTML, range/header getter order and warnings. `html-nonfinite-column-header-throw` verifies exact thrown sentinel identity for both column values.

The negative control uses task410 plus current main85e0fd43, before the column repair. Each browser matches54/57 cases, failing both new probes plus the retained `raw-numeric-boundaries` case. Fixed Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 each match56/57. Only the retained raw-coordinate mismatch remains. The runner still exits1 and does not filter it out. The seeded32-case restoration corpus retains seed `0x05e21a11` and12 operations per trace.

The first unchanged-suite attempt passed120/120 source cases, while Rust passed80, failed1 Firefox `beforeAll`10-second setup timeout and left39 unrun. Its full Rust report and aggregate verification are retained in `results/serialize-column/first-suite-attempt/`. A single bounded repeat with the same source assertions, timeouts, retries0 and workers2 passed120/120 source and120/120 Rust cases, with0 skipped and0 unrun. This records a setup-timeout instability, not a clean first-attempt pass. Both corpus attempts matched56/57 in each browser.

`results/serialize-column/baseline/` retains the negative-control report and both new source/Rust traces. `results/serialize-browser/` contains the complete fixed traces, final original-suite receipts, browser screenshots and hashes. `results/serialize-column/` contains baseline/fixed build logs and quality receipts. Linked production build reports are in `results/demos/build/`. All776 reference-manifest files matched before and after each run; server shutdowns reported no cleanup errors.

Windows Rust1.85.1 WASM-library Clippy with `-D warnings` passes. Exact workspace `cargo fmt --all -- --check` fails with Windows OS206. All five package-level format checks pass. The change touches only the browser range adapter; native tests, the complete all-feature quality matrix and Linux/macOS runtimes were not rerun for this scoped repair. Canonical integration gates remain required.

## Reproduction and limits

Set `CARGO_BUILD_JOBS=2`, `TEMP` and `TMP` to an owned O: worktree tmp directory, `SOURCE_ROOT` to the unchanged reference and `WASM_BINDGEN` to CLI0.2.100.

```sh
node scripts/build-demo.mjs --debug
node scripts/serialize-browser-check.mjs --suite
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
cargo fmt --all -- --check
for p in xterm-core xterm-addons xterm-web xterm-wasm-support xterm-font-addons; do
  cargo fmt -p "$p" -- --check || exit
done
```

This does not support arbitrary object/string column coercion, negative/fractional coordinates, NaN ANSI ranges, host replacement, palette-array identity or full lifetime/restoration equivalence. Negative Infinity columns can make source traversal non-progressing and are not added as hanging probes. Task346 and family322 remain open. Full equivalence is NOT PROVEN.
