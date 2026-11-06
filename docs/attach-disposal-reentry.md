# Attach disposal reentry

Task535 fixes the live iterator in `crates/xterm-addons/src/attach.rs`. Full utility/status family323 and full-port equivalence remain NOT PROVEN.

Upstream AttachAddon.dispose uses `for (const d of this._disposables)`. A socket's removeEventListener can synchronously call activate again, appending handles to that same array. The active disposal must visit those new handles too. Rust previously cloned a snapshot, leaving appended socket and terminal subscriptions active. The repair reads one owned handle at a time from the live vector and releases its borrow before invoking disposal. No new dependency, public API or unsafe code is needed.

The same code runs for explicit dispose and the addon's close/error callbacks. The shared fixture has 54 scenarios covering those three entry points, appending during message/close/error removal, zero/one/three reactivations, and both bidirectional settings. It records socket registration/removal order, remaining reactivations, surviving listeners, terminal writes and socket sends after disposal. The native function expands the same 54 combinations over actual portable AttachAddon adapters. The WASM function uses actual Rust Terminal and AttachAddon exports, not a JS terminal engine. A recording socket supplies deterministic synchronous reentry; it is not a real network transport test.

## Reproduce

Use Node24.15.0, Rust/Cargo1.85.1 and wasm-bindgen CLI0.2.100, plus installed Playwright engines. Use an owned O: worktree, regular private target and O: TEMP/TMP. Obtain a current bounded compiler/browser grant before builds.

```sh
node scripts/attach-disposal-reentry-oracle.mjs
cargo test --locked -j2 -p xterm-addons --test attach_disposal_reentry --test utility_attach
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
WASM_TEST=wasm_attach_disposal_reentry node scripts/browser-host-check.mjs
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_attach_disposal_reentry -- -D warnings
cargo fmt --all -- --check
```

The source oracle strips TypeScript syntax only and runs unchanged original addon methods. It verifies all776 reference hashes before and after execution. The expected trace JSON is a cohesive source-generated table, intentionally over350 lines; do not regenerate it to fit Rust. The original assertions and source_repo files are unchanged.

## Observed results

Native Windows uses the toolchain recorded in `results/attach-disposal-reentry/toolchain.json`. All207 xterm-addons test functions across49 harnesses passed initially and after the negative control, zero failed/ignored/filtered. One new function expands54 reactivation combinations. The exact main11ad71a8 `attach.rs` production file made that function fail on surviving socket handles. The driver restored the fixed source in finally and verified its SHA256 before the final full addon pass.

Warning-denied addon all-target/all-feature Clippy and actual WASM library/new-test Clippy passed. All five whole-member formatting checks and changed Rust formatting passed. Exact aggregate `cargo fmt --all -- --check` failed with Windows OS206; its diagnostic is retained, not promoted to a workspace pass.

Actual Rust/WASM initial and finally restored runs passed the new function in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Each function expands all54 shared scenarios,162 per matrix, not162 additional functions. The exact main11ad production file failed only that new function in Chromium, with the intended surviving-handler trace. All three compilations have `fresh=false`, exact fixture bytes embedded in the compiler artifacts, distinct old/fixed WASM hashes and identical initial/final fixed WASM hashes. Source restoration is hash-verified in finally. The original production file is the only negative-control change.

Retained task404 `wasm_attach_reentry` then passed both functions in every engine, six function executions total. One function expands11 unchanged source-derived callback scenarios; the other uses the existing real WebSocket peer to check nested message ordering and disposal. The peer and browsers close in finally. No new network fixture or dependency junction was created, and no original source assertion was modified.

The unchanged source oracle passed all54 cases on both the physical reference copy and original reference, with all776 hashes checked before and after each run. Raw native/build/browser logs, source/fixture pins and compiler/binding/runtime hashes are under `results/attach-disposal-reentry/`. Local driver and pin-builder text snapshots are receipts, not portable automation claims. Generated runtime binaries remain available only through independent review and safe owned cleanup; recorded runtime paths become historical after cleanup.

## Remaining contracts

Task536 explicitly retains the existing BrowserSocket one-shot removal and panic-on-remove-error behavior. Upstream repeatedly calls the current removeEventListener and propagates raw exceptions. This repair does not change `socket.rs` or claim that broader removal/lifetime contract is solved.

The inspection also found that ProgressAddon binding hardcodes clamping after unary numeric conversion, whereas source resolves Math.min and Math.max before the value getter and can retain arbitrary replacement-method returns. That broader raw-host contract remains full323. No partial Progress repair is included.

Full original addon suites, native Linux/macOS, complete browser/demo behavior, final coverage/comparison deliverables and stable whole-workspace quality gates remain outside this scoped repair. Function counts and expanded scenario counts are different measures and must not be added together.

## Review and cleanup

Task535 was independently accepted at283f5a28 with no findings. The reviewer reran seven pinned native functions and the source oracle on both reference roots, verified37 raw pins/1855 inputs/328 runtime pins and inspected the actual browser/control receipts without rerunning heavy jobs. See `docs/reviews/theodelinda535.md` and `docs/handoffs/theodelinda535.md`. Owned generated artifacts stayed available through review. Subsequent cleanup receipts describe exact unlink/removal phases; recorded runtime paths are historical once removal completes.
