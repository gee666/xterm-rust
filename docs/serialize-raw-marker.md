# ANSI raw marker allocation coercion

Task 549 repairs the allocation phase of browser ANSI `SerializeAddon.serialize`, within the still-open tasks 346 and 322. Full equivalence is NOT PROVEN.

The original `_serializeBufferByRange` reads `range.start` twice and its marker's `line`, captures columns, then reads `range.end` twice and its marker's `line`. `BaseSerializeHandler.serialize` evaluates `endRow - startRow` before calling `StringSerializeHandler._beforeSerialize`. That subtraction evaluates the end coordinate first, then the start coordinate, both with the number hint. Exceptions from either operand stop serialization before later option getters. A numeric invalid difference reaches the browser's Array constructor and throws its actual RangeError.

Current main 0442 validates only primitive-number differences, then converts the start index before the end. For raw object marker lines, this skips the source subtraction and changes conversion order, error precedence, reentrant state changes and diagnostics. The scoped repair must perform that subtraction before native index conversion. A small caught JavaScript operator is host-language glue, not a serializer or a JavaScript terminal engine. It stays beside this adapter's existing invalid-array-length bridge rather than making serialization depend on scrolling internals.

The shared `scripts/serialize-browser-cases.mjs` adds three cases with 35 subcases. They cover boxed start/end/both invalid differences, nonfinite/fractional/negative/oversized differences, Symbol.toPrimitive getter/call exceptions and exact thrown identity, Symbol/nonprimitive/mixed-BigInt results, reset/resize, nested serialization and end coercion mutating the start operand. Existing cases and source assertions remain intact. The complete runner expects 93 cases, including the unchanged 32 seeded traces with seed `0x05e21a11`.

## Scope limits

Valid arbitrary-object row traversal is still incomplete. Source comparisons, raw ring lookup, strict endpoint equality and row increment can each coerce the original coordinate again. This task does not replace those original values with a supposedly equivalent cached number. A successful subtraction does not prove valid raw coordinates are supported. Pure-BigInt difference traversal and arbitrary replaced host/public-buffer methods remain open. Equal infinite source bounds can hang and are not run or claimed compatible. No new dependency, unsafe code or source assertion changes are needed.

## Verification

Windows Rust 1.85.1, Cargo 1.85.1 and Node 24.15.0 were used. The exact old production control matches 90/93 cases in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. All three new groups fail in each engine; the prior 90 cases remain matching. The control uses accepted 531's full 75-file package, copied with donor permission and verified before and after transfer. Both its exact 1,866-input source identity and runtime pins are checked around execution. Source and package swaps restore the final owned bytes in `finally`.

The first fixed execution matches 93/93 in Chromium and WebKit but 92/93 in Firefox. Firefox's nonprimitive-conversion TypeError includes operand identifiers. The glue originally used `end`/`start`, while the source uses `endRow`/`startRow`. Commit `acb63750` preserves those source names instead of normalizing or weakening the comparison. The complete initial failure is retained under `results/serialize-raw-marker/initial-fixed/`.

The final execution matches 93/93 in all three engines. It also runs every unchanged original SerializeAddon assertion, 120/120 against source and 120/120 against Rust, with zero failed, skipped or unrun tests and no retries. The earlier fixed attempt also passed those complete original suites. The browser runner uses the actual Rust Terminal/SerializeAddon and retains full public-cell restoration traces and three screenshots. These are functionality results for a debug distribution, not speed measurements.

Final `cargo test --locked -j2 -p xterm-web --lib --all-features` passes 56 tests, zero failed, ignored, measured or filtered. Native all-feature library Clippy and WASM library Clippy both pass with `-D warnings`. The exact aggregate `cargo fmt --all -- --check` still fails with Windows OS error 206. All five separate whole-package formatting checks pass; this does not waive the failed aggregate command or close task 304. Full workspace/native-OS/release matrices were not run by task 549. Raw log whitespace is retained, not trimmed to make a check pass.

Both original and physical reference copies pass all 776 manifest entries before and after execution. No source_repo assertion, AGENTS rule, dependency, unrelated producer or canonical main content was changed. Resource grant 4665 was started only after task 547's actual release, and was explicitly released in comment 4710 after all compiler/browser/server processes had exited. TEMP, TMP, TMPDIR and the private Cargo target use literal owned O: paths. This file does not itself confirm final worktree removal.

## Reproduction and retained evidence

From an owned worktree at the final producer source, with the required toolchain, matching wasm-bindgen 0.2.100 and reference dependencies:

```sh
cargo test --locked -j2 -p xterm-web --lib --all-features
cargo clippy --locked -j2 -p xterm-web --lib --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
CARGO_BUILD_JOBS=2 node scripts/build-demo.mjs --debug
SOURCE_ROOT=/path/to/source_repo node scripts/serialize-browser-check.mjs --suite
```

Set literal O: TEMP/TMP/TMPDIR and Cargo target paths before those commands. `WASM_BINDGEN` can select the matching CLI. Do not start compiler/browser work without the current team's bounded allocation. To reproduce the negative control, preserve the final source commit, restore only `crates/xterm-web/src/serialize_addon/options.rs` from `0442e357`, rebuild, and run the same 93-case fixture. It must report the three named failures per engine. Restore the final file and rebuild before checking the fix. Never use an old package with a different source identity.

`results/serialize-raw-marker/` retains complete `before`, `initial-fixed` and `after` browser reports, raw source/Rust traces and original-suite reports. `summary.json` adds per-group/subcase counts without changing their contents. `compile.json`, final native/lint logs, initial compiler logs and copied build report trees retain command outcomes. `package-copy.json` and `borrowed-build-report.json` describe the negative package provenance. `runtime-pins.json` identifies the actual native executable, compiler WASM, served WASM, binding JS, all generated snippets, CSS and build receipt. `producer-pins.json` identifies the changed source, fixture, docs and retained evidence. Runtime locations become historical after cleanup; hashes and source receipts remain durable. `source-node.json` is an additional cached original BrowserTerminal replay under Node, not browser or Rust coverage.
