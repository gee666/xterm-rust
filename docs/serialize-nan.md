# HTML serialization with NaN row bounds

Task410 repairs one raw-number case from346. In the unchanged source, numeric `NaN` at either row boundary makes the row-loop condition false. The header and footer still run, but no column is coerced or cell visited. Rejecting the coordinate before header generation loses both output and user getter effects.

The browser binding reads the range properties in source order, including both reads of each row getter and the intervening terminal width read. When both captured row values are numbers and at least one is `NaN`, it supplies a reversed integral range to the shared Rust HTML serializer. That range represents zero row iterations while retaining normal header/footer generation. It does not return a fabricated empty string or skip header exceptions. Other inputs still use the existing integral representation. Native public APIs are unchanged.

## Evidence

The new `html-nan-row-bounds` probe compares five numeric pairs: `NaN/0`, `0/NaN`, `NaN/NaN`, `NaN/-Infinity` and `Infinity/NaN`. A Symbol start column must remain unused. Exact HTML, getter logs and warning lists match source. A separate probe checks that a header getter still executes and propagates the same thrown sentinel object.

The negative control built unmodified main51112351 serializer code with the new probes. Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 each matched52/55 cases, failing both new probes and the existing raw-coordinate corpus case. The fixed build matched54/55 cases in each engine. Both new probes pass. `raw-numeric-boundaries` remains mismatched and the runner still exits1; it is not filtered out. The unchanged original SerializeAddon suite passed120/120 cases against source and120/120 against Rust across the three browsers, with no skipped or unrun suite cases.

`results/serialize-nan/baseline/` retains the negative-control report and the two new source/Rust traces per browser. `results/serialize-browser/` retains the full fixed traces, original-suite reports, screenshots, source/build/runner hashes and clean server shutdown outcomes. Build logs and scoped lint/format logs are under `results/serialize-nan/`. The build reports linked by the browser receipts remain under `results/demos/build/`. Reference integrity verified776 unchanged source files before and after both runs.

Windows Rust1.85.1 WASM-library warning-denied Clippy passes. Exact `cargo fmt --all -- --check` fails with Windows OS206 at this worktree path. The equivalent five per-package format checks all pass; do not describe the exact command as passed. Native tests and Linux/macOS runtimes were not rerun for this browser-only change. The canonical integration helper still must run its changed-package and dependent gates.

## Reproduction and remaining work

Set `CARGO_BUILD_JOBS=2`, `TEMP` and `TMP` to the owned O: worktree tmp directory, `SOURCE_ROOT` to the unchanged reference and `WASM_BINDGEN` to CLI0.2.100.

```sh
node scripts/build-demo.mjs --debug
node scripts/serialize-browser-check.mjs --suite
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
cargo fmt --all -- --check
# If Windows OS206 prevents the single invocation, record that failure and run:
for p in xterm-core xterm-addons xterm-web xterm-wasm-support xterm-font-addons; do
  cargo fmt -p "$p" -- --check || exit
done
```

This repair covers numeric primitive row bounds only. It does not claim object coercion, NaN columns, ANSI NaN, signed/fractional ring indexing, method replacement, palette-array identity or complete lifetime/restoration equivalence. Task346 and family322 remain open. Full equivalence is NOT PROVEN.
