# ANSI scrollback array failures

Task498 closes the primitive numeric ANSI scrollback error mismatch retained in post639. It does not close raw346 or the full search/serialization family322. Overall equivalence remains NOT PROVEN.

## Source contract and repair

The source's `_serializeBufferByScrollback` captures the normal buffer length, adds `terminal.rows` to the supplied scrollback, clamps the sum to zero through that captured length, and forms `start = maxRows - correctRows` and `end = maxRows - 1`. `StringSerializeHandler._beforeSerialize` then allocates `new Array(end - start)`. A zero retained count therefore throws on length minus one. Fractional retained counts generally throw on a fractional length. NaN survives the clamp and also throws. Positive infinity clamps to the whole buffer; negative infinity clamps to zero and throws.

The old browser adapter rejected a fractional starting coordinate before the Array check. Its coordinate error message differs from the source in every engine. NaN used a fixed Chromium-style message, which does not match Firefox or WebKit.

The repair is confined to primitive numbers in `scrollback_buffer`. It performs the same floating-point addition and clamp, then passes the actual start and end to the existing `line_range` validator. That validator invokes the existing tiny browser Array bridge only for invalid lengths, preserving the engine's own RangeError name and message without allocating a valid large array. Valid integral ranges still use the Rust serializer. Subtracting the actual endpoints, rather than replacing the difference with `correctRows - 1`, preserves cancellation and rounding at boundary values.

Undefined, strings, objects and other raw values keep their existing path. HTML is untouched, including the separately accepted487 fractional HTML repair that was not in this producer's main5088 base. The thirteen production lines do not delegate terminal parsing, traversal or serialization to JavaScript and add no dependency.

## Probes and reproducible commands

Three additive shared source/Rust cases cover eighteen scrollback inputs, including both infinities, NaN, signed zero, fractional counts, negative clamps, huge finite numbers and rounding near zero and terminal rows. Getter logs verify that Array failure precedes `excludeAltBuffer` and `excludeModes` reads. Separate cases retain exact thrown getter identity after reset/resize and nested serialization while the alternate buffer is active. Full public-cell snapshots retain the side effects. The existing thirty-two seeded restoration cases keep seed `0x05e21a11` and twelve operations per trace.

`results/serialize-scrollback-array/reproduce.mjs` runs from an owned O: worktree. It reuses the unchanged suite runner and its original assertions. Set `PATH` to Git, Node and Rust, `SOURCE_ROOT` to the supplied read-only reference with existing dependencies/generated bundle, and `WASM_BINDGEN` to CLI0.2.100. Set `CARGO_BUILD_JOBS=2`, `CARGO_TARGET_DIR` to the owned O: target, and `TEMP`, then `TMP` and `TMPDIR`, to that worktree's `tmp/runtime`. Each build/browser phase needs the current309 resource grant.

```sh
node results/serialize-scrollback-array/reproduce.mjs quality
node results/serialize-scrollback-array/reproduce.mjs proof
node results/serialize-scrollback-array/reproduce.mjs audit
```

The negative control restores exact `fbea6472` browser serializer source while retaining the new probes. Its finally block restores the entry source/package/pointer, including failure paths. Source swaps use `writeFile`, not `copyFile`, to prevent Windows timestamp reuse from hiding a stale Cargo WASM artifact. Both variants must report `fresh=false` for the actual web artifact and produce distinct WASM hashes before the proof is accepted. The fixed package remains available for independent review, bound to its source hash.

A deliberately nonexistent rustc was tested before any browser run. It exits one and the helper restores the exact entry source/pointer hashes and package absence. `failure-restoration/check.json` records this helper-only check; it is not Rust runtime evidence.

## Current evidence and limits

Windows Rust/Cargo1.85.1 scoped native checks pass sixteen serialization test executables with sixty-nine tests, no failures or ignored tests. Warning-denied core library plus these tests and actual WASM web-library Clippy pass. Exact workspace `cargo fmt --all -- --check` fails with Windows OS206; all five complete member format checks pass. The workspace failure is not waived. Raw commands and results are in `quality.json` and its logs.

The baseline matches75/77 cases in Chromium148.0.7778.96 and74/77 in Firefox150.0.2 and WebKit26.4. Only the new ANSI scrollback cases differ. Fractional-coordinate messages fail in all three engines. Chromium already matched the old fixed NaN message, while Firefox and WebKit also fail the nested NaN case. The unmodified runner exits one for this negative control. Fixed matches77/77 cases in each engine with zero mismatches, and the runner exits zero. Exact traces, getter logs, sentinel identity, terminal snapshots and screenshots are under `baseline/browser/` and `fixed/browser/`.

Both phases also pass the unchanged original serializer suites against source and Rust, each120/120 expanded browser cases with zero failed, skipped or unrun. Each implementation has forty cases per engine. These suite totals are separate from the77-case differential corpus. Both server sets close without cleanup errors, and both before/after reference checks match all776 files.

The actual web artifacts report `fresh=false` in both builds. Baseline WASM hash is `144a977cfd63516d4386c4d9c33055c44124ec608c2ce19309f216a5ebe5df4b`; fixed is `e86f82c26e8985685c13cd35034ff9ccddcf2e86652cba1c1d2b22b19abaefde`. Corresponding input hashes over1773 files are `54cae3a63b1e87581ad596173227b00b879ad50290baa44f0718e9563bda3318` and `00afd76e208aef4dd3e7c763226efbd8d43eb2a7de14a7fafb38480bdead5f7e`. `summary.json`, the two `variant.json` files and copied build logs retain those identities. These are debug test-transport artifacts, not distribution-size measurements.

Preliminary independent source review found no implementation defect, verified all sixteen native binary pins and776 reference files, and reran fifteen executables with67 passing tests. It left the two report-writing differential tests unrun to preserve generated output. Final browser/pin review follows the proof.

Full all-feature/workspace/release/native Linux/macOS matrices, other raw serialization inputs, replaced public buffer methods, palette mutation and full-family acceptance remain separate obligations. Accepted487 is preserved on its producer branch and must be composed at integration, not replayed or silently dropped. Composing both producers requires retaining all three new ANSI cases and all four487 HTML cases, then changing the runner count from this branch's77 to81. No full81-case integrated browser result is claimed here.
