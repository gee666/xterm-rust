# Mouse coordinate zero signs

Task 532 checks the numeric coordinate policy against unchanged `src/browser/input/Mouse.ts` and `src/browser/services/MouseCoordsService.ts`.

The source uses `Math.min(Math.max(value, lower), upper)`. JavaScript requires the maximum of negative and positive zero to be positive zero, and the minimum to be negative zero. Rust's `f64::max` and `f64::min` permit either operand for equal zero values. The old helper produces a real debug-build discrepancy on Windows with Rust 1.85.1. A standalone test including the unmodified module fails at fixture row 1356. The source report column is negative zero, while Rust returns positive zero. The old optimized build passes the same corpus, so this defect depends on optimization. The production helper now makes both tie choices explicit.

## Reproduction

```
node scripts/mouse-coordinate-zero-oracle.mjs
cargo test -p xterm-web --test mouse_coordinate_zero -j2
cargo test -p xterm-web --test mouse_coordinate_zero --release -j2
WASM_TEST=mouse_coordinate_zero node scripts/browser-host-check.mjs
```

The oracle executes the original functions after removing TypeScript annotations, imports and constructor injection decorators. `getWindow` also comes from unchanged source. It does not rewrite the clamp. `XT_REFERENCE_ROOT` can select a read-only original reference. The fixture records 3,249 directed cases, crossing both zero signs, subnormal values, infinite/NaN coordinates, cell dimensions, canvas bounds, viewport counts and selection offsets. Nine cases have invalid character size. Rust compares number bits, except for NaN payloads, which are not JavaScript-observable here. The source hashes and fixture hash are in `results/mouse-coordinate-zero/source.json`.

Source generation and drift replay passed, both with the physical worktree reference and with the original read-only reference. All 776 original manifest hashes matched before testing. Standalone old debug failed and old optimized passed. After the fix, both standalone builds pass all 3,249 cases. The standalone include reports an unused padding helper because this isolated probe does not invoke it; this is not a production crate lint result. Actual Cargo execution also confirms the defect. Restoring only `mouse_coords.rs` from base `11ad71a8` fails the new test at the same row. A `finally` block restores the fixed bytes and checks their SHA-256 before the final passing rerun. `results/mouse-coordinate-zero/control.json` records both source hashes.

Native Windows/MSRV 1.85.1 checks passed: xterm-web all-target/all-feature tests, 418 tests across 174 summaries; focused release mouse tests, 14 tests across three summaries; native all-target/all-feature warning-denied Clippy; WASM workspace all-feature library Clippy; and WASM compilation/lint of the new test. Each passing test run has zero failed, ignored and filtered tests. `native.json`, `native-counts.json` and the raw logs retain exact commands and results. Workspace-wide rustfmt still fails with Windows OS206. Separate complete formatting checks passed for each of the five member packages. Browser runtime execution is pending. Linux/macOS and overall workspace completion are not claimed. This corpus does not execute complete original suites, implement raw mouse registries, or establish terminal equivalence. Task 305 and the full port remain open.
