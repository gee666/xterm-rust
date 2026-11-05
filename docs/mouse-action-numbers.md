# Numeric mouse actions

The source `MouseStateService` accepts any numeric `action`, even though its TypeScript declaration names five constants. Protocol restrictions use strict numeric comparisons. The wheel encoder later performs JavaScript bitwise conversion. Rejecting every non-enum number at the native boundary loses observable behavior, and truncating it before restriction changes that behavior.

`MouseAction::RawNumber(f64)` now retains these inputs through `MouseEvent`, builtin policies and custom callbacks. `numeric_value()` returns the original number. Named actions compare numerically with raw actions, including `-0 === 0`. NaN never equals itself. This matches the existing raw-number button representation. `TryFrom<u8>` still validates named constants and rejects other values; callers with unvalidated numeric input use `RawNumber` explicitly. Because the enum can contain NaN it no longer implements `Eq`, and numeric consumers use `numeric_value()` rather than enum casts.

The shared Rust mouse field policy remains the only builtin restriction/encoding implementation. The typed field adapter no longer assumes an enum-only action. Native `MousePolicy::trigger` also compares left/right actions numerically, so raw 2 and 3 cannot bypass its invalid non-wheel combination check. Its existing move comparison and debounce now recognize raw 32 as the same action as `Move`.

## Source-derived checks

Run from the repository root with Node.js 24.15 and the source oracle's existing esbuild dependency available. A physical reference copy may resolve that dependency through `NODE_PATH` pointing to the retained reference's `node_modules`; do not copy or link dependencies into a temporary worktree.

```sh
node scripts/mouse-raw-action-oracle.mjs
cargo test -p xterm-core --test mouse_action_numeric --test mouse_differential
cargo test -p xterm-web --test mouse_action_numeric --test mouse_policy --test mouse_routing --test mouse_failures
```

Use `--generate` only after inspecting source and fixture changes. The oracle executes unchanged source `MouseStateService` through the existing shared loader, then executes the unchanged normalized report/equality methods from `MouseService`. It does not edit source or assertions and does not substitute a JavaScript terminal for Rust tests.

The core fixture contains 19,328 rows. It covers 36 numeric actions, four buttons, all eight modifier combinations, five protocols and three encodings, plus 2,048 seeded finite/fractional/wrapping actions at seed `0x524ac710`. The values include NaN, infinities, negative zero, fractions, strict-comparison boundaries, signed/unsigned 32-bit boundaries and large finite numbers. Every row compares restriction, modifier mutations, untouched action bits and exact UTF-16 encoding. Encoding runs even after restriction rejects, as it is independently callable upstream.

The normalized report fixture contains 2,160 rows through original range/pair checks, coordinate increment, builtin restriction/encoding and output routing. The Rust consumer compares acceptance, coordinate/modifier changes, untouched action bits, output channel and exact UTF-16 payload. Additional native tests check named/raw equality, NaN/negative-zero retention, custom callback mutation and raw/named move debounce.

## Evidence and limits

`results/mouse-raw-actions/source.json` records source hashes, corpus hashes, seed, tool versions and exact row counts. Execution used Node.js 24.15.0 and Rust/Cargo 1.85.1 on native Windows. The original and physical-copy references each matched all 776 manifest entries before testing.

`results/mouse-raw-actions/quality/native.json` retains exact commands, exit statuses, timing and restoration hashes. Raw logs and `counts.json` record:

- Focused debug tests passed 33 core tests and 17 web tests. Focused release tests passed five core tests and 17 web tests.
- `cargo test -j2 --workspace --all-targets --all-features` passed 2,193 tests across 499 harness summaries, with zero failures, ignored tests, filtered tests or warnings. Workspace all-feature doctests passed two tests.
- Warning-denied Clippy passed for native workspace/all-target/all-feature code and WASM workspace/all-feature libraries.
- The exact `1106d480` native `mouse_policy.rs` file, with the new action representation retained, failed the intended normalized-report test. The separate debounce test passed. A `finally` block restored the exact final SHA-256, followed by both new web tests passing. This is a one-file policy control, not an old-core build claim.
- The exact workspace-format command failed with Windows OS error 206. All five complete package-format checks passed separately. The workspace failure remains a failure, not a waived green gate.

`source-pins.json` binds changed Rust/script/fixture bytes to the source commit. `native-pins.json` identifies retained executables for independent replay. Independent task 524 review accepted `18eb6cfa` with no findings. The reviewer verified all nine source pins, all 21 executable hashes, both 776-entry reference copies, raw counts and restoration hashes. It independently ran every pinned executable and replayed the unchanged-source oracle without compilation, browsers or file modifications. See `results/mouse-raw-actions/review.md`. The full workspace tests regenerated three pre-existing output files under `results/buffer-terminal/` and `results/serialize/`. Their hashes and fresh buffer summary were retained in `generated-side-effects.json`, then only those three known private-worktree outputs were restored to their committed historical bytes. No test input or assertion changed. An initial pin-collection command hit Node's default Git-output buffer limit on the large fixture; increasing that read-only command's bounded buffer completed collection. No tests or source changed for that tooling correction.

This work changes numeric representation and native normalized report behavior. It does not implement ordinary-object protocol/encoding registries, inherited keys, arbitrary nonnumeric values or raw callback returns. It does not establish real browser/DOM routing or native Linux/macOS execution. Those checks remain separate tasks. Overall terminal equivalence remains NOT PROVEN.
