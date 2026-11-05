# Serialization traversal row identity

Task377 fixes one warning-reentry defect from task346. The unchanged source `BaseSerializeHandler.serialize` captures `const line = this._buffer.getLine(row)` once per row. Its cell loop continues to use that object even if `console.warn` resets the terminal and retargets the public buffer view. Ordinary wrapping and style lookahead still use the live view.

The shared Rust `BufferAccess` contract now distinguishes `capture_line` and `captured_cell` from ordinary live `line_len` and `cell`. Immutable native implementations delegate to their existing reads. The browser retains the actual `LineRef` for the row. It copies one cell under a short borrow, releases all borrows, and only then calls a host warning. Nested serialization constructs its own BrowserView and cannot replace the outer traversal capture. Each next row captures the then-current public view.

## Regression evidence

The ANSI and HTML probes write ABCDEFGHIJKL into12 columns, shrink to6 columns after the end boundary has been captured, then reset and expand to12 columns in the first missing-cell warning. Source emits6 warnings for columns6 through11 and retains only ABCDEF. Old Rust reacquired the new blank row after the first warning, producing only1 warning and, in HTML,5 extra spaces.

The negative control reverted only browser `serialize_addon/view.rs` to the actual integrated base `ff0bec8`; new probes and core defaults stayed in place. The real rebuilt Rust/WASM comparison matched50/53 cases in each engine and failed both new probes. Reapplying the browser fix and rebuilding matched52/53 in Chromium148, Firefox150 and WebKit26.4. The sole remaining mismatch is the previously retained raw-numeric-boundaries case in346. The runner still exits nonzero for that mismatch.

The unchanged original SerializeAddon suite passed120/120 expanded cases against source and120/120 against Rust, across all three engines. All14 native serialization test binaries passed58 test functions. Three new functions assert retained row contents and next-row recapture for ANSI/HTML plus capture/cell error propagation. Existing7 diagnostic/late-state tests still pass. Source integrity checks cover776 unchanged reference files.

Windows Rust1.85.1 results are retained in `results/serialize-line-identity/`: all-native.log, native.log, clippy-native.log, clippy-wasm.log, fmt.log, build.json, baseline-build.log, baseline/verification.json and baseline/cases.json. Final complete source/Rust browser traces and original-suite reports remain in `results/serialize-browser/`. Browser artifacts were verified against the actual Rust source hash before and after execution. The baseline cases file retains just the two new probes per engine, rather than duplicating seeded traces.

## Commands

Set `CARGO_BUILD_JOBS=2`, `TEMP` and `TMP` to the owned O: worktree's tmp directory, `SOURCE_ROOT` to the supplied unchanged reference and `WASM_BINDGEN` to the installed CLI0.2.100.

```sh
cargo test --locked -j2 -p xterm-core --test serialize_line_identity --test serialize_reentry --test serialize_diagnostics
node scripts/build-demo.mjs --debug
node scripts/serialize-browser-check.mjs --suite
cargo clippy --locked -j2 -p xterm-core --test serialize_line_identity -- -D warnings
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
cargo fmt --all -- --check
```

The all-native command expands every `crates/xterm-core/tests/serialize_*.rs` into an explicit `--test` argument. This check is scoped to serialization and is not a full workspace quality matrix. Browser evidence is from the focused branch before final integration. Raw coordinates, late marker coercion, arbitrary replaced host/buffer methods, reentrant palette mutation and actual GC remain346/322 work. Overall equivalence is NOT PROVEN.
